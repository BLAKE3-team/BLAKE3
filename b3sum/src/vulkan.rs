use anyhow::Result;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};
use blake3::gpu::{shaders, GpuControl};
use blake3::{CHUNK_LEN, OUT_LEN};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::iter;
use std::mem::ManuallyDrop;
use std::ops::{BitAnd, BitOr, Deref, DerefMut};
use std::ptr::{self, NonNull};

pub struct GpuTask<'a> {
    pending: bool,

    cycle: u8,
    tail: u8,
    head: u8,

    input: MappedMemory,
    output: [MappedMemory; 2],

    descriptor_sets: &'a [[vk::DescriptorSet; 2]],
    command_buffer: &'a vk::CommandBuffer,
    fence: &'a vk::Fence,

    gpu: &'a GpuInstance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GpuTaskWaitResult {
    pub has_output: bool,
    pub has_more: bool,
}

impl GpuTask<'_> {
    #[inline]
    pub fn is_pending(&self) -> bool {
        self.pending
    }

    #[inline]
    pub fn input_buffer(&mut self) -> &mut [u8] {
        assert!(!self.pending);

        // Safety: not in use by the GPU (not pending), mutable borrow of self
        unsafe { self.input.as_mut_slice() }
    }

    #[inline]
    pub fn output_buffer(&mut self) -> &mut [u8] {
        let output =
            &mut self.output[self.descriptor_sets.len().wrapping_add(self.cycle as usize) & 1];

        // Safety: GPU is using the other output buffer, mutable borrow of self
        unsafe { output.as_mut_slice() }
    }

    pub fn submit(&mut self, control: &GpuControl, input: bool) -> Result<()> {
        assert!(!self.pending);

        if input {
            assert_eq!(self.tail, 0);
        } else {
            assert!(self.tail < self.head);
            self.tail += 1;
        }

        unsafe {
            let device = &self.gpu.device;
            let command_buffer = self.command_buffer;
            let pipeline_layout = &self.gpu.pipeline_layout;
            let chunk_pipeline = &self.gpu.chunk_pipeline;
            let parent_pipeline = &self.gpu.parent_pipeline;
            let group_counts = &self.gpu.group_counts;

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(*command_buffer, &begin_info)?;

            device.cmd_push_constants(
                *command_buffer,
                *pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                control.as_bytes(),
            );

            let mut bound = false;
            for step in self.tail..=self.head {
                if step == 0 {
                    device.cmd_bind_pipeline(
                        *command_buffer,
                        vk::PipelineBindPoint::COMPUTE,
                        *chunk_pipeline,
                    );
                } else if !bound {
                    device.cmd_bind_pipeline(
                        *command_buffer,
                        vk::PipelineBindPoint::COMPUTE,
                        *parent_pipeline,
                    );
                    bound = true;
                }

                let descriptor_set = &self.descriptor_sets[step as usize]
                    [(step.wrapping_add(self.cycle) & 1) as usize];
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    *pipeline_layout,
                    0,
                    &[*descriptor_set],
                    &[],
                );

                device.cmd_dispatch(*command_buffer, group_counts[step as usize], 1, 1);
            }

            device.end_command_buffer(*command_buffer)?;

            let queue = &self.gpu.queue;
            let fence = self.fence;

            let command_buffers = [*command_buffer];
            let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers);
            device.queue_submit(*queue, &[*submit_info], *fence)?;

            self.pending = true;
        }

        Ok(())
    }

    pub fn wait(&mut self) -> Result<GpuTaskWaitResult> {
        assert!(self.pending);

        let device = &self.gpu.device;
        let fence = self.fence;
        let command_buffer = self.command_buffer;

        unsafe {
            device.wait_for_fences(&[*fence], true, u64::max_value())?;
            device.reset_fences(&[*fence])?;

            self.pending = false;
        }

        self.cycle = self.cycle.wrapping_add(1);

        let fill = (self.head as usize) < self.descriptor_sets.len() - 1;
        if fill {
            self.head += 1;
        }

        unsafe {
            // The implicit reset done by device.begin_command_buffer()
            // releases the resources to the pool, instead of reusing them.
            device.reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())?;
        }

        Ok(GpuTaskWaitResult {
            has_output: !fill,
            has_more: self.tail < self.head,
        })
    }
}

impl Drop for GpuTask<'_> {
    fn drop(&mut self) {
        if self.pending {
            unsafe {
                // The device might still be processing the command buffer.
                // Wait for it to finish before releasing any resources.
                let _ = self
                    .gpu
                    .device
                    .wait_for_fences(&[*self.fence], true, u64::max_value());
            }
        }
    }
}

pub struct GpuInstance {
    task_count: usize,
    input_buffer_size: usize,
    group_counts: Vec<u32>,

    descriptor_sets: Vec<Vec<[vk::DescriptorSet; 2]>>,
    descriptor_pool: vk::DescriptorPool,

    input_mapped: Vec<MappedMemory>,
    output_mapped: Vec<MappedMemory>,

    input_buffers: Vec<vk::Buffer>,
    output_buffers: Vec<Vec<vk::Buffer>>,

    input_memory: Vec<vk::DeviceMemory>,
    device_memory: vk::DeviceMemory,
    output_memory: vk::DeviceMemory,

    chunk_pipeline: vk::Pipeline,
    parent_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    chunk_shader_module: vk::ShaderModule,
    parent_shader_module: vk::ShaderModule,

    fences: Vec<vk::Fence>,
    command_buffers: Vec<vk::CommandBuffer>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    device: Device,
    instance: Instance,
    _entry: Entry,
}

impl Drop for GpuInstance {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.input_buffers
                .iter()
                .for_each(|buffer| self.device.destroy_buffer(*buffer, None));
            self.output_buffers
                .iter()
                .flatten()
                .for_each(|buffer| self.device.destroy_buffer(*buffer, None));

            self.input_memory
                .iter()
                .for_each(|memory| self.device.free_memory(*memory, None));
            self.device.free_memory(self.device_memory, None);
            self.device.free_memory(self.output_memory, None);

            self.device.destroy_pipeline(self.chunk_pipeline, None);
            self.device.destroy_pipeline(self.parent_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device
                .destroy_shader_module(self.chunk_shader_module, None);
            self.device
                .destroy_shader_module(self.parent_shader_module, None);

            self.fences
                .iter()
                .for_each(|fence| self.device.destroy_fence(*fence, None));
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);

            // Safety: dropping Entry unloads the Vulkan library, all objects
            // created or allocated from it were destroyed or freed above.
        }
    }
}

impl GpuInstance {
    pub fn new(task_count: usize, input_buffer_size: usize) -> Result<Option<Self>> {
        assert!(
            input_buffer_size.is_power_of_two(),
            "invalid input buffer size"
        );
        assert!(
            input_buffer_size <= (1 << 30),
            "input buffer size too large"
        );

        let input_group_count = input_buffer_size / (CHUNK_LEN * shaders::blake3::WORKGROUP_SIZE);
        assert!(input_group_count > 0, "input buffer size too small");

        let group_counts = iter::successors(Some(input_group_count as u32), |&count| {
            if count > 1 {
                Some(count / 2)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        let entry = Entry::new()?;
        let instance = Self::create_instance(&entry)?;
        let (physical_device, queue_family_index) = match Self::get_physical_device(&instance)? {
            Some(t) => t,
            None => return Ok(None),
        };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let device = Self::create_device(&instance, physical_device, queue_family_index)?;
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let command_pool = Self::create_command_pool(&device, queue_family_index)?;
        let command_buffers = Self::allocate_command_buffers(&device, &command_pool, task_count)?;
        let fences = Self::create_fences(&device, task_count)?;

        let chunk_shader_module =
            Self::create_shader_module(&device, shaders::blake3::chunk_shader())?;
        let parent_shader_module =
            Self::create_shader_module(&device, shaders::blake3::parent_shader())?;
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device)?;
        let pipeline_layout = Self::create_pipeline_layout(&device, &descriptor_set_layout)?;
        let (chunk_pipeline, parent_pipeline) = Self::create_pipelines(
            &device,
            &pipeline_layout,
            &chunk_shader_module,
            &parent_shader_module,
        )?;

        let output_buffer_sizes = group_counts
            .iter()
            .map(|&count| count as usize * shaders::blake3::WORKGROUP_SIZE * OUT_LEN)
            .flat_map(|size| iter::repeat(size).take(2))
            .collect::<Vec<_>>();

        let (input_buffers, output_buffers) =
            Self::create_buffers(&device, task_count, input_buffer_size, &output_buffer_sizes)?;
        let (input_memory, device_memory, output_memory, input_mapped, output_mapped) =
            Self::allocate_memory(
                &device,
                &memory_properties,
                &input_buffers,
                &output_buffers,
                input_buffer_size,
                &output_buffer_sizes,
            )?;
        let (descriptor_pool, descriptor_sets) = Self::allocate_descriptor_sets(
            &device,
            &descriptor_set_layout,
            &input_buffers,
            &output_buffers,
        )?;

        Ok(Some(Self {
            task_count,
            input_buffer_size,
            group_counts,

            descriptor_sets,
            descriptor_pool: descriptor_pool.take(),

            input_mapped,
            output_mapped,

            input_buffers: input_buffers.take(),
            output_buffers: output_buffers.take(),

            input_memory: input_memory.take(),
            device_memory: device_memory.take(),
            output_memory: output_memory.take(),

            chunk_pipeline: chunk_pipeline.take(),
            parent_pipeline: parent_pipeline.take(),
            pipeline_layout: pipeline_layout.take(),
            descriptor_set_layout: descriptor_set_layout.take(),
            chunk_shader_module: chunk_shader_module.take(),
            parent_shader_module: parent_shader_module.take(),

            fences: fences.take(),
            command_buffers,
            command_pool: command_pool.take(),
            queue,
            device: device.take(),
            instance: instance.take(),
            _entry: entry,
        }))
    }

    #[inline]
    pub fn input_buffer_size(&self) -> usize {
        self.input_buffer_size
    }

    pub fn tasks(&mut self) -> Vec<GpuTask> {
        let mut tasks = Vec::with_capacity(self.task_count);
        for task in 0..self.task_count {
            tasks.push(GpuTask {
                pending: false,

                cycle: 0,
                tail: 0,
                head: 0,

                // Safety: disjoint sets, mutable borrow of self
                input: self.input_mapped[task].clone(),
                output: [
                    self.output_mapped[task * 2].clone(),
                    self.output_mapped[task * 2 + 1].clone(),
                ],

                descriptor_sets: &self.descriptor_sets[task],
                command_buffer: &self.command_buffers[task],
                fence: &self.fences[task],

                gpu: self,
            });
        }
        tasks
    }

    fn create_instance(entry: &Entry) -> Result<impl Wrap<Instance>> {
        let application_name = CString::new(env!("CARGO_PKG_NAME")).unwrap();
        let application_version = vk::make_version(
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        );

        let application_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(application_version)
            .api_version(vk::make_version(1, 0, 0));
        let create_info = vk::InstanceCreateInfo::builder().application_info(&application_info);

        Ok(Guard::new(
            unsafe { entry.create_instance(&create_info, None)? },
            |instance| unsafe { instance.destroy_instance(None) },
        ))
    }

    fn get_physical_device(instance: &Instance) -> Result<Option<(vk::PhysicalDevice, u32)>> {
        Ok(unsafe {
            instance
                .enumerate_physical_devices()?
                .into_iter()
                .filter_map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(physical_device)
                        .into_iter()
                        .enumerate()
                        .find(|&(_, queue_family_properties)| {
                            queue_family_properties
                                .queue_flags
                                .contains(vk::QueueFlags::COMPUTE)
                        })
                        .map(|(queue_family_index, _)| (physical_device, queue_family_index))
                })
                .min_by_key(|&(physical_device, _)| {
                    match instance
                        .get_physical_device_properties(physical_device)
                        .device_type
                    {
                        // When both are available, prefer integrated GPU.
                        // This avoids waking up the discrete GPU.
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                        vk::PhysicalDeviceType::DISCRETE_GPU => 2,
                        vk::PhysicalDeviceType::VIRTUAL_GPU => 3,
                        vk::PhysicalDeviceType::CPU => 4,
                        _ => 5,
                    }
                })
                .map(|(physical_device, queue_family_index)| {
                    (physical_device, queue_family_index as u32)
                })
        })
    }

    fn create_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<impl Wrap<Device>> {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[0.5]);

        let queue_create_infos = [*queue_create_info];
        let create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_infos);

        Ok(Guard::new(
            unsafe { instance.create_device(physical_device, &create_info, None)? },
            |device| unsafe { device.destroy_device(None) },
        ))
    }

    fn create_command_pool<'a>(
        device: &'a Device,
        queue_family_index: u32,
    ) -> Result<impl Wrap<vk::CommandPool> + 'a> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        Ok(Guard::new(
            unsafe { device.create_command_pool(&create_info, None)? },
            move |command_pool| unsafe { device.destroy_command_pool(*command_pool, None) },
        ))
    }

    fn allocate_command_buffers(
        device: &Device,
        command_pool: &vk::CommandPool,
        task_count: usize,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(task_count as u32);
        Ok(unsafe { device.allocate_command_buffers(&allocate_info)? })
    }

    fn create_fences<'a>(
        device: &'a Device,
        task_count: usize,
    ) -> Result<impl Wrap<Vec<vk::Fence>> + 'a> {
        let mut fences = Guard::new(Vec::with_capacity(task_count), move |fences| {
            fences
                .iter_mut()
                .for_each(|fence| unsafe { device.destroy_fence(*fence, None) })
        });

        let create_info = vk::FenceCreateInfo::default();
        for _ in 0..task_count {
            fences.push(unsafe { device.create_fence(&create_info, None)? });
        }

        Ok(fences)
    }

    fn create_shader_module<'a>(
        device: &'a Device,
        code: &[u8],
    ) -> Result<impl Wrap<vk::ShaderModule> + 'a> {
        let code = ash::util::read_spv(&mut Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        Ok(Guard::new(
            unsafe { device.create_shader_module(&create_info, None)? },
            move |shader_module| unsafe { device.destroy_shader_module(*shader_module, None) },
        ))
    }

    fn create_descriptor_set_layout<'a>(
        device: &'a Device,
    ) -> Result<impl Wrap<vk::DescriptorSetLayout> + 'a> {
        let binding0 = vk::DescriptorSetLayoutBinding::builder()
            .binding(shaders::blake3::INPUT_BUFFER_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE);
        let binding1 = vk::DescriptorSetLayoutBinding::builder()
            .binding(shaders::blake3::OUTPUT_BUFFER_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE);

        let bindings = [*binding0, *binding1];
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        Ok(Guard::new(
            unsafe { device.create_descriptor_set_layout(&create_info, None)? },
            move |descriptor_set_layout| unsafe {
                device.destroy_descriptor_set_layout(*descriptor_set_layout, None)
            },
        ))
    }

    fn create_pipeline_layout<'a>(
        device: &'a Device,
        descriptor_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<impl Wrap<vk::PipelineLayout> + 'a> {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(shaders::blake3::CONTROL_UNIFORM_SIZE as u32);

        let set_layouts = [*descriptor_set_layout];
        let push_constant_ranges = [*push_constant_range];
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        Ok(Guard::new(
            unsafe { device.create_pipeline_layout(&create_info, None)? },
            move |pipeline_layout| unsafe {
                device.destroy_pipeline_layout(*pipeline_layout, None)
            },
        ))
    }

    fn create_pipelines<'a>(
        device: &'a Device,
        pipeline_layout: &vk::PipelineLayout,
        chunk_shader_module: &vk::ShaderModule,
        parent_shader_module: &vk::ShaderModule,
    ) -> Result<(impl Wrap<vk::Pipeline> + 'a, impl Wrap<vk::Pipeline> + 'a)> {
        let main = CStr::from_bytes_with_nul(b"main\0").unwrap();

        let chunk_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(*chunk_shader_module)
            .name(main);
        let chunk_create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(*chunk_stage)
            .layout(*pipeline_layout);

        let parent_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(*parent_shader_module)
            .name(main);
        let parent_create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(*parent_stage)
            .layout(*pipeline_layout);

        let create_infos = [*chunk_create_info, *parent_create_info];
        let result = unsafe {
            device.create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None)
        };
        let destroy =
            move |pipeline: &mut vk::Pipeline| unsafe { device.destroy_pipeline(*pipeline, None) };

        let err = |mut pipelines: Vec<vk::Pipeline>, result: vk::Result| {
            pipelines
                .iter_mut()
                .filter(|pipeline| **pipeline != vk::Pipeline::null())
                .for_each(destroy);
            Err(result.into())
        };
        match result {
            Ok(pipelines) => {
                if let [chunk_pipeline, parent_pipeline] = *pipelines {
                    Ok((
                        Guard::new(chunk_pipeline, destroy),
                        Guard::new(parent_pipeline, destroy),
                    ))
                } else {
                    err(pipelines, vk::Result::ERROR_UNKNOWN)
                }
            }
            Err((pipelines, result)) => err(pipelines, result),
        }
    }

    fn create_buffers<'a>(
        device: &'a Device,
        task_count: usize,
        input_buffer_size: usize,
        output_buffer_sizes: &[usize],
    ) -> Result<(
        impl Wrap<Vec<vk::Buffer>> + 'a,
        impl Wrap<Vec<Vec<vk::Buffer>>> + 'a,
    )> {
        let create_buffer = |size| unsafe {
            let create_info = vk::BufferCreateInfo::builder()
                .size(size as u64)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            device.create_buffer(&create_info, None)
        };
        let destroy =
            move |buffer: &mut vk::Buffer| unsafe { device.destroy_buffer(*buffer, None) };

        let mut input_buffers = Guard::new(Vec::with_capacity(task_count), move |buffers| {
            buffers.iter_mut().for_each(destroy)
        });
        let mut output_buffers = Guard::new(Vec::with_capacity(task_count), move |buffers| {
            buffers.iter_mut().flatten().for_each(destroy)
        });

        for task in 0..task_count {
            input_buffers.push(create_buffer(input_buffer_size)?);
            output_buffers.push(Vec::with_capacity(output_buffer_sizes.len()));

            let buffers = &mut output_buffers[task];
            for size in output_buffer_sizes {
                buffers.push(create_buffer(*size)?);
            }
        }

        Ok((input_buffers, output_buffers))
    }

    fn allocate_memory<'a>(
        device: &'a Device,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        input_buffers: &[vk::Buffer],
        output_buffers: &[Vec<vk::Buffer>],
        input_buffer_size: usize,
        output_buffer_sizes: &[usize],
    ) -> Result<(
        impl Wrap<Vec<vk::DeviceMemory>> + 'a,
        impl Wrap<vk::DeviceMemory> + 'a,
        impl Wrap<vk::DeviceMemory> + 'a,
        Vec<MappedMemory>,
        Vec<MappedMemory>,
    )> {
        let memory_type_index = |size, memory_type_bits, property_flags| {
            memory_properties.memory_types[..memory_properties.memory_type_count as usize]
                .iter()
                .enumerate()
                .find(|&(index, memory_type)| {
                    (memory_type_bits & (1 << index)) != 0
                        && memory_type.property_flags.contains(property_flags)
                        && memory_properties.memory_heaps[memory_type.heap_index as usize].size
                            >= size
                })
                .map(|(index, _)| index as u32)
        };

        let host_visible_index = |size, memory_type_bits, prefer_flag| {
            memory_type_index(
                size,
                memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT
                    | prefer_flag,
            )
            .or_else(|| {
                memory_type_index(
                    size,
                    memory_type_bits,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
            })
            .ok_or(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY)
        };
        let device_local_index = |size, memory_type_bits| {
            memory_type_index(
                size,
                memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .ok_or(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY)
        };

        let allocate_memory = |size, memory_type_index| unsafe {
            let create_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(size)
                .memory_type_index(memory_type_index);

            device.allocate_memory(&create_info, None)
        };
        let destroy =
            move |memory: &mut vk::DeviceMemory| unsafe { device.free_memory(*memory, None) };

        let mut input_memory = Guard::new(Vec::with_capacity(input_buffers.len()), move |memory| {
            memory.iter_mut().for_each(destroy)
        });
        let mut input_mapped = Vec::with_capacity(input_buffers.len());

        for buffer in input_buffers {
            let memory_requirements = unsafe { device.get_buffer_memory_requirements(*buffer) };
            let memory_type_index = host_visible_index(
                memory_requirements.size,
                memory_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;

            let memory = allocate_memory(memory_requirements.size, memory_type_index)?;
            input_memory.push(memory);

            unsafe {
                device.bind_buffer_memory(*buffer, memory, 0)?;

                assert!(memory_requirements.size >= input_buffer_size as u64);
                let mapped = device.map_memory(
                    memory,
                    0,
                    input_buffer_size as u64,
                    vk::MemoryMapFlags::empty(),
                )? as *mut u8;
                input_mapped.push(MappedMemory::new(mapped, 0, input_buffer_size));
            }
        }

        let align = |size: u64, alignment_mask: u64| (size + alignment_mask) & !alignment_mask;
        let allocation_requirements = |requirements: &[vk::MemoryRequirements]| {
            let memory_type_bits = requirements
                .iter()
                .map(|memory_requirements| memory_requirements.memory_type_bits)
                .fold(!0, BitAnd::bitand);
            let alignment_mask = requirements
                .iter()
                .map(|memory_requirements| memory_requirements.alignment - 1)
                .fold(0, BitOr::bitor);
            let size = requirements
                .iter()
                .map(|memory_requirements| align(memory_requirements.size, alignment_mask))
                .sum();

            (size, alignment_mask, memory_type_bits)
        };

        let device_memory = {
            let buffer_sizes = &output_buffer_sizes[..output_buffer_sizes.len() - 2];
            let buffers = output_buffers
                .iter()
                .flat_map(|buffers| &buffers[..buffers.len() - 2])
                .collect::<Vec<_>>();
            debug_assert_eq!(buffers.len(), buffer_sizes.len() * output_buffers.len());

            let requirements = buffers
                .iter()
                .map(|&buffer| unsafe { device.get_buffer_memory_requirements(*buffer) })
                .collect::<Vec<_>>();
            let (mut size, alignment_mask, memory_type_bits) =
                allocation_requirements(&requirements);

            if size == 0 {
                size = alignment_mask + 1;
            }

            let memory_type_index = device_local_index(size, memory_type_bits)?;
            let memory = Guard::new(allocate_memory(size, memory_type_index)?, destroy);

            let mut offset = 0;
            for (&buffer, &buffer_size) in buffers.iter().zip(buffer_sizes.iter().cycle()) {
                let aligned_size = align(buffer_size as u64, alignment_mask);
                assert!(size >= aligned_size);

                unsafe {
                    device.bind_buffer_memory(*buffer, *memory, offset)?;
                }

                offset += aligned_size;
                size -= aligned_size;
            }

            memory
        };

        let mut output_mapped = Vec::with_capacity(output_buffers.len() * 2);

        let output_memory = {
            let buffer_sizes = &output_buffer_sizes[output_buffer_sizes.len() - 2..];
            let buffers = output_buffers
                .iter()
                .flat_map(|buffers| &buffers[buffers.len() - 2..])
                .collect::<Vec<_>>();
            debug_assert_eq!(buffers.len(), buffer_sizes.len() * output_buffers.len());
            debug_assert_eq!(output_mapped.capacity(), buffers.len());

            let requirements = buffers
                .iter()
                .map(|&buffer| unsafe { device.get_buffer_memory_requirements(*buffer) })
                .collect::<Vec<_>>();
            let (mut size, alignment_mask, memory_type_bits) =
                allocation_requirements(&requirements);

            let memory_type_index =
                host_visible_index(size, memory_type_bits, vk::MemoryPropertyFlags::HOST_CACHED)?;
            let memory = Guard::new(allocate_memory(size, memory_type_index)?, destroy);

            let mapped = unsafe {
                device.map_memory(*memory, 0, size, vk::MemoryMapFlags::empty())? as *mut u8
            };

            let mut offset = 0;
            for (&buffer, &buffer_size) in buffers.iter().zip(buffer_sizes.iter().cycle()) {
                let aligned_size = align(buffer_size as u64, alignment_mask);
                assert!(size >= aligned_size);

                unsafe {
                    device.bind_buffer_memory(*buffer, *memory, offset)?;

                    output_mapped.push(MappedMemory::new(mapped, offset as usize, buffer_size));
                }

                offset += aligned_size;
                size -= aligned_size;
            }

            memory
        };

        Ok((
            input_memory,
            device_memory,
            output_memory,
            input_mapped,
            output_mapped,
        ))
    }

    fn allocate_descriptor_sets<'a>(
        device: &'a Device,
        descriptor_set_layout: &vk::DescriptorSetLayout,
        input_buffers: &[vk::Buffer],
        output_buffers: &[Vec<vk::Buffer>],
    ) -> Result<(
        impl Wrap<vk::DescriptorPool> + 'a,
        Vec<Vec<[vk::DescriptorSet; 2]>>,
    )> {
        assert_eq!(shaders::blake3::INPUT_BUFFER_BINDING, 0);
        assert_eq!(shaders::blake3::OUTPUT_BUFFER_BINDING, 1);

        let set_count = output_buffers.iter().map(|buffers| buffers.len()).sum();
        let pool_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(set_count as u32 * 2);

        let pool_sizes = [*pool_size];
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(set_count as u32)
            .pool_sizes(&pool_sizes);

        let descriptor_pool = Guard::new(
            unsafe { device.create_descriptor_pool(&create_info, None)? },
            move |descriptor_pool| unsafe {
                device.destroy_descriptor_pool(*descriptor_pool, None)
            },
        );

        let set_layouts = vec![*descriptor_set_layout; set_count];
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(*descriptor_pool)
            .set_layouts(&set_layouts);
        let mut allocated =
            VecDeque::from(unsafe { device.allocate_descriptor_sets(&allocate_info)? });

        let update_descriptor_set = |descriptor_set, input: &vk::Buffer, output: &vk::Buffer| {
            let buffer_info0 = vk::DescriptorBufferInfo::builder()
                .buffer(*input)
                .offset(0)
                .range(vk::WHOLE_SIZE);
            let buffer_info1 = vk::DescriptorBufferInfo::builder()
                .buffer(*output)
                .offset(0)
                .range(vk::WHOLE_SIZE);

            let buffer_info = [*buffer_info0, *buffer_info1];
            let write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_info);
            unsafe { device.update_descriptor_sets(&[*write], &[]) }
        };

        let mut descriptor_sets = Vec::with_capacity(output_buffers.len());

        for (input, buffers) in input_buffers.iter().zip(output_buffers.iter()) {
            let mut task_sets = Vec::with_capacity(buffers.len() / 2);

            let mut input = [input, input];

            for output in buffers.chunks_exact(2) {
                let sets = [
                    allocated.pop_front().unwrap(),
                    allocated.pop_front().unwrap(),
                ];

                update_descriptor_set(sets[0], input[0], &output[0]);
                update_descriptor_set(sets[1], input[1], &output[1]);

                input = [&output[0], &output[1]];

                task_sets.push(sets);
            }

            descriptor_sets.push(task_sets);
        }
        debug_assert!(allocated.is_empty());

        Ok((descriptor_pool, descriptor_sets))
    }
}

#[derive(Clone, Debug)]
struct MappedMemory(NonNull<[u8]>);

unsafe impl Send for MappedMemory {}

impl MappedMemory {
    #[inline]
    unsafe fn new(ptr: *mut u8, offset: usize, size: usize) -> Self {
        Self(NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
            ptr.add(offset),
            size,
        )))
    }

    #[inline]
    unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

struct Guard<T, D: FnOnce(&mut T)> {
    value: ManuallyDrop<T>,
    destroy: Option<D>,
}

impl<T, D: FnOnce(&mut T)> Guard<T, D> {
    #[inline]
    fn new(value: T, destroy: D) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            destroy: Some(destroy),
        }
    }
}

impl<T, D: FnOnce(&mut T)> Deref for Guard<T, D> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, D: FnOnce(&mut T)> DerefMut for Guard<T, D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, D: FnOnce(&mut T)> Drop for Guard<T, D> {
    #[inline]
    fn drop(&mut self) {
        if let Some(destroy) = self.destroy.take() {
            // Safety: take wasn't called since self.destroy is not None
            unsafe {
                destroy(&mut self.value);
                ManuallyDrop::drop(&mut self.value);
            }
        }
    }
}

trait Wrap<T>: Deref<Target = T> + DerefMut {
    fn take(self) -> T;
}

impl<T, D: FnOnce(&mut T)> Wrap<T> for Guard<T, D> {
    #[inline]
    fn take(mut self) -> T {
        // Safety: can only be called once since this method takes ownership
        // Safety: won't be destroyed and dropped when self.destroy is None
        unsafe {
            self.destroy = None;
            ManuallyDrop::take(&mut self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use blake3::gpu::GpuHasher;
    use blake3::join::RayonJoin;

    fn selftest_seq(len: usize) -> Vec<u8> {
        let seed = len as u32;
        let mut out = Vec::with_capacity(len);

        let mut a = seed.wrapping_mul(0xDEAD4BAD);
        let mut b = 1;

        for _ in 0..len {
            let t = a.wrapping_add(b);
            a = b;
            b = t;
            out.push((t >> 24) as u8);
        }

        out
    }

    #[test]
    fn selftest_chunk_shader_only() -> Result<()> {
        let input_size = shaders::blake3::WORKGROUP_SIZE * CHUNK_LEN;
        let mut instance = GpuInstance::new(3, input_size)?.expect("No GPU found");
        let mut tasks = instance.tasks();
        let task = &mut tasks[0];

        let hasher = GpuHasher::new();
        let control = hasher.gpu_control(0);

        let data = selftest_seq(input_size);
        task.input_buffer().copy_from_slice(&data);
        task.submit(&control, true)?;

        let mut expected = vec![0; task.output_buffer().len()];
        hasher.simulate_chunk_shader::<RayonJoin>(
            input_size / CHUNK_LEN,
            &data,
            &mut expected,
            &control,
        );
        GpuHasher::swap_endian::<RayonJoin>(&mut expected);

        let result = task.wait()?;
        assert_eq!(
            result,
            GpuTaskWaitResult {
                has_output: true,
                has_more: false,
            }
        );

        assert_eq!(task.output_buffer(), &*expected);

        Ok(())
    }

    #[test]
    fn selftest_both_shaders() -> Result<()> {
        let input_size = 2 * shaders::blake3::WORKGROUP_SIZE * CHUNK_LEN;
        let mut instance = GpuInstance::new(3, input_size)?.expect("No GPU found");
        let mut tasks = instance.tasks();
        let task = &mut tasks[0];

        let hasher = GpuHasher::new();
        let control = hasher.gpu_control(0);

        let data = selftest_seq(input_size);
        task.input_buffer().copy_from_slice(&data);
        task.submit(&control, true)?;

        let mut expected = vec![0; task.output_buffer().len()];
        let mut buffer = vec![0; task.output_buffer().len() * 2];
        hasher.simulate_chunk_shader::<RayonJoin>(
            input_size / CHUNK_LEN,
            &data,
            &mut buffer,
            &control,
        );
        hasher.simulate_parent_shader::<RayonJoin>(
            input_size / CHUNK_LEN / 2,
            &buffer,
            &mut expected,
            &control,
        );
        GpuHasher::swap_endian::<RayonJoin>(&mut expected);

        let result = task.wait()?;
        assert_eq!(
            result,
            GpuTaskWaitResult {
                has_output: false,
                has_more: true,
            }
        );

        task.submit(&control, false)?;
        let result = task.wait()?;
        assert_eq!(
            result,
            GpuTaskWaitResult {
                has_output: true,
                has_more: false,
            }
        );

        assert_eq!(task.output_buffer(), &*expected);

        Ok(())
    }
}
