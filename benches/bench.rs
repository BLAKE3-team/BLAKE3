#![feature(test)]

extern crate test;

use arrayref::array_ref;
use arrayvec::ArrayVec;
use blake3::platform::MAX_SIMD_DEGREE;
use blake3::{BLOCK_LEN, CHUNK_LEN, KEY_LEN, OUT_LEN};
use rand::prelude::*;
use test::Bencher;

const KIB: usize = 1024;

// This struct randomizes two things:
// 1. The actual bytes of input.
// 2. The page offset the input starts at.
pub struct RandomInput {
    buf: Vec<u8>,
    len: usize,
    offsets: Vec<usize>,
    offset_index: usize,
}

impl RandomInput {
    pub fn new(b: &mut Bencher, len: usize) -> Self {
        b.bytes += len as u64;
        let page_size: usize = page_size::get();
        let mut buf = vec![0u8; len + page_size];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);
        let mut offsets: Vec<usize> = (0..page_size).collect();
        offsets.shuffle(&mut rng);
        Self {
            buf,
            len,
            offsets,
            offset_index: 0,
        }
    }

    pub fn get(&mut self) -> &[u8] {
        let offset = self.offsets[self.offset_index];
        self.offset_index += 1;
        if self.offset_index >= self.offsets.len() {
            self.offset_index = 0;
        }
        &self.buf[offset..][..self.len]
    }
}

type CompressFn = unsafe fn(
    cv: &[u8; 32],
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    offset: u64,
    flags: u8,
) -> [u8; 64];

fn bench_single_compression_fn(b: &mut Bencher, f: CompressFn) {
    let state: [u8; 32];
    let mut r = RandomInput::new(b, 64);
    state = *array_ref!(r.get(), 0, 32);
    let input = array_ref!(r.get(), 0, 64);
    unsafe {
        b.iter(|| f(&state, input, 64 as u8, 0, 0));
    }
}

#[bench]
fn bench_single_compression_portable(b: &mut Bencher) {
    bench_single_compression_fn(b, blake3::portable::compress);
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse41(b: &mut Bencher) {
    if !blake3::platform::sse41_detected() {
        return;
    }
    bench_single_compression_fn(b, blake3::sse41::compress);
}

type HashManyFn<A> = unsafe fn(
    inputs: &[&A],
    key: &[u8; blake3::KEY_LEN],
    offset: u64,
    offset_deltas: &[u64; 17],
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8],
);

fn bench_many_chunks_fn(b: &mut Bencher, f: HashManyFn<[u8; CHUNK_LEN]>, degree: usize) {
    let mut inputs = Vec::new();
    for _ in 0..degree {
        inputs.push(RandomInput::new(b, CHUNK_LEN));
    }
    unsafe {
        b.iter(|| {
            let input_arrays: ArrayVec<[&[u8; CHUNK_LEN]; MAX_SIMD_DEGREE]> = inputs
                .iter_mut()
                .take(degree)
                .map(|i| array_ref!(i.get(), 0, CHUNK_LEN))
                .collect();
            let mut out = [0; MAX_SIMD_DEGREE * OUT_LEN];
            f(
                &input_arrays[..],
                &[0; KEY_LEN],
                0,
                &[0; 17],
                0,
                0,
                0,
                &mut out,
            );
        });
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse41(b: &mut Bencher) {
    if !blake3::platform::sse41_detected() {
        return;
    }
    bench_many_chunks_fn(b, blake3::sse41::hash_many, blake3::sse41::DEGREE);
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_avx2(b: &mut Bencher) {
    if !blake3::platform::avx2_detected() {
        return;
    }
    bench_many_chunks_fn(b, blake3::avx2::hash_many, blake3::avx2::DEGREE);
}

// TODO: When we get const generics we can unify this with the chunks code.
fn bench_many_parents_fn(b: &mut Bencher, f: HashManyFn<[u8; BLOCK_LEN]>, degree: usize) {
    let mut inputs = Vec::new();
    for _ in 0..degree {
        inputs.push(RandomInput::new(b, BLOCK_LEN));
    }
    unsafe {
        b.iter(|| {
            let input_arrays: ArrayVec<[&[u8; BLOCK_LEN]; MAX_SIMD_DEGREE]> = inputs
                .iter_mut()
                .take(degree)
                .map(|i| array_ref!(i.get(), 0, BLOCK_LEN))
                .collect();
            let mut out = [0; MAX_SIMD_DEGREE * OUT_LEN];
            f(
                &input_arrays[..],
                &[0; KEY_LEN],
                0,
                &[0; 17],
                0,
                0,
                0,
                &mut out,
            );
        });
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse41(b: &mut Bencher) {
    if !blake3::platform::sse41_detected() {
        return;
    }
    bench_many_parents_fn(b, blake3::sse41::hash_many, blake3::sse41::DEGREE);
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_avx2(b: &mut Bencher) {
    if !blake3::platform::avx2_detected() {
        return;
    }
    bench_many_parents_fn(b, blake3::avx2::hash_many, blake3::avx2::DEGREE);
}

fn bench_atonce(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    b.iter(|| blake3::hash(input.get()));
}

#[bench]
fn bench_atonce_0001_block(b: &mut Bencher) {
    bench_atonce(b, BLOCK_LEN);
}

#[bench]
fn bench_atonce_0001_chunk(b: &mut Bencher) {
    bench_atonce(b, CHUNK_LEN);
}

#[bench]
fn bench_atonce_0004_kib(b: &mut Bencher) {
    bench_atonce(b, 4 * KIB);
}

#[bench]
fn bench_atonce_0008_kib(b: &mut Bencher) {
    bench_atonce(b, 8 * KIB);
}

#[bench]
fn bench_atonce_0016_kib(b: &mut Bencher) {
    bench_atonce(b, 16 * KIB);
}

#[bench]
fn bench_atonce_0032_kib(b: &mut Bencher) {
    bench_atonce(b, 32 * KIB);
}

#[bench]
fn bench_atonce_0064_kib(b: &mut Bencher) {
    bench_atonce(b, 64 * KIB);
}

#[bench]
fn bench_atonce_0128_kib(b: &mut Bencher) {
    bench_atonce(b, 128 * KIB);
}

#[bench]
fn bench_atonce_0256_kib(b: &mut Bencher) {
    bench_atonce(b, 256 * KIB);
}

#[bench]
fn bench_atonce_0512_kib(b: &mut Bencher) {
    bench_atonce(b, 512 * KIB);
}

#[bench]
fn bench_atonce_1024_kib(b: &mut Bencher) {
    bench_atonce(b, 1024 * KIB);
}

fn bench_incremental(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    b.iter(|| blake3::Hasher::new().update(input.get()).finalize());
}

#[bench]
fn bench_incremental_0001_block(b: &mut Bencher) {
    bench_incremental(b, BLOCK_LEN);
}

#[bench]
fn bench_incremental_0001_chunk(b: &mut Bencher) {
    bench_incremental(b, CHUNK_LEN);
}

#[bench]
fn bench_incremental_0004_kib(b: &mut Bencher) {
    bench_incremental(b, 4 * KIB);
}

#[bench]
fn bench_incremental_0008_kib(b: &mut Bencher) {
    bench_incremental(b, 8 * KIB);
}

#[bench]
fn bench_incremental_0016_kib(b: &mut Bencher) {
    bench_incremental(b, 16 * KIB);
}

#[bench]
fn bench_incremental_0032_kib(b: &mut Bencher) {
    bench_incremental(b, 32 * KIB);
}

#[bench]
fn bench_incremental_0064_kib(b: &mut Bencher) {
    bench_incremental(b, 64 * KIB);
}

#[bench]
fn bench_incremental_0128_kib(b: &mut Bencher) {
    bench_incremental(b, 128 * KIB);
}

#[bench]
fn bench_incremental_0256_kib(b: &mut Bencher) {
    bench_incremental(b, 256 * KIB);
}

#[bench]
fn bench_incremental_0512_kib(b: &mut Bencher) {
    bench_incremental(b, 512 * KIB);
}

#[bench]
fn bench_incremental_1024_kib(b: &mut Bencher) {
    bench_incremental(b, 1024 * KIB);
}

fn bench_reference(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    b.iter(|| {
        let mut hasher = reference_impl::Hasher::new();
        hasher.update(input.get());
        let mut out = [0; 32];
        hasher.finalize(&mut out);
        out
    });
}

#[bench]
fn bench_reference_0001_block(b: &mut Bencher) {
    bench_reference(b, BLOCK_LEN);
}

#[bench]
fn bench_reference_0001_chunk(b: &mut Bencher) {
    bench_reference(b, CHUNK_LEN);
}

#[bench]
fn bench_reference_0004_kib(b: &mut Bencher) {
    bench_reference(b, 4 * KIB);
}

#[bench]
fn bench_reference_0008_kib(b: &mut Bencher) {
    bench_reference(b, 8 * KIB);
}

#[bench]
fn bench_reference_0016_kib(b: &mut Bencher) {
    bench_reference(b, 16 * KIB);
}

#[bench]
fn bench_reference_0032_kib(b: &mut Bencher) {
    bench_reference(b, 32 * KIB);
}

#[bench]
fn bench_reference_0064_kib(b: &mut Bencher) {
    bench_reference(b, 64 * KIB);
}

#[bench]
fn bench_reference_0128_kib(b: &mut Bencher) {
    bench_reference(b, 128 * KIB);
}

#[bench]
fn bench_reference_0256_kib(b: &mut Bencher) {
    bench_reference(b, 256 * KIB);
}

#[bench]
fn bench_reference_0512_kib(b: &mut Bencher) {
    bench_reference(b, 512 * KIB);
}

#[bench]
fn bench_reference_1024_kib(b: &mut Bencher) {
    bench_reference(b, 1024 * KIB);
}
