use anyhow::Result;
use blake3::gpu::GpuHasher;
use blake3::join::RayonJoin;
use blake3::{OutputReader, CHUNK_LEN};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::vulkan::{gpu_init, GpuStep, GpuTask, Queue};

const BUFFER_SIZE: usize = 32 * 1024 * 1024;
const TASKS: usize = 3;

const STEPS: &[GpuStep] = &[
    GpuStep::Blake3Chunk(256),
    GpuStep::Blake3Parent(128),
    GpuStep::Blake3Parent(64),
    GpuStep::Blake3Parent(32),
    GpuStep::Blake3Parent(16),
    GpuStep::Blake3Parent(8),
    GpuStep::Blake3Parent(4),
    GpuStep::Blake3Parent(2),
    GpuStep::Blake3Parent(1),
];

pub struct Gpu {
    disabled: bool,
    inner: Option<GpuState>,
}

impl Gpu {
    #[inline]
    pub fn new() -> Self {
        Gpu {
            disabled: false,
            inner: None,
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
        } else if file_size < 4 * BUFFER_SIZE as u64 {
            // Too small to be worth the overhead.
            None
        } else if let Some(state) = &mut self.inner {
            // Device already initialized.
            Some(state.hash(base_hasher, file)?)
        } else if let Some((queues, tasks)) = gpu_init(TASKS, STEPS)? {
            // Device not yet initialized.
            let state = GpuState::new(queues, tasks);
            Some(self.inner.get_or_insert(state).hash(base_hasher, file)?)
        } else {
            // No GPU found.
            self.disabled = true;
            None
        })
    }
}

struct GpuState {
    queues: Vec<Arc<Queue>>,
    tasks: Vec<GpuTask>,
}

struct GpuTaskRef<'a>(&'a mut GpuTask);

impl<'a> Deref for GpuTaskRef<'a> {
    type Target = GpuTask;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for GpuTaskRef<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Drop for GpuTaskRef<'a> {
    fn drop(&mut self) {
        // An early exit due to an error might leave the GPU task running or
        // locked, reset it to prepare for the next file.
        self.reset().expect("Unable to reset GPU task state")
    }
}

enum GpuTaskState<'a> {
    Full(GpuTaskRef<'a>),
    Tail(GpuTaskRef<'a>, usize),
}

impl GpuState {
    #[inline]
    pub fn new(queues: Vec<Arc<Queue>>, tasks: Vec<GpuTask>) -> Self {
        GpuState { queues, tasks }
    }

    pub fn hash(&mut self, base_hasher: &GpuHasher, file: &File) -> Result<OutputReader> {
        let mut hasher = base_hasher.clone();
        let mut chunk_counter = 0;

        let queues = &self.queues;
        let mut next_queue = 0;

        let mut tasks: VecDeque<_> = self.tasks.iter_mut().map(GpuTaskRef).collect();
        let mut pending = VecDeque::with_capacity(tasks.len());

        let chunk_count = (BUFFER_SIZE / CHUNK_LEN) as u64;
        let mut tail = false;
        loop {
            let mut task = if let Some(task) = tasks.pop_front() {
                task
            } else if let Some(state) = pending.pop_front() {
                match state {
                    GpuTaskState::Full(mut task) => {
                        task.wait()?;
                        {
                            let mut buffer = unsafe { task.lock_output_buffer()? };
                            hasher.update_from_gpu::<RayonJoin>(chunk_count, &mut buffer);
                        }
                        task
                    }
                    GpuTaskState::Tail(task, size) => {
                        debug_assert!(tasks.is_empty() && pending.is_empty() && tail);
                        {
                            let buffer = unsafe { task.lock_input_buffer()? };
                            hasher.update_with_join::<RayonJoin>(&buffer[..size]);
                        }
                        task
                    }
                }
            } else {
                break;
            };

            if !tail {
                task.write_control(&hasher.gpu_control(chunk_counter))?;
                chunk_counter += chunk_count;

                let size = {
                    let mut buffer = unsafe { task.lock_input_buffer()? };
                    debug_assert_eq!(buffer.len(), BUFFER_SIZE);
                    read_all(file, &mut buffer)?
                };

                if size < BUFFER_SIZE {
                    tail = true;
                    tasks.clear();

                    if size > 0 {
                        pending.push_back(GpuTaskState::Tail(task, size));
                    }
                } else {
                    let queue = &queues[next_queue];
                    next_queue = (next_queue + 1) % queues.len();

                    task.submit(queue)?;
                    pending.push_back(GpuTaskState::Full(task));
                }
            }
        }

        Ok(hasher.finalize_xof())
    }
}

fn read_all<R: Read>(mut reader: R, mut buf: &mut [u8]) -> io::Result<usize> {
    let len = buf.len();
    while !buf.is_empty() {
        match reader.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(len - buf.len())
}
