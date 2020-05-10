use anyhow::{Context, Result};
use blake3::gpu::GpuControl;
use blake3::{BLOCK_LEN, CHUNK_LEN, OUT_LEN};
use std::iter;
use std::ops::DerefMut;
use std::sync::Arc;
use vulkano::app_info_from_cargo_toml;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess};
use vulkano::command_buffer::submit::SubmitCommandBufferBuilder;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutAbstract};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::instance::{
    Instance, InstanceExtensions, PhysicalDevice, PhysicalDeviceType, QueueFamily,
};
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::{self, Fence};

pub use vulkano::device::Queue;

mod shaders;

pub struct GpuTask {
    locked: bool,
    pending: bool,

    command_buffer: AutoCommandBuffer,
    fence: Fence,

    input_buffer: Arc<CpuAccessibleBuffer<[u8]>>,
    output_buffer: Arc<CpuAccessibleBuffer<[u8]>>,

    chunk_control: Arc<CpuAccessibleBuffer<[u8]>>,
    parent_control: Arc<CpuAccessibleBuffer<[u8]>>,
}

impl GpuTask {
    // Safety: this buffer might be uninitialized.
    pub unsafe fn lock_input_buffer<'a>(&'a self) -> Result<impl DerefMut<Target = [u8]> + 'a> {
        Ok(self.input_buffer.write()?)
    }

    // Safety: this buffer might be uninitialized.
    pub unsafe fn lock_output_buffer<'a>(&'a self) -> Result<impl DerefMut<Target = [u8]> + 'a> {
        Ok(self.output_buffer.write()?)
    }

    pub fn write_chunk_control(&self, control: &GpuControl) -> Result<()> {
        self.chunk_control
            .write()?
            .copy_from_slice(control.as_bytes());
        Ok(())
    }

    pub fn write_parent_control(&self, control: &GpuControl) -> Result<()> {
        self.parent_control
            .write()?
            .copy_from_slice(control.as_bytes());
        Ok(())
    }

    pub fn submit(&mut self, queue: &Queue) -> Result<()> {
        self.lock(queue)?;

        let mut builder = SubmitCommandBufferBuilder::new();

        // Safety: the command buffer is kept alive by the wait in drop().
        unsafe {
            builder.add_command_buffer(self.command_buffer.inner());
        }

        // Safety: the fence is reset before self.pending is set to false.
        // Safety: the fence is kept alive by the wait in drop().
        unsafe {
            builder.set_fence_signal(&self.fence);
        }

        builder.submit(queue)?;
        self.pending = true;

        Ok(())
    }

    pub fn wait(&mut self) -> Result<()> {
        assert!(self.pending);

        self.fence.wait(None)?;
        self.fence.reset()?;
        self.pending = false;

        self.unlock();
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if self.pending {
            self.wait()
        } else if self.locked {
            Ok(self.unlock())
        } else {
            Ok(())
        }
    }

    fn lock(&mut self, queue: &Queue) -> Result<()> {
        assert!(!self.locked && !self.pending);
        let future = sync::now(queue.device().clone());
        self.command_buffer.lock_submit(&future, queue)?;
        self.locked = true;
        Ok(())
    }

    fn unlock(&mut self) {
        assert!(self.locked && !self.pending);
        // Safety: self.locked ensures this is called once for each lock().
        unsafe {
            self.command_buffer.unlock();
        }
        self.locked = false;
    }
}

impl Drop for GpuTask {
    fn drop(&mut self) {
        if self.pending {
            // Do not drop the resources held by this struct while in use by the GPU.
            let _ = self.fence.wait(None);
            self.pending = false;
        }

        if self.locked {
            self.unlock();
        }
    }
}

pub fn gpu_init(
    tasks: usize,
    steps: &[GpuStep],
) -> Result<Option<(Vec<Arc<Queue>>, Vec<GpuTask>)>> {
    assert!(!steps.is_empty());

    let instance = Instance::new(
        Some(&app_info_from_cargo_toml!()),
        &InstanceExtensions::none(),
        None,
    )
    .context("Error creating Vulkan instance")?;

    let queue_family = PhysicalDevice::enumerate(&instance)
        .flat_map(|physical_device| {
            // Prefer the queue family with the most compute queues.
            physical_device
                .queue_families()
                .filter(QueueFamily::supports_compute)
                .max_by_key(QueueFamily::queues_count)
        })
        .min_by_key(|queue_family| match queue_family.physical_device().ty() {
            // When both are available, prefer integrated GPU.
            // This avoids waking up the discrete GPU.
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::DiscreteGpu => 2,
            PhysicalDeviceType::VirtualGpu => 3,
            PhysicalDeviceType::Cpu => 4,
            _ => 5,
        });
    let queue_family = match queue_family {
        Some(queue_family) => queue_family,
        None => return Ok(None),
    };
    let physical_device = queue_family.physical_device();

    let extensions = DeviceExtensions {
        khr_get_memory_requirements2: true,
        khr_dedicated_allocation: true,
        ..DeviceExtensions::none()
    };
    let extensions =
        DeviceExtensions::supported_by_device(physical_device).intersection(&extensions);

    let queue_families = (0..tasks.min(queue_family.queues_count())).map(|_| (queue_family, 0.5));

    let (device, queues) = Device::new(
        physical_device,
        &Features::none(),
        &extensions,
        queue_families,
    )
    .context("Error creating Vulkan device")?;
    let queues = queues.collect();

    let pipelines = gpu_init_pipelines(&device)?;
    let buffers = gpu_init_buffers(&device, tasks, steps)?;
    let command_buffers =
        gpu_init_command_buffers(&device, queue_family, tasks, steps, &pipelines, &buffers)?;

    Ok(Some((queues, gpu_tasks(command_buffers, buffers, device)?)))
}

fn gpu_tasks(
    command_buffers: Vec<AutoCommandBuffer>,
    buffers: GpuBuffers,
    device: Arc<Device>,
) -> Result<Vec<GpuTask>> {
    let mut tasks = Vec::with_capacity(command_buffers.len());

    let mut input_buffers = buffers.input_buffers.into_iter();
    let mut output_buffers = buffers.output_buffers.into_iter();

    let mut chunk_control_buffers = buffers.control_buffers_chunk_staging.into_iter();
    let mut parent_control_buffers = buffers.control_buffers_parent_staging.into_iter();

    for command_buffer in command_buffers {
        let fence = Fence::alloc(device.clone())?;

        let input_buffer = input_buffers.next().unwrap();
        let output_buffer = output_buffers.next().unwrap();

        let chunk_control = chunk_control_buffers.next().unwrap();
        let parent_control = parent_control_buffers.next().unwrap();

        tasks.push(GpuTask {
            locked: false,
            pending: false,
            command_buffer,
            fence,
            input_buffer,
            output_buffer,
            chunk_control,
            parent_control,
        });
    }

    Ok(tasks)
}

struct GpuPipelines {
    blake3_chunk: Arc<ComputePipeline<PipelineLayout<shaders::blake3::Layout>>>,
    blake3_parent: Arc<ComputePipeline<PipelineLayout<shaders::blake3::Layout>>>,
}

fn gpu_init_pipelines(device: &Arc<Device>) -> Result<GpuPipelines> {
    let blake3_chunk_shader = shaders::blake3::Shader::load_chunk(device.clone())?;
    let blake3_chunk_pipeline =
        ComputePipeline::new(device.clone(), &blake3_chunk_shader.main_entry_point(), &())?;
    let blake3_chunk = Arc::new(blake3_chunk_pipeline);

    let blake3_parent_shader = shaders::blake3::Shader::load_parent(device.clone())?;
    let blake3_parent_pipeline = ComputePipeline::new(
        device.clone(),
        &blake3_parent_shader.main_entry_point(),
        &(),
    )?;
    let blake3_parent = Arc::new(blake3_parent_pipeline);

    Ok(GpuPipelines {
        blake3_chunk,
        blake3_parent,
    })
}

struct GpuBuffers {
    input_buffers: Vec<Arc<CpuAccessibleBuffer<[u8]>>>,
    output_buffers: Vec<Arc<CpuAccessibleBuffer<[u8]>>>,

    task_buffers: Vec<Vec<Arc<dyn TypedBufferAccess<Content = [u8]> + Send + Sync>>>,

    control_buffers_chunk: Vec<Arc<DeviceLocalBuffer<[u8]>>>,
    control_buffers_parent: Vec<Arc<DeviceLocalBuffer<[u8]>>>,

    control_buffers_chunk_staging: Vec<Arc<CpuAccessibleBuffer<[u8]>>>,
    control_buffers_parent_staging: Vec<Arc<CpuAccessibleBuffer<[u8]>>>,
}

fn gpu_init_buffers(device: &Arc<Device>, tasks: usize, steps: &[GpuStep]) -> Result<GpuBuffers> {
    // These buffers are created and allocated in order of decreasing size.

    let mut buffer_sizes = Vec::with_capacity(steps.len() + 1);
    buffer_sizes.push(steps[0].input_buffer_size());
    for (i, step) in steps.iter().enumerate() {
        assert_eq!(buffer_sizes[i], step.input_buffer_size());
        buffer_sizes.push(step.output_buffer_size());
    }
    debug_assert_eq!(buffer_sizes.len(), steps.len() + 1);

    let mut input_buffers = Vec::with_capacity(tasks);
    let mut output_buffers = Vec::with_capacity(tasks);

    let mut task_buffers = (0..tasks)
        .map(|_| Vec::with_capacity(steps.len() + 1))
        .collect::<Vec<Vec<Arc<dyn TypedBufferAccess<Content = [u8]> + Send + Sync>>>>();

    let input_buffer_size = buffer_sizes[0];
    for buffers in &mut task_buffers {
        let buffer = create_host_cached_storage_buffer(device.clone(), input_buffer_size)?;
        input_buffers.push(buffer.clone());
        buffers.push(buffer);
    }

    for buffer_size in &buffer_sizes[1..buffer_sizes.len() - 1] {
        for buffers in &mut task_buffers {
            buffers.push(create_device_storage_buffer(device.clone(), *buffer_size)?);
        }
    }

    let output_buffer_size = buffer_sizes[buffer_sizes.len() - 1];
    for buffers in &mut task_buffers {
        let buffer = create_host_cached_storage_buffer(device.clone(), output_buffer_size)?;
        output_buffers.push(buffer.clone());
        buffers.push(buffer);
    }

    debug_assert_eq!(input_buffers.len(), tasks);
    debug_assert_eq!(output_buffers.len(), tasks);

    for buffers in &task_buffers {
        debug_assert_eq!(buffers.len(), steps.len() + 1);
    }

    let uniform_size = blake3::gpu::shaders::blake3::CONTROL_UNIFORM_SIZE;

    let control_buffers_chunk = (0..tasks)
        .map(|_| create_device_uniform_buffer(device.clone(), uniform_size))
        .collect::<Result<_>>()?;
    let control_buffers_parent = (0..tasks)
        .map(|_| create_device_uniform_buffer(device.clone(), uniform_size))
        .collect::<Result<_>>()?;

    let control_buffers_chunk_staging = (0..tasks)
        .map(|_| create_transfer_source_buffer(device.clone(), uniform_size))
        .collect::<Result<_>>()?;
    let control_buffers_parent_staging = (0..tasks)
        .map(|_| create_transfer_source_buffer(device.clone(), uniform_size))
        .collect::<Result<_>>()?;

    Ok(GpuBuffers {
        input_buffers,
        output_buffers,
        task_buffers,
        control_buffers_chunk,
        control_buffers_parent,
        control_buffers_chunk_staging,
        control_buffers_parent_staging,
    })
}

fn create_device_storage_buffer(
    device: Arc<Device>,
    size: usize,
) -> Result<Arc<DeviceLocalBuffer<[u8]>>> {
    Ok(
        DeviceLocalBuffer::array(device, size, storage_buffer(), iter::empty())
            .with_context(|| format!("Error creating device local buffer with size {}", size))?,
    )
}

fn create_device_uniform_buffer(
    device: Arc<Device>,
    size: usize,
) -> Result<Arc<DeviceLocalBuffer<[u8]>>> {
    Ok(DeviceLocalBuffer::array(
        device,
        size,
        BufferUsage::uniform_buffer_transfer_destination(),
        iter::empty(),
    )
    .with_context(|| format!("Error creating device local buffer with size {}", size))?)
}

fn create_host_cached_storage_buffer(
    device: Arc<Device>,
    size: usize,
) -> Result<Arc<CpuAccessibleBuffer<[u8]>>> {
    Ok(unsafe {
        CpuAccessibleBuffer::uninitialized_array(device, size, storage_buffer(), true)
            .with_context(|| format!("Error creating host visible buffer with size {}", size))?
    })
}

fn create_transfer_source_buffer(
    device: Arc<Device>,
    size: usize,
) -> Result<Arc<CpuAccessibleBuffer<[u8]>>> {
    Ok(unsafe {
        CpuAccessibleBuffer::uninitialized_array(
            device,
            size,
            BufferUsage::transfer_source(),
            false,
        )
        .with_context(|| format!("Error creating host visible buffer with size {}", size))?
    })
}

#[inline]
fn storage_buffer() -> BufferUsage {
    BufferUsage {
        storage_buffer: true,
        ..BufferUsage::none()
    }
}

fn gpu_init_command_buffers(
    device: &Arc<Device>,
    queue_family: QueueFamily,
    tasks: usize,
    steps: &[GpuStep],
    pipelines: &GpuPipelines,
    buffers: &GpuBuffers,
) -> Result<Vec<AutoCommandBuffer>> {
    let mut command_buffers = Vec::with_capacity(tasks);

    for task in 0..tasks {
        let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue_family)?;

        builder = builder.copy_buffer(
            buffers.control_buffers_chunk_staging[task].clone(),
            buffers.control_buffers_chunk[task].clone(),
        )?;
        builder = builder.copy_buffer(
            buffers.control_buffers_parent_staging[task].clone(),
            buffers.control_buffers_parent[task].clone(),
        )?;

        for (i, step) in steps.iter().enumerate() {
            match *step {
                GpuStep::Blake3Chunk(group_count) => {
                    let pipeline = &pipelines.blake3_chunk;
                    let layout = pipeline.descriptor_set_layout(0).unwrap();
                    let descriptor_set = PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(buffers.task_buffers[task][i].clone())?
                        .add_buffer(buffers.task_buffers[task][i + 1].clone())?
                        .add_buffer(buffers.control_buffers_chunk[task].clone())?
                        .build()?;
                    builder = builder.dispatch(
                        [group_count, 1, 1],
                        pipeline.clone(),
                        descriptor_set,
                        (),
                    )?;
                }

                GpuStep::Blake3Parent(group_count) => {
                    let pipeline = &pipelines.blake3_parent;
                    let layout = pipeline.descriptor_set_layout(0).unwrap();
                    let descriptor_set = PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(buffers.task_buffers[task][i].clone())?
                        .add_buffer(buffers.task_buffers[task][i + 1].clone())?
                        .add_buffer(buffers.control_buffers_parent[task].clone())?
                        .build()?;
                    builder = builder.dispatch(
                        [group_count, 1, 1],
                        pipeline.clone(),
                        descriptor_set,
                        (),
                    )?;
                }
            }
        }

        command_buffers.push(builder.build()?);
    }

    Ok(command_buffers)
}

#[derive(Debug)]
pub enum GpuStep {
    Blake3Chunk(u32),
    Blake3Parent(u32),
}

impl GpuStep {
    #[inline]
    pub fn group_count(&self) -> u32 {
        match *self {
            Self::Blake3Chunk(group_count) | Self::Blake3Parent(group_count) => group_count,
        }
    }

    #[inline]
    pub fn workgroup_size(&self) -> u32 {
        match *self {
            Self::Blake3Chunk(_) | Self::Blake3Parent(_) => {
                blake3::gpu::shaders::blake3::WORKGROUP_SIZE
            }
        }
    }

    #[inline]
    pub fn invocation_count(&self) -> usize {
        self.group_count() as usize * self.workgroup_size() as usize
    }

    #[inline]
    pub fn invocation_input_size(&self) -> usize {
        match *self {
            Self::Blake3Chunk(_) => CHUNK_LEN,
            Self::Blake3Parent(_) => BLOCK_LEN,
        }
    }

    #[inline]
    pub fn invocation_output_size(&self) -> usize {
        match *self {
            Self::Blake3Chunk(_) | Self::Blake3Parent(_) => OUT_LEN,
        }
    }

    #[inline]
    pub fn input_buffer_size(&self) -> usize {
        self.invocation_count() * self.invocation_input_size()
    }

    #[inline]
    pub fn output_buffer_size(&self) -> usize {
        self.invocation_count() * self.invocation_output_size()
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
    fn simulate_blake3_chunk_shader() -> Result<()> {
        let steps = [GpuStep::Blake3Chunk(8)];
        let (queues, mut tasks) = gpu_init(1, &steps)?.expect("No GPU found");

        let hasher = GpuHasher::new();
        let chunk_control = hasher.gpu_control(0);
        let parent_control = hasher.gpu_control_parent();

        let data = selftest_seq(steps[0].input_buffer_size());
        unsafe {
            let mut buf = tasks[0].lock_input_buffer()?;
            buf.copy_from_slice(&data);
        }
        tasks[0].write_chunk_control(&chunk_control)?;
        tasks[0].write_parent_control(&parent_control)?;
        tasks[0].submit(&queues[0])?;

        let mut expected = vec![0; steps[0].output_buffer_size()];
        hasher.simulate_chunk_shader::<RayonJoin>(
            steps[0].invocation_count(),
            &data,
            &mut expected,
            &chunk_control,
        );
        GpuHasher::swap_endian::<RayonJoin>(&mut expected);

        tasks[0].wait()?;
        unsafe {
            assert_eq!(&*tasks[0].lock_output_buffer()?, &*expected);
        }

        Ok(())
    }

    #[test]
    fn simulate_blake3_parent_shader() -> Result<()> {
        let steps = [GpuStep::Blake3Parent(128)];
        let (queues, mut tasks) = gpu_init(1, &steps)?.expect("No GPU found");

        let hasher = GpuHasher::new();
        let chunk_control = hasher.gpu_control(0);
        let parent_control = hasher.gpu_control_parent();

        let data = selftest_seq(steps[0].input_buffer_size());
        unsafe {
            let mut buf = tasks[0].lock_input_buffer()?;
            buf.copy_from_slice(&data);
            GpuHasher::swap_endian::<RayonJoin>(&mut buf);
        }
        tasks[0].write_chunk_control(&chunk_control)?;
        tasks[0].write_parent_control(&parent_control)?;
        tasks[0].submit(&queues[0])?;

        let mut expected = vec![0; steps[0].output_buffer_size()];
        hasher.simulate_parent_shader::<RayonJoin>(
            steps[0].invocation_count(),
            &data,
            &mut expected,
            &parent_control,
        );
        GpuHasher::swap_endian::<RayonJoin>(&mut expected);

        tasks[0].wait()?;
        unsafe {
            assert_eq!(&*tasks[0].lock_output_buffer()?, &*expected);
        }

        Ok(())
    }
}
