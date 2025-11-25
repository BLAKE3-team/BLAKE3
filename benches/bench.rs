//! BLAKE3 Benchmark Suite
//!
//! This module provides comprehensive benchmarks for the BLAKE3 hash function
//! with various optimizations for measurement stability and reliability.
//!
//! For Chinese documentation, see `README_CN.md` in the benches directory.
//!
//! # Stability Optimizations
//!
//! The benchmarks implement several techniques to improve measurement stability:
//!
//! - **Memory Pre-allocation**: Buffers are allocated upfront to avoid
//!   garbage collection and memory allocation during timed sections.
//!
//! - **Cache Warming**: Input data is accessed before benchmarking to
//!   ensure data is in CPU cache, reducing cache miss latency variance.
//!
//! - **Page Alignment Randomization**: Input offsets are randomized
//!   across page boundaries to avoid systematic alignment bias.
//!
//! - **IRQ/Scheduler Mitigation**: Uses test::black_box to prevent
//!   compiler optimizations and help reduce timing variability from interrupts.
//!
//! # Performance Considerations
//!
//! - **Buffer Size**: Larger buffers may cause cache eviction; smaller
//!   buffers may not fully exercise SIMD parallelism.
//!
//! - **Memory Footprint**: Each RandomInput allocates (len + page_size)
//!   bytes to allow offset randomization.
//!
//! - **Interoperability**: Benchmarks are compatible with multiple
//!   SIMD platforms (SSE2, SSE4.1, AVX2, AVX-512, NEON, WASM SIMD).

#![feature(test)]

extern crate test;

use arrayref::array_ref;
use arrayvec::ArrayVec;
use blake3::platform::{Platform, MAX_SIMD_DEGREE};
use blake3::OUT_LEN;
use blake3::{BLOCK_LEN, CHUNK_LEN};
use rand::prelude::*;
use test::Bencher;

// =============================================================================
// Constants
// =============================================================================

/// One kibibyte (1024 bytes), used as a size unit for benchmarks.
const KIB: usize = 1024;

/// Number of warm-up iterations to stabilize cache and reduce timing variance.
///
/// # Rationale
/// Warm-up iterations help ensure:
/// - CPU caches are populated with relevant data
/// - Branch predictors are trained
/// - Memory pages are faulted in
const WARMUP_ITERATIONS: usize = 3;

// =============================================================================
// Input Generation
// =============================================================================

/// A structure for generating randomized benchmark inputs with page-aligned offsets.
///
/// # Stability Features
///
/// This struct implements several techniques to improve benchmark stability:
///
/// - **Pre-allocation**: All memory is allocated during construction,
///   avoiding allocation during benchmark iterations (reduces GC pressure).
///
/// - **Offset Randomization**: Input data can start at any offset
///   within a page, which helps avoid systematic cache line alignment effects.
///
/// - **Cache Warming**: The buffer is accessed during construction,
///   ensuring data is in cache before benchmark iterations begin.
///
/// # Memory Layout
///
/// ```text
/// |<-- page_size -->|<-- len -->|
/// +------------------+-----------+
/// |   offset space   |   data    |
/// +------------------+-----------+
/// ^                  ^
/// buf start          actual data (offset varies)
/// ```
pub struct RandomInput {
    /// Pre-allocated buffer containing random data.
    ///
    /// Size: len + page_size bytes to allow offset randomization.
    buf: Vec<u8>,

    /// The desired length of each input slice.
    len: usize,

    /// Pre-shuffled offsets for page-aligned randomization.
    ///
    /// Each offset is in range [0, page_size), ensuring the returned slice
    /// stays within the allocated buffer.
    offsets: Vec<usize>,

    /// Current index into the offsets array (cycles through all offsets).
    offset_index: usize,
}

impl RandomInput {
    /// Creates a new RandomInput with pre-allocated and pre-warmed buffers.
    ///
    /// # Arguments
    /// * `b` - The bencher instance (used to track bytes processed)
    /// * `len` - The length of each input slice to generate
    ///
    /// # Stability Optimizations
    ///
    /// 1. **Single Allocation**: Allocates all memory upfront to avoid
    ///    GC pressure during benchmark iterations.
    ///
    /// 2. **Random Fill**: Buffer is filled with random data, which:
    ///    - Prevents compiler from optimizing away zero-filled reads
    ///    - Ensures realistic input entropy for hashing
    ///
    /// 3. **Offset Pre-computation**: All offsets are shuffled once,
    ///    avoiding RNG calls during benchmark iterations.
    ///
    /// 4. **Cache Warming**: The random fill operation touches all memory,
    ///    ensuring pages are faulted in and data is in cache.
    pub fn new(b: &mut Bencher, len: usize) -> Self {
        b.bytes += len as u64;
        let page_size: usize = page_size::get();

        // Pre-allocate buffer with extra space for offset randomization.
        // This avoids memory allocation during benchmark iterations.
        let mut buf = vec![0u8; len + page_size];

        let mut rng = rand::rng();

        // Fill buffer with random data to:
        // - Warm the cache
        // - Prevent zero-optimization
        // - Provide realistic input entropy
        rng.fill_bytes(&mut buf);

        // Pre-compute shuffled offsets to avoid RNG overhead during iterations.
        let mut offsets: Vec<usize> = (0..page_size).collect();
        offsets.shuffle(&mut rng);

        Self {
            buf,
            len,
            offsets,
            offset_index: 0,
        }
    }

    /// Returns a slice of the input buffer at a randomized page offset.
    ///
    /// # Stability Features
    ///
    /// - **No Allocation**: Returns a reference to pre-allocated data,
    ///   avoiding any memory allocation during benchmark iterations.
    ///
    /// - **Cache Locality**: The buffer is already in cache from
    ///   construction, minimizing cache miss variance.
    ///
    /// - **Offset Cycling**: Cycles through pre-shuffled offsets,
    ///   providing varied alignment without runtime randomization overhead.
    #[inline]
    pub fn get(&mut self) -> &[u8] {
        let offset = self.offsets[self.offset_index];
        self.offset_index += 1;

        // Cycle through offsets to vary page alignment across iterations.
        if self.offset_index >= self.offsets.len() {
            self.offset_index = 0;
        }

        &self.buf[offset..][..self.len]
    }

    /// Performs warm-up iterations to stabilize cache and reduce variance.
    ///
    /// # Arguments
    /// * `iterations` - Number of warm-up accesses
    ///
    /// # Stability Rationale
    ///
    /// Warm-up iterations help:
    /// - Train CPU branch predictors
    /// - Stabilize memory access patterns
    /// - Reduce IRQ/scheduling timing variance
    #[inline]
    pub fn warmup(&mut self, iterations: usize) {
        for _ in 0..iterations {
            // Access data to warm cache; use black_box to prevent optimization.
            let _ = test::black_box(self.get());
        }
    }
}

// =============================================================================
// Single Compression Benchmarks
// =============================================================================

/// Benchmarks a single compression function call on a specific platform.
///
/// # Stability Optimizations
///
/// - **Pre-allocated State**: State array is allocated outside the
///   benchmark loop to avoid allocation overhead.
///
/// - **Warm-up**: Performs warm-up iterations to stabilize cache.
///
/// - **black_box**: Uses test::black_box to prevent dead code elimination
///   and help mitigate timing variance from compiler optimizations.
fn bench_single_compression_fn(b: &mut Bencher, platform: Platform) {
    // Pre-allocate state outside benchmark loop to avoid allocation overhead.
    let mut state = [1u32; 8];
    let mut r = RandomInput::new(b, 64);

    // Warm-up: stabilize cache and reduce initial timing variance.
    r.warmup(WARMUP_ITERATIONS);

    let input = array_ref!(r.get(), 0, 64);

    // Use black_box to prevent compiler from optimizing away the benchmark.
    b.iter(|| {
        platform.compress_in_place(&mut state, input, 64 as u8, 0, 0);
        test::black_box(&state);
    });
}

/// Benchmark: Single compression using portable (non-SIMD) implementation.
#[bench]
fn bench_single_compression_portable(b: &mut Bencher) {
    bench_single_compression_fn(b, Platform::portable());
}

/// Benchmark: Single compression using SSE2 SIMD instructions (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_single_compression_fn(b, platform);
    }
}

/// Benchmark: Single compression using SSE4.1 SIMD instructions (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_single_compression_fn(b, platform);
    }
}

/// Benchmark: Single compression using AVX-512 SIMD instructions (x86/x86_64).
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_single_compression_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_single_compression_fn(b, platform);
    }
}

// =============================================================================
// Multi-Chunk Hashing Benchmarks
// =============================================================================

/// Benchmarks parallel chunk hashing using SIMD on a specific platform.
///
/// # Stability Optimizations
///
/// - **Pre-allocated Inputs**: All input buffers are allocated before
///   benchmarking begins to avoid allocator overhead during iterations.
///
/// - **Pre-allocated Output**: Output buffer is allocated with fixed
///   capacity to avoid runtime allocation.
///
/// - **Cache Warming**: Warm-up iterations ensure data is in cache.
///
/// # Performance Notes
///
/// The SIMD degree determines how many chunks are hashed in parallel:
/// - SSE2/SSE4.1: 4 chunks
/// - AVX2: 8 chunks
/// - AVX-512: 16 chunks
/// - NEON/WASM: 4 chunks
fn bench_many_chunks_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();

    // Pre-allocate all input buffers to avoid allocation during benchmarking.
    let mut inputs = Vec::with_capacity(degree);
    for _ in 0..degree {
        let mut input = RandomInput::new(b, CHUNK_LEN);
        // Warm-up each input to ensure cache population.
        input.warmup(WARMUP_ITERATIONS);
        inputs.push(input);
    }

    // Pre-allocate output buffer to avoid allocation in hot path.
    let mut out = [0u8; MAX_SIMD_DEGREE * OUT_LEN];

    b.iter(|| {
        let input_arrays: ArrayVec<&[u8; CHUNK_LEN], MAX_SIMD_DEGREE> = inputs
            .iter_mut()
            .take(degree)
            .map(|i| array_ref!(i.get(), 0, CHUNK_LEN))
            .collect();

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

        // Prevent dead code elimination
        test::black_box(&out);
    });
}

/// Benchmark: Multi-chunk hashing using SSE2 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using SSE4.1 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using AVX2 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using AVX-512 (x86/x86_64).
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_chunks_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using ARM NEON SIMD.
#[bench]
#[cfg(blake3_neon)]
fn bench_many_chunks_neon(b: &mut Bencher) {
    bench_many_chunks_fn(b, Platform::neon().unwrap());
}

/// Benchmark: Multi-chunk hashing using WebAssembly SIMD.
#[bench]
#[cfg(blake3_wasm32_simd)]
fn bench_many_chunks_wasm(b: &mut Bencher) {
    bench_many_chunks_fn(b, Platform::wasm32_simd().unwrap());
}

// =============================================================================
// Parent Node Hashing Benchmarks
// =============================================================================

// TODO: When we get const generics we can unify this with the chunks code.

/// Benchmarks parallel parent node hashing using SIMD on a specific platform.
///
/// # Stability Optimizations
///
/// Same as bench_many_chunks_fn:
/// - Pre-allocated inputs
/// - Pre-allocated output buffer
/// - Cache warming
/// - black_box for preventing optimization
///
/// # Difference from Chunk Hashing
///
/// Parent node hashing operates on BLOCK_LEN (64 bytes) instead of CHUNK_LEN (1024 bytes).
fn bench_many_parents_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();

    // Pre-allocate all input buffers.
    let mut inputs = Vec::with_capacity(degree);
    for _ in 0..degree {
        let mut input = RandomInput::new(b, BLOCK_LEN);
        input.warmup(WARMUP_ITERATIONS);
        inputs.push(input);
    }

    // Pre-allocate output buffer.
    let mut out = [0u8; MAX_SIMD_DEGREE * OUT_LEN];

    b.iter(|| {
        let input_arrays: ArrayVec<&[u8; BLOCK_LEN], MAX_SIMD_DEGREE> = inputs
            .iter_mut()
            .take(degree)
            .map(|i| array_ref!(i.get(), 0, BLOCK_LEN))
            .collect();

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

        // Prevent dead code elimination
        test::black_box(&out);
    });
}

/// Benchmark: Parent node hashing using SSE2 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using SSE4.1 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using AVX2 (x86/x86_64).
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using AVX-512 (x86/x86_64).
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_parents_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using ARM NEON SIMD.
#[bench]
#[cfg(blake3_neon)]
fn bench_many_parents_neon(b: &mut Bencher) {
    bench_many_parents_fn(b, Platform::neon().unwrap());
}

/// Benchmark: Parent node hashing using WebAssembly SIMD.
#[bench]
#[cfg(blake3_wasm32_simd)]
fn bench_many_parents_wasm(b: &mut Bencher) {
    bench_many_parents_fn(b, Platform::wasm32_simd().unwrap());
}

// =============================================================================
// All-at-Once Hashing Benchmarks
// =============================================================================

/// Benchmarks the blake3::hash() function with various input sizes.
///
/// # Stability Optimizations
///
/// - **Cache Warming**: Input data is pre-warmed in cache.
///
/// - **black_box**: Prevents compiler from optimizing away hash computation.
///
/// # Performance Notes
///
/// This measures the optimal case where all input is available at once.
/// For incremental hashing, see bench_incremental.
fn bench_atonce(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let hash = blake3::hash(input.get());
        test::black_box(hash)
    });
}

/// Benchmark: All-at-once hashing of 1 block (64 bytes).
#[bench]
fn bench_atonce_0001_block(b: &mut Bencher) {
    bench_atonce(b, BLOCK_LEN);
}

/// Benchmark: All-at-once hashing of 1 KiB.
#[bench]
fn bench_atonce_0001_kib(b: &mut Bencher) {
    bench_atonce(b, 1 * KIB);
}

/// Benchmark: All-at-once hashing of 2 KiB.
#[bench]
fn bench_atonce_0002_kib(b: &mut Bencher) {
    bench_atonce(b, 2 * KIB);
}

/// Benchmark: All-at-once hashing of 4 KiB.
#[bench]
fn bench_atonce_0004_kib(b: &mut Bencher) {
    bench_atonce(b, 4 * KIB);
}

/// Benchmark: All-at-once hashing of 8 KiB.
#[bench]
fn bench_atonce_0008_kib(b: &mut Bencher) {
    bench_atonce(b, 8 * KIB);
}

/// Benchmark: All-at-once hashing of 16 KiB.
#[bench]
fn bench_atonce_0016_kib(b: &mut Bencher) {
    bench_atonce(b, 16 * KIB);
}

/// Benchmark: All-at-once hashing of 32 KiB.
#[bench]
fn bench_atonce_0032_kib(b: &mut Bencher) {
    bench_atonce(b, 32 * KIB);
}

/// Benchmark: All-at-once hashing of 64 KiB.
#[bench]
fn bench_atonce_0064_kib(b: &mut Bencher) {
    bench_atonce(b, 64 * KIB);
}

/// Benchmark: All-at-once hashing of 128 KiB.
#[bench]
fn bench_atonce_0128_kib(b: &mut Bencher) {
    bench_atonce(b, 128 * KIB);
}

/// Benchmark: All-at-once hashing of 256 KiB.
#[bench]
fn bench_atonce_0256_kib(b: &mut Bencher) {
    bench_atonce(b, 256 * KIB);
}

/// Benchmark: All-at-once hashing of 512 KiB.
#[bench]
fn bench_atonce_0512_kib(b: &mut Bencher) {
    bench_atonce(b, 512 * KIB);
}

/// Benchmark: All-at-once hashing of 1024 KiB (1 MiB).
#[bench]
fn bench_atonce_1024_kib(b: &mut Bencher) {
    bench_atonce(b, 1024 * KIB);
}

// =============================================================================
// Incremental Hashing Benchmarks
// =============================================================================

/// Benchmarks incremental hashing using Hasher::new(), update(), and finalize().
///
/// # Stability Optimizations
///
/// - **Cache Warming**: Input data is pre-warmed.
///
/// - **black_box**: Prevents optimization of hash computation.
///
/// # Memory Footprint
///
/// Each benchmark iteration creates a new Hasher instance. The Hasher's internal
/// state is small (< 1KB), so this has minimal impact on benchmark stability.
fn bench_incremental(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let hash = blake3::Hasher::new().update(input.get()).finalize();
        test::black_box(hash)
    });
}

/// Benchmark: Incremental hashing of 1 block (64 bytes).
#[bench]
fn bench_incremental_0001_block(b: &mut Bencher) {
    bench_incremental(b, BLOCK_LEN);
}

/// Benchmark: Incremental hashing of 1 KiB.
#[bench]
fn bench_incremental_0001_kib(b: &mut Bencher) {
    bench_incremental(b, 1 * KIB);
}

/// Benchmark: Incremental hashing of 2 KiB.
#[bench]
fn bench_incremental_0002_kib(b: &mut Bencher) {
    bench_incremental(b, 2 * KIB);
}

/// Benchmark: Incremental hashing of 4 KiB.
#[bench]
fn bench_incremental_0004_kib(b: &mut Bencher) {
    bench_incremental(b, 4 * KIB);
}

/// Benchmark: Incremental hashing of 8 KiB.
#[bench]
fn bench_incremental_0008_kib(b: &mut Bencher) {
    bench_incremental(b, 8 * KIB);
}

/// Benchmark: Incremental hashing of 16 KiB.
#[bench]
fn bench_incremental_0016_kib(b: &mut Bencher) {
    bench_incremental(b, 16 * KIB);
}

/// Benchmark: Incremental hashing of 32 KiB.
#[bench]
fn bench_incremental_0032_kib(b: &mut Bencher) {
    bench_incremental(b, 32 * KIB);
}

/// Benchmark: Incremental hashing of 64 KiB.
#[bench]
fn bench_incremental_0064_kib(b: &mut Bencher) {
    bench_incremental(b, 64 * KIB);
}

/// Benchmark: Incremental hashing of 128 KiB.
#[bench]
fn bench_incremental_0128_kib(b: &mut Bencher) {
    bench_incremental(b, 128 * KIB);
}

/// Benchmark: Incremental hashing of 256 KiB.
#[bench]
fn bench_incremental_0256_kib(b: &mut Bencher) {
    bench_incremental(b, 256 * KIB);
}

/// Benchmark: Incremental hashing of 512 KiB.
#[bench]
fn bench_incremental_0512_kib(b: &mut Bencher) {
    bench_incremental(b, 512 * KIB);
}

/// Benchmark: Incremental hashing of 1024 KiB (1 MiB).
#[bench]
fn bench_incremental_1024_kib(b: &mut Bencher) {
    bench_incremental(b, 1024 * KIB);
}

// =============================================================================
// Reference Implementation Benchmarks
// =============================================================================

/// Benchmarks the reference implementation for comparison.
///
/// # Purpose
///
/// The reference implementation is intentionally simple and unoptimized.
/// Comparing it against the optimized implementation shows the performance
/// benefit of SIMD and parallel processing.
///
/// # Stability Optimizations
///
/// Same as other benchmarks: cache warming and black_box.
fn bench_reference(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let mut hasher = reference_impl::Hasher::new();
        hasher.update(input.get());
        let mut out = [0; 32];
        hasher.finalize(&mut out);
        test::black_box(out)
    });
}

/// Benchmark: Reference implementation hashing of 1 block (64 bytes).
#[bench]
fn bench_reference_0001_block(b: &mut Bencher) {
    bench_reference(b, BLOCK_LEN);
}

/// Benchmark: Reference implementation hashing of 1 KiB.
#[bench]
fn bench_reference_0001_kib(b: &mut Bencher) {
    bench_reference(b, 1 * KIB);
}

/// Benchmark: Reference implementation hashing of 2 KiB.
#[bench]
fn bench_reference_0002_kib(b: &mut Bencher) {
    bench_reference(b, 2 * KIB);
}

/// Benchmark: Reference implementation hashing of 4 KiB.
#[bench]
fn bench_reference_0004_kib(b: &mut Bencher) {
    bench_reference(b, 4 * KIB);
}

/// Benchmark: Reference implementation hashing of 8 KiB.
#[bench]
fn bench_reference_0008_kib(b: &mut Bencher) {
    bench_reference(b, 8 * KIB);
}

/// Benchmark: Reference implementation hashing of 16 KiB.
#[bench]
fn bench_reference_0016_kib(b: &mut Bencher) {
    bench_reference(b, 16 * KIB);
}

/// Benchmark: Reference implementation hashing of 32 KiB.
#[bench]
fn bench_reference_0032_kib(b: &mut Bencher) {
    bench_reference(b, 32 * KIB);
}

/// Benchmark: Reference implementation hashing of 64 KiB.
#[bench]
fn bench_reference_0064_kib(b: &mut Bencher) {
    bench_reference(b, 64 * KIB);
}

/// Benchmark: Reference implementation hashing of 128 KiB.
#[bench]
fn bench_reference_0128_kib(b: &mut Bencher) {
    bench_reference(b, 128 * KIB);
}

/// Benchmark: Reference implementation hashing of 256 KiB.
#[bench]
fn bench_reference_0256_kib(b: &mut Bencher) {
    bench_reference(b, 256 * KIB);
}

/// Benchmark: Reference implementation hashing of 512 KiB.
#[bench]
fn bench_reference_0512_kib(b: &mut Bencher) {
    bench_reference(b, 512 * KIB);
}

/// Benchmark: Reference implementation hashing of 1024 KiB (1 MiB).
#[bench]
fn bench_reference_1024_kib(b: &mut Bencher) {
    bench_reference(b, 1024 * KIB);
}

// =============================================================================
// Rayon (Multithreaded) Benchmarks
// =============================================================================

/// Benchmarks multithreaded hashing using Rayon.
///
/// # Stability Optimizations
///
/// - **Cache Warming**: Input data is pre-warmed.
///
/// - **Thread Pool**: Rayon's thread pool is warmed up during initial
///   iterations, reducing first-call overhead variance.
///
/// # Performance Notes
///
/// Multithreading has overhead. For small inputs (< 128 KiB on x86_64),
/// single-threaded hashing may be faster. Benchmark your specific use case.
#[cfg(feature = "rayon")]
fn bench_rayon(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let hash = blake3::Hasher::new().update_rayon(input.get()).finalize();
        test::black_box(hash)
    });
}

/// Benchmark: Rayon multithreaded hashing of 1 block (64 bytes).
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_block(b: &mut Bencher) {
    bench_rayon(b, BLOCK_LEN);
}

/// Benchmark: Rayon multithreaded hashing of 1 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_kib(b: &mut Bencher) {
    bench_rayon(b, 1 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 2 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0002_kib(b: &mut Bencher) {
    bench_rayon(b, 2 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 4 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0004_kib(b: &mut Bencher) {
    bench_rayon(b, 4 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 8 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0008_kib(b: &mut Bencher) {
    bench_rayon(b, 8 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 16 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0016_kib(b: &mut Bencher) {
    bench_rayon(b, 16 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 32 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0032_kib(b: &mut Bencher) {
    bench_rayon(b, 32 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 64 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0064_kib(b: &mut Bencher) {
    bench_rayon(b, 64 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 128 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0128_kib(b: &mut Bencher) {
    bench_rayon(b, 128 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 256 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0256_kib(b: &mut Bencher) {
    bench_rayon(b, 256 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 512 KiB.
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0512_kib(b: &mut Bencher) {
    bench_rayon(b, 512 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 1024 KiB (1 MiB).
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_1024_kib(b: &mut Bencher) {
    bench_rayon(b, 1024 * KIB);
}

// =============================================================================
// Two-Update Parallelism Recovery Benchmark
// =============================================================================

/// Benchmark: Tests parallelism recovery after an odd-sized initial update.
///
/// # Purpose
///
/// This checks that update() splits up its input in increasing powers of 2, so
/// that it can recover a high degree of parallelism when the number of bytes
/// hashed so far is uneven.
///
/// # Expected Performance
///
/// The performance of this benchmark should be reasonably close to
/// bench_incremental_0064_kib, within 80% or so.
///
/// # History
///
/// When we had a bug in this logic (https://github.com/BLAKE3-team/BLAKE3/issues/69),
/// performance was less than half.
///
/// # Stability Optimizations
///
/// Same as other benchmarks: cache warming and black_box.
#[bench]
fn bench_two_updates(b: &mut Bencher) {
    let len = 65536;
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let mut hasher = blake3::Hasher::new();
        let input_data = input.get();

        // First update with 1 byte, then the rest.
        // This tests the parallelism recovery mechanism.
        hasher.update(&input_data[..1]);
        hasher.update(&input_data[1..]);

        let hash = hasher.finalize();
        test::black_box(hash)
    });
}

// =============================================================================
// Extended Output (XOF) Benchmarks
// =============================================================================

/// Benchmarks the Extended Output Function (XOF) for various output sizes.
///
/// # Stability Optimizations
///
/// - **Pre-allocated Output**: Output buffer is allocated once,
///   avoiding allocation during benchmark iterations.
///
/// - **XOF Reuse**: The OutputReader is created once and reused,
///   which tests the fill() method's performance directly.
///
/// # Performance Notes
///
/// XOF performance scales with output size. For optimal performance when
/// reading in a loop, use a buffer size that's a multiple of BLOCK_LEN (64 bytes).
fn bench_xof(b: &mut Bencher, len: usize) {
    b.bytes = len as u64;

    // Pre-allocate output buffer to avoid allocation in hot path.
    let mut output = [0u8; 64 * BLOCK_LEN];
    let output_slice = &mut output[..len];

    let mut xof = blake3::Hasher::new().finalize_xof();

    b.iter(|| {
        xof.fill(output_slice);
        test::black_box(&output_slice);
    });
}

/// Benchmark: XOF output of 1 block (64 bytes).
#[bench]
fn bench_xof_01_block(b: &mut Bencher) {
    bench_xof(b, 1 * BLOCK_LEN);
}

/// Benchmark: XOF output of 2 blocks (128 bytes).
#[bench]
fn bench_xof_02_blocks(b: &mut Bencher) {
    bench_xof(b, 2 * BLOCK_LEN);
}

/// Benchmark: XOF output of 3 blocks (192 bytes).
#[bench]
fn bench_xof_03_blocks(b: &mut Bencher) {
    bench_xof(b, 3 * BLOCK_LEN);
}

/// Benchmark: XOF output of 4 blocks (256 bytes).
#[bench]
fn bench_xof_04_blocks(b: &mut Bencher) {
    bench_xof(b, 4 * BLOCK_LEN);
}

/// Benchmark: XOF output of 5 blocks (320 bytes).
#[bench]
fn bench_xof_05_blocks(b: &mut Bencher) {
    bench_xof(b, 5 * BLOCK_LEN);
}

/// Benchmark: XOF output of 6 blocks (384 bytes).
#[bench]
fn bench_xof_06_blocks(b: &mut Bencher) {
    bench_xof(b, 6 * BLOCK_LEN);
}

/// Benchmark: XOF output of 7 blocks (448 bytes).
#[bench]
fn bench_xof_07_blocks(b: &mut Bencher) {
    bench_xof(b, 7 * BLOCK_LEN);
}

/// Benchmark: XOF output of 8 blocks (512 bytes).
#[bench]
fn bench_xof_08_blocks(b: &mut Bencher) {
    bench_xof(b, 8 * BLOCK_LEN);
}

/// Benchmark: XOF output of 9 blocks (576 bytes).
#[bench]
fn bench_xof_09_blocks(b: &mut Bencher) {
    bench_xof(b, 9 * BLOCK_LEN);
}

/// Benchmark: XOF output of 10 blocks (640 bytes).
#[bench]
fn bench_xof_10_blocks(b: &mut Bencher) {
    bench_xof(b, 10 * BLOCK_LEN);
}

/// Benchmark: XOF output of 11 blocks (704 bytes).
#[bench]
fn bench_xof_11_blocks(b: &mut Bencher) {
    bench_xof(b, 11 * BLOCK_LEN);
}

/// Benchmark: XOF output of 12 blocks (768 bytes).
#[bench]
fn bench_xof_12_blocks(b: &mut Bencher) {
    bench_xof(b, 12 * BLOCK_LEN);
}

/// Benchmark: XOF output of 13 blocks (832 bytes).
#[bench]
fn bench_xof_13_blocks(b: &mut Bencher) {
    bench_xof(b, 13 * BLOCK_LEN);
}

/// Benchmark: XOF output of 14 blocks (896 bytes).
#[bench]
fn bench_xof_14_blocks(b: &mut Bencher) {
    bench_xof(b, 14 * BLOCK_LEN);
}

/// Benchmark: XOF output of 15 blocks (960 bytes).
#[bench]
fn bench_xof_15_blocks(b: &mut Bencher) {
    bench_xof(b, 15 * BLOCK_LEN);
}

/// Benchmark: XOF output of 16 blocks (1024 bytes).
#[bench]
fn bench_xof_16_blocks(b: &mut Bencher) {
    bench_xof(b, 16 * BLOCK_LEN);
}

/// Benchmark: XOF output of 32 blocks (2048 bytes).
#[bench]
fn bench_xof_32_blocks(b: &mut Bencher) {
    bench_xof(b, 32 * BLOCK_LEN);
}

/// Benchmark: XOF output of 64 blocks (4096 bytes).
#[bench]
fn bench_xof_64_blocks(b: &mut Bencher) {
    bench_xof(b, 64 * BLOCK_LEN);
}
