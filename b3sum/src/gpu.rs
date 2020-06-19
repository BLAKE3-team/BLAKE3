use anyhow::Result;
use blake3::gpu::GpuHasher;
use blake3::join::RayonJoin;
use blake3::{OutputReader, CHUNK_LEN};
use std::collections::VecDeque;
use std::fs::File;

use super::vulkan::{GpuInstance, GpuTask};

const BUFFER_SIZE: usize = 32 * 1024 * 1024;
const TASKS: usize = 3;

pub struct Gpu {
    disabled: bool,
    instance: Option<GpuInstance>,
}

impl Gpu {
    #[inline]
    pub fn new() -> Self {
        Gpu {
            disabled: false,
            instance: None,
        }
    }

    pub fn maybe_hash(
        &mut self,
        base_hasher: &GpuHasher,
        file: &File,
    ) -> Result<Option<OutputReader>> {
        if self.disabled {
            // No GPU found.
            return Ok(None);
        }

        let metadata = file.metadata()?;
        let file_size = metadata.len();
        Ok(if !metadata.is_file() {
            // Not a real file.
            None
        } else if file_size > isize::max_value() as u64 {
            // Too long to safely map.
            // https://github.com/danburkert/memmap-rs/issues/69
            None
        } else if file_size < 4 * BUFFER_SIZE as u64 {
            // Too small to be worth the overhead.
            None
        } else if let Some(ref mut instance) = &mut self.instance {
            // Device already initialized.
            Some(hash_file(instance, base_hasher, file, file_size)?)
        } else if let Some(instance) = GpuInstance::new(TASKS, BUFFER_SIZE)? {
            // Device not yet initialized.
            Some(hash_file(
                self.instance.get_or_insert(instance),
                base_hasher,
                file,
                file_size,
            )?)
        } else {
            // No GPU found.
            self.disabled = true;
            None
        })
    }
}

fn hash_file(
    instance: &mut GpuInstance,
    base_hasher: &GpuHasher,
    file: &File,
    file_size: u64,
) -> Result<OutputReader> {
    let map = unsafe {
        memmap::MmapOptions::new()
            .len(file_size as usize)
            .map(&file)?
    };
    hash(instance, base_hasher, &map)
}

fn hash(
    instance: &mut GpuInstance,
    base_hasher: &GpuHasher,
    mut data: &[u8],
) -> Result<OutputReader> {
    let mut hasher = base_hasher.clone();
    let mut chunk_counter = 0;

    let buffer_size = instance.input_buffer_size();
    let mut tasks = instance.tasks();
    let mut tasks: VecDeque<&mut GpuTask> = tasks.iter_mut().collect();

    let chunk_count = (buffer_size / CHUNK_LEN) as u64;

    let mut tail = None;
    while let Some(task) = tasks.pop_front() {
        let wait_result = if task.is_pending() {
            task.wait()?
        } else {
            Default::default()
        };

        if tail.is_none() {
            if data.len() < buffer_size {
                tail = Some(data);
            } else {
                stream(task.input_buffer(), &data[..buffer_size]);
                data = &data[buffer_size..];
            }
        }

        if tail.is_none() || wait_result.has_more {
            task.submit(&hasher.gpu_control(chunk_counter), tail.is_none())?;
            chunk_counter += chunk_count;
        }

        if wait_result.has_output {
            hasher.update_from_gpu::<RayonJoin>(chunk_count, task.output_buffer());
        }

        if tail.is_none() || wait_result.has_more {
            tasks.push_back(task);
        }
    }

    if let Some(data) = tail {
        hasher.update_with_join::<RayonJoin>(data);
    }

    Ok(hasher.finalize_xof())
}

#[cfg(target_arch = "x86")]
#[target_feature(enable = "sse2")]
unsafe fn stream_sse2(output: &mut [u8], input: &[u8]) {
    use std::arch::x86::*;

    assert_eq!(output.len(), input.len());
    assert_eq!(output.len() % (16 * 8), 0);

    let count = output.len() / 16;
    let input = input.as_ptr() as *const __m128i;
    let output = output.as_mut_ptr() as *mut __m128i;

    _mm_prefetch(input.wrapping_add(0) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(4) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(8) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(12) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(16) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(20) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(24) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(28) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(32) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(36) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(40) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(44) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(48) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(52) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(56) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(60) as *const _, _MM_HINT_NTA);

    for i in (0..count).step_by(8) {
        let xmm0 = _mm_load_si128(input.add(i));
        let xmm1 = _mm_load_si128(input.add(i + 1));
        let xmm2 = _mm_load_si128(input.add(i + 2));
        let xmm3 = _mm_load_si128(input.add(i + 3));
        let xmm4 = _mm_load_si128(input.add(i + 4));
        let xmm5 = _mm_load_si128(input.add(i + 5));
        let xmm6 = _mm_load_si128(input.add(i + 6));
        let xmm7 = _mm_load_si128(input.add(i + 7));

        _mm_prefetch(input.wrapping_add(i + 64) as *const _, _MM_HINT_NTA);
        _mm_prefetch(input.wrapping_add(i + 68) as *const _, _MM_HINT_NTA);

        _mm_stream_si128(output.add(i), xmm0);
        _mm_stream_si128(output.add(i + 1), xmm1);
        _mm_stream_si128(output.add(i + 2), xmm2);
        _mm_stream_si128(output.add(i + 3), xmm3);
        _mm_stream_si128(output.add(i + 4), xmm4);
        _mm_stream_si128(output.add(i + 5), xmm5);
        _mm_stream_si128(output.add(i + 6), xmm6);
        _mm_stream_si128(output.add(i + 7), xmm7);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn stream_sse2(output: &mut [u8], input: &[u8]) {
    use std::arch::x86_64::*;

    assert_eq!(output.len(), input.len());
    assert_eq!(output.len() % (16 * 16), 0);

    let count = output.len() / 16;
    let input = input.as_ptr() as *const __m128i;
    let output = output.as_mut_ptr() as *mut __m128i;

    _mm_prefetch(input.wrapping_add(0) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(4) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(8) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(12) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(16) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(20) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(24) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(28) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(32) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(36) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(40) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(44) as *const _, _MM_HINT_NTA);

    _mm_prefetch(input.wrapping_add(48) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(52) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(56) as *const _, _MM_HINT_NTA);
    _mm_prefetch(input.wrapping_add(60) as *const _, _MM_HINT_NTA);

    for i in (0..count).step_by(16) {
        let xmm0 = _mm_load_si128(input.add(i));
        let xmm1 = _mm_load_si128(input.add(i + 1));
        let xmm2 = _mm_load_si128(input.add(i + 2));
        let xmm3 = _mm_load_si128(input.add(i + 3));
        let xmm4 = _mm_load_si128(input.add(i + 4));
        let xmm5 = _mm_load_si128(input.add(i + 5));
        let xmm6 = _mm_load_si128(input.add(i + 6));
        let xmm7 = _mm_load_si128(input.add(i + 7));
        let xmm8 = _mm_load_si128(input.add(i + 8));
        let xmm9 = _mm_load_si128(input.add(i + 9));
        let xmm10 = _mm_load_si128(input.add(i + 10));
        let xmm11 = _mm_load_si128(input.add(i + 11));
        let xmm12 = _mm_load_si128(input.add(i + 12));
        let xmm13 = _mm_load_si128(input.add(i + 13));
        let xmm14 = _mm_load_si128(input.add(i + 14));
        let xmm15 = _mm_load_si128(input.add(i + 15));

        _mm_prefetch(input.wrapping_add(i + 64) as *const _, _MM_HINT_NTA);
        _mm_prefetch(input.wrapping_add(i + 68) as *const _, _MM_HINT_NTA);
        _mm_prefetch(input.wrapping_add(i + 72) as *const _, _MM_HINT_NTA);
        _mm_prefetch(input.wrapping_add(i + 76) as *const _, _MM_HINT_NTA);

        _mm_stream_si128(output.add(i), xmm0);
        _mm_stream_si128(output.add(i + 1), xmm1);
        _mm_stream_si128(output.add(i + 2), xmm2);
        _mm_stream_si128(output.add(i + 3), xmm3);
        _mm_stream_si128(output.add(i + 4), xmm4);
        _mm_stream_si128(output.add(i + 5), xmm5);
        _mm_stream_si128(output.add(i + 6), xmm6);
        _mm_stream_si128(output.add(i + 7), xmm7);
        _mm_stream_si128(output.add(i + 8), xmm8);
        _mm_stream_si128(output.add(i + 9), xmm9);
        _mm_stream_si128(output.add(i + 10), xmm10);
        _mm_stream_si128(output.add(i + 11), xmm11);
        _mm_stream_si128(output.add(i + 12), xmm12);
        _mm_stream_si128(output.add(i + 13), xmm13);
        _mm_stream_si128(output.add(i + 14), xmm14);
        _mm_stream_si128(output.add(i + 15), xmm15);
    }
}

fn stream(output: &mut [u8], input: &[u8]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        if is_x86_feature_detected!("sse2") {
            return stream_sse2(output, input);
        }
    }

    output.copy_from_slice(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use blake3::gpu::{shaders, GpuHasher};
    use blake3::join::RayonJoin;
    use blake3::OUT_LEN;

    // Should be big enough for at least 3 steps.
    const TEST_BUFFER_SIZE: usize = 4 * shaders::blake3::WORKGROUP_SIZE * CHUNK_LEN;

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
    fn task_sequence() -> Result<()> {
        let mut instance = GpuInstance::new(3, TEST_BUFFER_SIZE)?.expect("No GPU found");

        let mut test = |data: &[u8]| -> Result<()> {
            let mut hasher = GpuHasher::new();

            let mut output = hash(&mut instance, &hasher, data)?;
            let mut hash = [0; OUT_LEN];
            output.fill(&mut hash);

            hasher.update_with_join::<RayonJoin>(&data);
            let expected = hasher.finalize();

            assert_eq!(&hash, expected.as_bytes());
            Ok(())
        };

        let data = selftest_seq(16 * TEST_BUFFER_SIZE + 16 + 1);
        for count in 0..=16 {
            // No partial buffers
            test(&data[..count * TEST_BUFFER_SIZE])?;

            // Partial buffer at the end
            test(&data[..count * TEST_BUFFER_SIZE + count + 1])?;
        }
        Ok(())
    }
}
