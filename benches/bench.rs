#![feature(test)]

extern crate test;

use arrayref::array_ref;
use arrayvec::ArrayVec;
use blake3::guts::{BLOCK_LEN, CHUNK_LEN};
use blake3::platform::{Platform, MAX_SIMD_DEGREE};
use blake3::OUT_LEN;
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
    alignment_skip: usize,
}

impl RandomInput {
    pub fn new(b: &mut Bencher, len: usize) -> Self {
        Self::new_aligned(b, len, 1)
    }

    pub fn new_aligned(b: &mut Bencher, len: usize, alignment: usize) -> Self {
        b.bytes += len as u64;
        let page_size: usize = page_size::get();
        let mut buf = vec![0u8; len + page_size + alignment];
        let buf_misalign = (buf.as_ptr() as usize) % alignment;
        let alignment_skip = if buf_misalign == 0 {
            0
        } else {
            alignment - buf_misalign
        };
        assert_eq!(0, (buf.as_ptr() as usize + alignment_skip) % alignment);
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);
        let mut offsets = Vec::new();
        for offset in 0..page_size {
            if offset % alignment == 0 {
                offsets.push(offset);
            }
        }
        offsets.shuffle(&mut rng);
        let mut ret = Self {
            buf,
            len,
            offsets,
            offset_index: 0,
            alignment_skip,
        };
        // Be extra careful sure that we're meeting our alignment guarantees.
        for _ in 0..100 {
            assert_eq!(0, ret.get().as_ptr() as usize % alignment);
        }
        ret
    }

    pub fn get(&mut self) -> &[u8] {
        let offset = self.offsets[self.offset_index];
        self.offset_index += 1;
        if self.offset_index >= self.offsets.len() {
            self.offset_index = 0;
        }
        &self.buf[self.alignment_skip + offset..][..self.len]
    }
}

fn bench_single_compression_fn(b: &mut Bencher, platform: Platform) {
    let mut state = [1u32; 8];
    let mut r = RandomInput::new(b, 64);
    let input = array_ref!(r.get(), 0, 64);
    b.iter(|| platform.compress_in_place(&mut state, input, 64 as u8, 0, 0));
}

#[bench]
fn bench_single_compression_portable(b: &mut Bencher) {
    bench_single_compression_fn(b, Platform::portable());
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_single_compression_fn(b, platform);
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_single_compression_fn(b, platform);
    }
}

#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_single_compression_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_single_compression_fn(b, platform);
    }
}

fn bench_kernel_compression_fn(b: &mut Bencher, f: blake3::kernel::CompressionFn) {
    let mut state = [1u32; 8];
    let mut r = RandomInput::new(b, 64);
    let input = array_ref!(r.get(), 0, 64);
    b.iter(|| unsafe { f(&mut state, input, 64, 0, 0) });
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_kernel_compression_sse2(b: &mut Bencher) {
    if !is_x86_feature_detected!("sse2") {
        return;
    }
    bench_kernel_compression_fn(b, blake3::kernel::blake3_sse2_compress);
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_kernel_compression_sse41(b: &mut Bencher) {
    if !is_x86_feature_detected!("sse4.1") {
        return;
    }
    bench_kernel_compression_fn(b, blake3::kernel::blake3_sse41_compress);
}

#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_kernel_compression_avx512(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    bench_kernel_compression_fn(b, blake3::kernel::blake3_avx512_compress);
}

fn bench_many_chunks_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();
    let mut inputs = Vec::new();
    for _ in 0..degree {
        inputs.push(RandomInput::new(b, CHUNK_LEN));
    }
    b.iter(|| {
        let input_arrays: ArrayVec<&[u8; CHUNK_LEN], MAX_SIMD_DEGREE> = inputs
            .iter_mut()
            .take(degree)
            .map(|i| array_ref!(i.get(), 0, CHUNK_LEN))
            .collect();
        let mut out = [0; MAX_SIMD_DEGREE * OUT_LEN];
        platform.hash_many(
            &input_arrays[..],
            &[0; 8],
            0,
            blake3::IncrementCounter::Yes,
            0,
            0,
            0,
            &mut out,
        );
    });
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_chunks_fn(b, platform);
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_chunks_fn(b, platform);
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_chunks_fn(b, platform);
    }
}

#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_chunks_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_chunks_fn(b, platform);
    }
}

#[bench]
#[cfg(feature = "neon")]
fn bench_many_chunks_neon(b: &mut Bencher) {
    if let Some(platform) = Platform::neon() {
        bench_many_chunks_fn(b, platform);
    }
}

#[bench]
fn bench_many_chunks_kernel(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut input = RandomInput::new(b, 16 * CHUNK_LEN);
    let mut out = [blake3::kernel::Words16([0; 16]); 8];
    b.iter(|| unsafe {
        blake3::kernel::chunks16(input.get().try_into().unwrap(), &[0; 8], 0, 0, &mut out);
    });
}

// TODO: When we get const generics we can unify this with the chunks code.
fn bench_many_parents_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();
    let mut inputs = Vec::new();
    for _ in 0..degree {
        inputs.push(RandomInput::new(b, BLOCK_LEN));
    }
    b.iter(|| {
        let input_arrays: ArrayVec<&[u8; BLOCK_LEN], MAX_SIMD_DEGREE> = inputs
            .iter_mut()
            .take(degree)
            .map(|i| array_ref!(i.get(), 0, BLOCK_LEN))
            .collect();
        let mut out = [0; MAX_SIMD_DEGREE * OUT_LEN];
        platform.hash_many(
            &input_arrays[..],
            &[0; 8],
            0,
            blake3::IncrementCounter::No,
            0,
            0,
            0,
            &mut out,
        );
    });
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_parents_fn(b, platform);
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_parents_fn(b, platform);
    }
}

#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_parents_fn(b, platform);
    }
}

#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_parents_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_parents_fn(b, platform);
    }
}

#[bench]
#[cfg(feature = "neon")]
fn bench_many_parents_neon(b: &mut Bencher) {
    if let Some(platform) = Platform::neon() {
        bench_many_parents_fn(b, platform);
    }
}

#[bench]
fn bench_many_parents_kernel(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    use blake3::kernel::Words16;
    let size = 16 * std::mem::size_of::<Words16>();
    let alignment = std::mem::align_of::<Words16>();
    assert_eq!(alignment, 64);
    let mut input = RandomInput::new_aligned(b, size, alignment);
    for _ in 0..100 {
        assert_eq!(0, (input.get().as_ptr() as usize) % alignment);
    }
    let mut output = [blake3::kernel::Words16([0; 16]); 8];
    b.iter(|| unsafe {
        let rand_ptr = input.get().as_ptr();
        let rand_left_children = &*(rand_ptr as *const [Words16; 8]);
        let rand_right_children = &*(rand_ptr.add(size / 2) as *const [Words16; 8]);
        blake3::kernel::parents16(
            &rand_left_children,
            &rand_right_children,
            &[0; 8],
            0,
            &mut output,
        );
    });
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
fn bench_atonce_0001_kib(b: &mut Bencher) {
    bench_atonce(b, 1 * KIB);
}

#[bench]
fn bench_atonce_0002_kib(b: &mut Bencher) {
    bench_atonce(b, 2 * KIB);
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
    let mut state = blake3::Hasher::new();
    b.iter(|| {
        state.update(input.get());
    });
}

#[bench]
fn bench_incremental_0001_block(b: &mut Bencher) {
    bench_incremental(b, BLOCK_LEN);
}

#[bench]
fn bench_incremental_0001_kib(b: &mut Bencher) {
    bench_incremental(b, 1 * KIB);
}

#[bench]
fn bench_incremental_0002_kib(b: &mut Bencher) {
    bench_incremental(b, 2 * KIB);
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
fn bench_reference_0001_kib(b: &mut Bencher) {
    bench_reference(b, 1 * KIB);
}

#[bench]
fn bench_reference_0002_kib(b: &mut Bencher) {
    bench_reference(b, 2 * KIB);
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

#[cfg(feature = "rayon")]
fn bench_rayon(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    b.iter(|| blake3::Hasher::new().update_rayon(input.get()).finalize());
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_block(b: &mut Bencher) {
    bench_rayon(b, BLOCK_LEN);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_kib(b: &mut Bencher) {
    bench_rayon(b, 1 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0002_kib(b: &mut Bencher) {
    bench_rayon(b, 2 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0004_kib(b: &mut Bencher) {
    bench_rayon(b, 4 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0008_kib(b: &mut Bencher) {
    bench_rayon(b, 8 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0016_kib(b: &mut Bencher) {
    bench_rayon(b, 16 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0032_kib(b: &mut Bencher) {
    bench_rayon(b, 32 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0064_kib(b: &mut Bencher) {
    bench_rayon(b, 64 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0128_kib(b: &mut Bencher) {
    bench_rayon(b, 128 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0256_kib(b: &mut Bencher) {
    bench_rayon(b, 256 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0512_kib(b: &mut Bencher) {
    bench_rayon(b, 512 * KIB);
}

#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_1024_kib(b: &mut Bencher) {
    bench_rayon(b, 1024 * KIB);
}

// This checks that update() splits up its input in increasing powers of 2, so
// that it can recover a high degree of parallelism when the number of bytes
// hashed so far is uneven. The performance of this benchmark should be
// reasonably close to bench_incremental_0064_kib, within 80% or so. When we
// had a bug in this logic (https://github.com/BLAKE3-team/BLAKE3/issues/69),
// performance was less than half.
#[bench]
fn bench_two_updates(b: &mut Bencher) {
    let len = 65536;
    let mut input = RandomInput::new(b, len);
    b.iter(|| {
        let mut hasher = blake3::Hasher::new();
        let input = input.get();
        hasher.update(&input[..1]);
        hasher.update(&input[1..]);
        hasher.finalize()
    });
}

#[bench]
fn bench_xof_stream_kernel(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut output = [0; 16 * 64];
    b.bytes = output.len() as u64;
    let message_words = [0; 16];
    let key_words = [0; 8];
    let counter = 0;
    let block_length = 0;
    let flags = 1 | 2 | 16; // CHUNK_START | CHUNK_END | KEYED_HASH
    b.iter(|| unsafe {
        blake3::kernel::xof_stream16(
            &message_words,
            &key_words,
            counter,
            block_length,
            flags,
            &mut output,
        );
    });
    // Double check that this output is reasonable.
    let mut expected = [0; 16 * 64];
    blake3::Hasher::new_keyed(&[0; 32])
        .finalize_xof()
        .fill(&mut expected);
    assert_eq!(expected, output);
}

#[bench]
fn bench_xof_xor_kernel(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut output = [0; 16 * 64];
    b.bytes = output.len() as u64;
    let message_words = [0; 16];
    let key_words = [0; 8];
    let counter = 0;
    let block_length = 0;
    let flags = 1 | 2 | 16; // CHUNK_START | CHUNK_END | KEYED_HASH
    b.iter(|| unsafe {
        blake3::kernel::xof_xor16(
            &message_words,
            &key_words,
            counter,
            block_length,
            flags,
            &mut output,
        );
    });
}

#[bench]
fn bench_chunks16_kernel2(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut input = RandomInput::new(b, 16 * CHUNK_LEN);
    let key_words = [0; 8];
    let counter = 0;
    let flags = 0;
    b.iter(|| unsafe {
        let message_array = &*(input.get().as_ptr() as *const [u8; 16 * CHUNK_LEN]);
        blake3::kernel2::chunks_16(message_array, &key_words, counter, flags);
    });
}

#[bench]
fn bench_xof_kernel2(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut output = [0; 16 * 64];
    b.bytes = output.len() as u64;
    let block_bytes = [0; 64];
    let key_words = [0; 8];
    let counter = 0;
    let block_length = 0;
    let flags = 1 | 2 | 8 | 16; // CHUNK_START | CHUNK_END | ROOT | KEYED_HASH
    b.iter(|| unsafe {
        blake3::kernel2::xof_16(
            &block_bytes,
            &key_words,
            counter,
            block_length,
            flags,
            &mut output,
        );
    });
    // Double check that this output is reasonable.
    let mut expected = [0; 16 * 64];
    blake3::Hasher::new_keyed(&[0; 32])
        .finalize_xof()
        .fill(&mut expected);
    assert_eq!(expected, output);
}

#[bench]
fn bench_xof_xor_kernel2(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    let mut output = [0; 16 * 64];
    b.bytes = output.len() as u64;
    let block_bytes = [0; 64];
    let key_words = [0; 8];
    let counter = 0;
    let block_length = 0;
    let flags = 1 | 2 | 8 | 16; // CHUNK_START | CHUNK_END | ROOT | KEYED_HASH
    b.iter(|| unsafe {
        blake3::kernel2::xof_xor_16(
            &block_bytes,
            &key_words,
            counter,
            block_length,
            flags,
            &mut output,
        );
    });
}

#[bench]
fn bench_just_kernel2(b: &mut Bencher) {
    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
        return;
    }
    b.bytes = 16 * 64;
    b.iter(|| unsafe {
        blake3::kernel2::just_kernel2();
    });
}
