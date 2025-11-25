//! BLAKE3 Benchmark Suite
//! BLAKE3 基准测试套件
//!
//! This module provides comprehensive benchmarks for the BLAKE3 hash function
//! with various optimizations for measurement stability and reliability.
//! 本模块提供 BLAKE3 哈希函数的全面基准测试，并针对测量稳定性和可靠性进行了各种优化。
//!
//! # Stability Optimizations / 稳定性优化
//!
//! The benchmarks implement several techniques to improve measurement stability:
//! 基准测试实现了多种技术来提高测量稳定性：
//!
//! - **Memory Pre-allocation / 内存预分配**: Buffers are allocated upfront to avoid
//!   garbage collection and memory allocation during timed sections.
//!   缓冲区预先分配，避免在计时区间内进行垃圾回收和内存分配。
//!
//! - **Cache Warming / 缓存预热**: Input data is accessed before benchmarking to
//!   ensure data is in CPU cache, reducing cache miss latency variance.
//!   在基准测试前访问输入数据，确保数据在 CPU 缓存中，减少缓存未命中延迟变化。
//!
//! - **Page Alignment Randomization / 页对齐随机化**: Input offsets are randomized
//!   across page boundaries to avoid systematic alignment bias.
//!   输入偏移量在页边界之间随机化，避免系统性对齐偏差。
//!
//! - **IRQ/Scheduler Mitigation / 中断/调度器缓解**: Uses test::black_box to prevent
//!   compiler optimizations and help reduce timing variability from interrupts.
//!   使用 test::black_box 防止编译器优化，帮助减少中断引起的计时变化。
//!
//! # Performance Considerations / 性能注意事项
//!
//! - **Buffer Size / 缓冲区大小**: Larger buffers may cause cache eviction; smaller
//!   buffers may not fully exercise SIMD parallelism.
//!   较大的缓冲区可能导致缓存驱逐；较小的缓冲区可能无法充分利用 SIMD 并行性。
//!
//! - **Memory Footprint / 内存占用**: Each RandomInput allocates (len + page_size)
//!   bytes to allow offset randomization.
//!   每个 RandomInput 分配 (len + page_size) 字节以允许偏移量随机化。
//!
//! - **Interoperability / 互操作性**: Benchmarks are compatible with multiple
//!   SIMD platforms (SSE2, SSE4.1, AVX2, AVX-512, NEON, WASM SIMD).
//!   基准测试与多个 SIMD 平台兼容（SSE2、SSE4.1、AVX2、AVX-512、NEON、WASM SIMD）。

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
// Constants / 常量
// =============================================================================

/// One kibibyte (1024 bytes), used as a size unit for benchmarks.
/// 一个千字节（1024 字节），用作基准测试的大小单位。
const KIB: usize = 1024;

/// Number of warm-up iterations to stabilize cache and reduce timing variance.
/// 预热迭代次数，用于稳定缓存并减少计时变化。
///
/// # Rationale / 原理
/// Warm-up iterations help ensure:
/// 预热迭代有助于确保：
/// - CPU caches are populated with relevant data / CPU 缓存填充了相关数据
/// - Branch predictors are trained / 分支预测器已训练
/// - Memory pages are faulted in / 内存页已加载
const WARMUP_ITERATIONS: usize = 3;

// =============================================================================
// Input Generation / 输入生成
// =============================================================================

/// A structure for generating randomized benchmark inputs with page-aligned offsets.
/// 用于生成具有页对齐偏移量的随机化基准测试输入的结构。
///
/// # Stability Features / 稳定性特性
///
/// This struct implements several techniques to improve benchmark stability:
/// 此结构实现了多种技术来提高基准测试稳定性：
///
/// - **Pre-allocation / 预分配**: All memory is allocated during construction,
///   avoiding allocation during benchmark iterations (reduces GC pressure).
///   所有内存在构造时分配，避免在基准测试迭代期间分配（减少 GC 压力）。
///
/// - **Offset Randomization / 偏移量随机化**: Input data can start at any offset
///   within a page, which helps avoid systematic cache line alignment effects.
///   输入数据可以从页内的任何偏移量开始，有助于避免系统性缓存行对齐效应。
///
/// - **Cache Warming / 缓存预热**: The buffer is accessed during construction,
///   ensuring data is in cache before benchmark iterations begin.
///   缓冲区在构造时被访问，确保数据在基准测试迭代开始前已在缓存中。
///
/// # Memory Layout / 内存布局
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
    /// 包含随机数据的预分配缓冲区。
    ///
    /// Size: len + page_size bytes to allow offset randomization.
    /// 大小：len + page_size 字节，以允许偏移量随机化。
    buf: Vec<u8>,

    /// The desired length of each input slice.
    /// 每个输入切片的期望长度。
    len: usize,

    /// Pre-shuffled offsets for page-aligned randomization.
    /// 用于页对齐随机化的预洗牌偏移量。
    ///
    /// Each offset is in range [0, page_size), ensuring the returned slice
    /// stays within the allocated buffer.
    /// 每个偏移量在 [0, page_size) 范围内，确保返回的切片保持在分配的缓冲区内。
    offsets: Vec<usize>,

    /// Current index into the offsets array (cycles through all offsets).
    /// 偏移量数组中的当前索引（循环遍历所有偏移量）。
    offset_index: usize,
}

impl RandomInput {
    /// Creates a new RandomInput with pre-allocated and pre-warmed buffers.
    /// 创建一个具有预分配和预热缓冲区的新 RandomInput。
    ///
    /// # Arguments / 参数
    /// * `b` - The bencher instance (used to track bytes processed)
    ///         基准测试器实例（用于跟踪处理的字节数）
    /// * `len` - The length of each input slice to generate
    ///           要生成的每个输入切片的长度
    ///
    /// # Stability Optimizations / 稳定性优化
    ///
    /// 1. **Single Allocation / 单次分配**: Allocates all memory upfront to avoid
    ///    GC pressure during benchmark iterations.
    ///    预先分配所有内存，避免基准测试迭代期间的 GC 压力。
    ///
    /// 2. **Random Fill / 随机填充**: Buffer is filled with random data, which:
    ///    缓冲区填充随机数据，这：
    ///    - Prevents compiler from optimizing away zero-filled reads
    ///      防止编译器优化掉零填充读取
    ///    - Ensures realistic input entropy for hashing
    ///      确保哈希的真实输入熵
    ///
    /// 3. **Offset Pre-computation / 偏移量预计算**: All offsets are shuffled once,
    ///    avoiding RNG calls during benchmark iterations.
    ///    所有偏移量只洗牌一次，避免基准测试迭代期间的 RNG 调用。
    ///
    /// 4. **Cache Warming / 缓存预热**: The random fill operation touches all memory,
    ///    ensuring pages are faulted in and data is in cache.
    ///    随机填充操作触及所有内存，确保页面已加载且数据在缓存中。
    pub fn new(b: &mut Bencher, len: usize) -> Self {
        b.bytes += len as u64;
        let page_size: usize = page_size::get();

        // Pre-allocate buffer with extra space for offset randomization.
        // 预分配缓冲区，留出额外空间用于偏移量随机化。
        // This avoids memory allocation during benchmark iterations.
        // 这避免了基准测试迭代期间的内存分配。
        let mut buf = vec![0u8; len + page_size];

        let mut rng = rand::rng();

        // Fill buffer with random data to:
        // 用随机数据填充缓冲区，以：
        // - Warm the cache / 预热缓存
        // - Prevent zero-optimization / 防止零优化
        // - Provide realistic input entropy / 提供真实的输入熵
        rng.fill_bytes(&mut buf);

        // Pre-compute shuffled offsets to avoid RNG overhead during iterations.
        // 预计算洗牌后的偏移量，避免迭代期间的 RNG 开销。
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
    /// 返回输入缓冲区在随机页偏移量处的切片。
    ///
    /// # Stability Features / 稳定性特性
    ///
    /// - **No Allocation / 无分配**: Returns a reference to pre-allocated data,
    ///   avoiding any memory allocation during benchmark iterations.
    ///   返回对预分配数据的引用，避免基准测试迭代期间的任何内存分配。
    ///
    /// - **Cache Locality / 缓存局部性**: The buffer is already in cache from
    ///   construction, minimizing cache miss variance.
    ///   缓冲区从构造时就已在缓存中，最小化缓存未命中变化。
    ///
    /// - **Offset Cycling / 偏移量循环**: Cycles through pre-shuffled offsets,
    ///   providing varied alignment without runtime randomization overhead.
    ///   循环遍历预洗牌的偏移量，提供多样化的对齐而无运行时随机化开销。
    #[inline]
    pub fn get(&mut self) -> &[u8] {
        let offset = self.offsets[self.offset_index];
        self.offset_index += 1;

        // Cycle through offsets to vary page alignment across iterations.
        // 循环遍历偏移量，在迭代中改变页对齐。
        if self.offset_index >= self.offsets.len() {
            self.offset_index = 0;
        }

        &self.buf[offset..][..self.len]
    }

    /// Performs warm-up iterations to stabilize cache and reduce variance.
    /// 执行预热迭代以稳定缓存并减少变化。
    ///
    /// # Arguments / 参数
    /// * `iterations` - Number of warm-up accesses / 预热访问次数
    ///
    /// # Stability Rationale / 稳定性原理
    ///
    /// Warm-up iterations help:
    /// 预热迭代有助于：
    /// - Train CPU branch predictors / 训练 CPU 分支预测器
    /// - Stabilize memory access patterns / 稳定内存访问模式
    /// - Reduce IRQ/scheduling timing variance / 减少 IRQ/调度计时变化
    #[inline]
    pub fn warmup(&mut self, iterations: usize) {
        for _ in 0..iterations {
            // Access data to warm cache; use black_box to prevent optimization.
            // 访问数据以预热缓存；使用 black_box 防止优化。
            let _ = test::black_box(self.get());
        }
    }
}

// =============================================================================
// Single Compression Benchmarks / 单次压缩基准测试
// =============================================================================

/// Benchmarks a single compression function call on a specific platform.
/// 在特定平台上对单次压缩函数调用进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Pre-allocated State / 预分配状态**: State array is allocated outside the
///   benchmark loop to avoid GC pressure.
///   状态数组在基准测试循环外分配，避免 GC 压力。
///
/// - **Warm-up / 预热**: Performs warm-up iterations to stabilize cache.
///   执行预热迭代以稳定缓存。
///
/// - **black_box / 黑盒**: Uses test::black_box to prevent dead code elimination
///   and help mitigate timing variance from compiler optimizations.
///   使用 test::black_box 防止死代码消除，帮助缓解编译器优化引起的计时变化。
fn bench_single_compression_fn(b: &mut Bencher, platform: Platform) {
    // Pre-allocate state outside benchmark loop to avoid allocation overhead.
    // 在基准测试循环外预分配状态，避免分配开销。
    let mut state = [1u32; 8];
    let mut r = RandomInput::new(b, 64);

    // Warm-up: stabilize cache and reduce initial timing variance.
    // 预热：稳定缓存并减少初始计时变化。
    r.warmup(WARMUP_ITERATIONS);

    let input = array_ref!(r.get(), 0, 64);

    // Use black_box to prevent compiler from optimizing away the benchmark.
    // 使用 black_box 防止编译器优化掉基准测试。
    b.iter(|| {
        platform.compress_in_place(&mut state, input, 64 as u8, 0, 0);
        test::black_box(&state);
    });
}

/// Benchmark: Single compression using portable (non-SIMD) implementation.
/// 基准测试：使用可移植（非 SIMD）实现的单次压缩。
#[bench]
fn bench_single_compression_portable(b: &mut Bencher) {
    bench_single_compression_fn(b, Platform::portable());
}

/// Benchmark: Single compression using SSE2 SIMD instructions (x86/x86_64).
/// 基准测试：使用 SSE2 SIMD 指令（x86/x86_64）的单次压缩。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_single_compression_fn(b, platform);
    }
}

/// Benchmark: Single compression using SSE4.1 SIMD instructions (x86/x86_64).
/// 基准测试：使用 SSE4.1 SIMD 指令（x86/x86_64）的单次压缩。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_single_compression_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_single_compression_fn(b, platform);
    }
}

/// Benchmark: Single compression using AVX-512 SIMD instructions (x86/x86_64).
/// 基准测试：使用 AVX-512 SIMD 指令（x86/x86_64）的单次压缩。
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_single_compression_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_single_compression_fn(b, platform);
    }
}

// =============================================================================
// Multi-Chunk Hashing Benchmarks / 多块哈希基准测试
// =============================================================================

/// Benchmarks parallel chunk hashing using SIMD on a specific platform.
/// 在特定平台上使用 SIMD 对并行块哈希进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Pre-allocated Inputs / 预分配输入**: All input buffers are allocated before
///   benchmarking begins to avoid GC pressure during iterations.
///   所有输入缓冲区在基准测试开始前分配，避免迭代期间的 GC 压力。
///
/// - **Pre-allocated Output / 预分配输出**: Output buffer is allocated with fixed
///   capacity to avoid runtime allocation.
///   输出缓冲区以固定容量分配，避免运行时分配。
///
/// - **Cache Warming / 缓存预热**: Warm-up iterations ensure data is in cache.
///   预热迭代确保数据在缓存中。
///
/// # Performance Notes / 性能说明
///
/// The SIMD degree determines how many chunks are hashed in parallel:
/// SIMD 度数决定了并行哈希多少块：
/// - SSE2/SSE4.1: 4 chunks / 4 块
/// - AVX2: 8 chunks / 8 块
/// - AVX-512: 16 chunks / 16 块
/// - NEON/WASM: 4 chunks / 4 块
fn bench_many_chunks_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();

    // Pre-allocate all input buffers to avoid allocation during benchmarking.
    // 预分配所有输入缓冲区，避免基准测试期间的分配。
    let mut inputs = Vec::with_capacity(degree);
    for _ in 0..degree {
        let mut input = RandomInput::new(b, CHUNK_LEN);
        // Warm-up each input to ensure cache population.
        // 预热每个输入以确保缓存填充。
        input.warmup(WARMUP_ITERATIONS);
        inputs.push(input);
    }

    // Pre-allocate output buffer to avoid allocation in hot path.
    // 预分配输出缓冲区，避免热路径中的分配。
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

        // Prevent dead code elimination / 防止死代码消除
        test::black_box(&out);
    });
}

/// Benchmark: Multi-chunk hashing using SSE2 (x86/x86_64).
/// 基准测试：使用 SSE2 的多块哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using SSE4.1 (x86/x86_64).
/// 基准测试：使用 SSE4.1 的多块哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using AVX2 (x86/x86_64).
/// 基准测试：使用 AVX2 的多块哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_chunks_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using AVX-512 (x86/x86_64).
/// 基准测试：使用 AVX-512 的多块哈希（x86/x86_64）。
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_chunks_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_chunks_fn(b, platform);
    }
}

/// Benchmark: Multi-chunk hashing using ARM NEON SIMD.
/// 基准测试：使用 ARM NEON SIMD 的多块哈希。
#[bench]
#[cfg(blake3_neon)]
fn bench_many_chunks_neon(b: &mut Bencher) {
    bench_many_chunks_fn(b, Platform::neon().unwrap());
}

/// Benchmark: Multi-chunk hashing using WebAssembly SIMD.
/// 基准测试：使用 WebAssembly SIMD 的多块哈希。
#[bench]
#[cfg(blake3_wasm32_simd)]
fn bench_many_chunks_wasm(b: &mut Bencher) {
    bench_many_chunks_fn(b, Platform::wasm32_simd().unwrap());
}

// =============================================================================
// Parent Node Hashing Benchmarks / 父节点哈希基准测试
// =============================================================================

// TODO: When we get const generics we can unify this with the chunks code.
// TODO: 当我们获得 const 泛型时，可以将此与 chunks 代码统一。

/// Benchmarks parallel parent node hashing using SIMD on a specific platform.
/// 在特定平台上使用 SIMD 对并行父节点哈希进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// Same as bench_many_chunks_fn:
/// 与 bench_many_chunks_fn 相同：
/// - Pre-allocated inputs / 预分配输入
/// - Pre-allocated output buffer / 预分配输出缓冲区
/// - Cache warming / 缓存预热
/// - black_box for preventing optimization / 使用 black_box 防止优化
///
/// # Difference from Chunk Hashing / 与块哈希的区别
///
/// Parent node hashing operates on BLOCK_LEN (64 bytes) instead of CHUNK_LEN (1024 bytes).
/// 父节点哈希操作 BLOCK_LEN（64 字节）而不是 CHUNK_LEN（1024 字节）。
fn bench_many_parents_fn(b: &mut Bencher, platform: Platform) {
    let degree = platform.simd_degree();

    // Pre-allocate all input buffers.
    // 预分配所有输入缓冲区。
    let mut inputs = Vec::with_capacity(degree);
    for _ in 0..degree {
        let mut input = RandomInput::new(b, BLOCK_LEN);
        input.warmup(WARMUP_ITERATIONS);
        inputs.push(input);
    }

    // Pre-allocate output buffer.
    // 预分配输出缓冲区。
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

        // Prevent dead code elimination / 防止死代码消除
        test::black_box(&out);
    });
}

/// Benchmark: Parent node hashing using SSE2 (x86/x86_64).
/// 基准测试：使用 SSE2 的父节点哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse2(b: &mut Bencher) {
    if let Some(platform) = Platform::sse2() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using SSE4.1 (x86/x86_64).
/// 基准测试：使用 SSE4.1 的父节点哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_sse41(b: &mut Bencher) {
    if let Some(platform) = Platform::sse41() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using AVX2 (x86/x86_64).
/// 基准测试：使用 AVX2 的父节点哈希（x86/x86_64）。
#[bench]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn bench_many_parents_avx2(b: &mut Bencher) {
    if let Some(platform) = Platform::avx2() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using AVX-512 (x86/x86_64).
/// 基准测试：使用 AVX-512 的父节点哈希（x86/x86_64）。
#[bench]
#[cfg(blake3_avx512_ffi)]
fn bench_many_parents_avx512(b: &mut Bencher) {
    if let Some(platform) = Platform::avx512() {
        bench_many_parents_fn(b, platform);
    }
}

/// Benchmark: Parent node hashing using ARM NEON SIMD.
/// 基准测试：使用 ARM NEON SIMD 的父节点哈希。
#[bench]
#[cfg(blake3_neon)]
fn bench_many_parents_neon(b: &mut Bencher) {
    bench_many_parents_fn(b, Platform::neon().unwrap());
}

/// Benchmark: Parent node hashing using WebAssembly SIMD.
/// 基准测试：使用 WebAssembly SIMD 的父节点哈希。
#[bench]
#[cfg(blake3_wasm32_simd)]
fn bench_many_parents_wasm(b: &mut Bencher) {
    bench_many_parents_fn(b, Platform::wasm32_simd().unwrap());
}

// =============================================================================
// All-at-Once Hashing Benchmarks / 一次性哈希基准测试
// =============================================================================

/// Benchmarks the blake3::hash() function with various input sizes.
/// 使用各种输入大小对 blake3::hash() 函数进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Cache Warming / 缓存预热**: Input data is pre-warmed in cache.
///   输入数据在缓存中预热。
///
/// - **black_box / 黑盒**: Prevents compiler from optimizing away hash computation.
///   防止编译器优化掉哈希计算。
///
/// # Performance Notes / 性能说明
///
/// This measures the optimal case where all input is available at once.
/// For incremental hashing, see bench_incremental.
/// 这测量了所有输入一次性可用的最佳情况。
/// 对于增量哈希，请参见 bench_incremental。
fn bench_atonce(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let hash = blake3::hash(input.get());
        test::black_box(hash)
    });
}

/// Benchmark: All-at-once hashing of 1 block (64 bytes).
/// 基准测试：一次性哈希 1 个块（64 字节）。
#[bench]
fn bench_atonce_0001_block(b: &mut Bencher) {
    bench_atonce(b, BLOCK_LEN);
}

/// Benchmark: All-at-once hashing of 1 KiB.
/// 基准测试：一次性哈希 1 KiB。
#[bench]
fn bench_atonce_0001_kib(b: &mut Bencher) {
    bench_atonce(b, 1 * KIB);
}

/// Benchmark: All-at-once hashing of 2 KiB.
/// 基准测试：一次性哈希 2 KiB。
#[bench]
fn bench_atonce_0002_kib(b: &mut Bencher) {
    bench_atonce(b, 2 * KIB);
}

/// Benchmark: All-at-once hashing of 4 KiB.
/// 基准测试：一次性哈希 4 KiB。
#[bench]
fn bench_atonce_0004_kib(b: &mut Bencher) {
    bench_atonce(b, 4 * KIB);
}

/// Benchmark: All-at-once hashing of 8 KiB.
/// 基准测试：一次性哈希 8 KiB。
#[bench]
fn bench_atonce_0008_kib(b: &mut Bencher) {
    bench_atonce(b, 8 * KIB);
}

/// Benchmark: All-at-once hashing of 16 KiB.
/// 基准测试：一次性哈希 16 KiB。
#[bench]
fn bench_atonce_0016_kib(b: &mut Bencher) {
    bench_atonce(b, 16 * KIB);
}

/// Benchmark: All-at-once hashing of 32 KiB.
/// 基准测试：一次性哈希 32 KiB。
#[bench]
fn bench_atonce_0032_kib(b: &mut Bencher) {
    bench_atonce(b, 32 * KIB);
}

/// Benchmark: All-at-once hashing of 64 KiB.
/// 基准测试：一次性哈希 64 KiB。
#[bench]
fn bench_atonce_0064_kib(b: &mut Bencher) {
    bench_atonce(b, 64 * KIB);
}

/// Benchmark: All-at-once hashing of 128 KiB.
/// 基准测试：一次性哈希 128 KiB。
#[bench]
fn bench_atonce_0128_kib(b: &mut Bencher) {
    bench_atonce(b, 128 * KIB);
}

/// Benchmark: All-at-once hashing of 256 KiB.
/// 基准测试：一次性哈希 256 KiB。
#[bench]
fn bench_atonce_0256_kib(b: &mut Bencher) {
    bench_atonce(b, 256 * KIB);
}

/// Benchmark: All-at-once hashing of 512 KiB.
/// 基准测试：一次性哈希 512 KiB。
#[bench]
fn bench_atonce_0512_kib(b: &mut Bencher) {
    bench_atonce(b, 512 * KIB);
}

/// Benchmark: All-at-once hashing of 1024 KiB (1 MiB).
/// 基准测试：一次性哈希 1024 KiB（1 MiB）。
#[bench]
fn bench_atonce_1024_kib(b: &mut Bencher) {
    bench_atonce(b, 1024 * KIB);
}

// =============================================================================
// Incremental Hashing Benchmarks / 增量哈希基准测试
// =============================================================================

/// Benchmarks incremental hashing using Hasher::new(), update(), and finalize().
/// 使用 Hasher::new()、update() 和 finalize() 对增量哈希进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Cache Warming / 缓存预热**: Input data is pre-warmed.
///   输入数据预热。
///
/// - **black_box / 黑盒**: Prevents optimization of hash computation.
///   防止哈希计算的优化。
///
/// # Memory Footprint / 内存占用
///
/// Each benchmark iteration creates a new Hasher instance. The Hasher's internal
/// state is small (< 1KB), so this has minimal impact on benchmark stability.
/// 每次基准测试迭代创建一个新的 Hasher 实例。Hasher 的内部状态很小（< 1KB），
/// 因此对基准测试稳定性的影响最小。
fn bench_incremental(b: &mut Bencher, len: usize) {
    let mut input = RandomInput::new(b, len);
    input.warmup(WARMUP_ITERATIONS);

    b.iter(|| {
        let hash = blake3::Hasher::new().update(input.get()).finalize();
        test::black_box(hash)
    });
}

/// Benchmark: Incremental hashing of 1 block (64 bytes).
/// 基准测试：增量哈希 1 个块（64 字节）。
#[bench]
fn bench_incremental_0001_block(b: &mut Bencher) {
    bench_incremental(b, BLOCK_LEN);
}

/// Benchmark: Incremental hashing of 1 KiB.
/// 基准测试：增量哈希 1 KiB。
#[bench]
fn bench_incremental_0001_kib(b: &mut Bencher) {
    bench_incremental(b, 1 * KIB);
}

/// Benchmark: Incremental hashing of 2 KiB.
/// 基准测试：增量哈希 2 KiB。
#[bench]
fn bench_incremental_0002_kib(b: &mut Bencher) {
    bench_incremental(b, 2 * KIB);
}

/// Benchmark: Incremental hashing of 4 KiB.
/// 基准测试：增量哈希 4 KiB。
#[bench]
fn bench_incremental_0004_kib(b: &mut Bencher) {
    bench_incremental(b, 4 * KIB);
}

/// Benchmark: Incremental hashing of 8 KiB.
/// 基准测试：增量哈希 8 KiB。
#[bench]
fn bench_incremental_0008_kib(b: &mut Bencher) {
    bench_incremental(b, 8 * KIB);
}

/// Benchmark: Incremental hashing of 16 KiB.
/// 基准测试：增量哈希 16 KiB。
#[bench]
fn bench_incremental_0016_kib(b: &mut Bencher) {
    bench_incremental(b, 16 * KIB);
}

/// Benchmark: Incremental hashing of 32 KiB.
/// 基准测试：增量哈希 32 KiB。
#[bench]
fn bench_incremental_0032_kib(b: &mut Bencher) {
    bench_incremental(b, 32 * KIB);
}

/// Benchmark: Incremental hashing of 64 KiB.
/// 基准测试：增量哈希 64 KiB。
#[bench]
fn bench_incremental_0064_kib(b: &mut Bencher) {
    bench_incremental(b, 64 * KIB);
}

/// Benchmark: Incremental hashing of 128 KiB.
/// 基准测试：增量哈希 128 KiB。
#[bench]
fn bench_incremental_0128_kib(b: &mut Bencher) {
    bench_incremental(b, 128 * KIB);
}

/// Benchmark: Incremental hashing of 256 KiB.
/// 基准测试：增量哈希 256 KiB。
#[bench]
fn bench_incremental_0256_kib(b: &mut Bencher) {
    bench_incremental(b, 256 * KIB);
}

/// Benchmark: Incremental hashing of 512 KiB.
/// 基准测试：增量哈希 512 KiB。
#[bench]
fn bench_incremental_0512_kib(b: &mut Bencher) {
    bench_incremental(b, 512 * KIB);
}

/// Benchmark: Incremental hashing of 1024 KiB (1 MiB).
/// 基准测试：增量哈希 1024 KiB（1 MiB）。
#[bench]
fn bench_incremental_1024_kib(b: &mut Bencher) {
    bench_incremental(b, 1024 * KIB);
}

// =============================================================================
// Reference Implementation Benchmarks / 参考实现基准测试
// =============================================================================

/// Benchmarks the reference implementation for comparison.
/// 对参考实现进行基准测试以进行比较。
///
/// # Purpose / 目的
///
/// The reference implementation is intentionally simple and unoptimized.
/// Comparing it against the optimized implementation shows the performance
/// benefit of SIMD and parallel processing.
/// 参考实现故意简单且未优化。
/// 将其与优化实现进行比较，显示 SIMD 和并行处理的性能优势。
///
/// # Stability Optimizations / 稳定性优化
///
/// Same as other benchmarks: cache warming and black_box.
/// 与其他基准测试相同：缓存预热和 black_box。
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
/// 基准测试：参考实现哈希 1 个块（64 字节）。
#[bench]
fn bench_reference_0001_block(b: &mut Bencher) {
    bench_reference(b, BLOCK_LEN);
}

/// Benchmark: Reference implementation hashing of 1 KiB.
/// 基准测试：参考实现哈希 1 KiB。
#[bench]
fn bench_reference_0001_kib(b: &mut Bencher) {
    bench_reference(b, 1 * KIB);
}

/// Benchmark: Reference implementation hashing of 2 KiB.
/// 基准测试：参考实现哈希 2 KiB。
#[bench]
fn bench_reference_0002_kib(b: &mut Bencher) {
    bench_reference(b, 2 * KIB);
}

/// Benchmark: Reference implementation hashing of 4 KiB.
/// 基准测试：参考实现哈希 4 KiB。
#[bench]
fn bench_reference_0004_kib(b: &mut Bencher) {
    bench_reference(b, 4 * KIB);
}

/// Benchmark: Reference implementation hashing of 8 KiB.
/// 基准测试：参考实现哈希 8 KiB。
#[bench]
fn bench_reference_0008_kib(b: &mut Bencher) {
    bench_reference(b, 8 * KIB);
}

/// Benchmark: Reference implementation hashing of 16 KiB.
/// 基准测试：参考实现哈希 16 KiB。
#[bench]
fn bench_reference_0016_kib(b: &mut Bencher) {
    bench_reference(b, 16 * KIB);
}

/// Benchmark: Reference implementation hashing of 32 KiB.
/// 基准测试：参考实现哈希 32 KiB。
#[bench]
fn bench_reference_0032_kib(b: &mut Bencher) {
    bench_reference(b, 32 * KIB);
}

/// Benchmark: Reference implementation hashing of 64 KiB.
/// 基准测试：参考实现哈希 64 KiB。
#[bench]
fn bench_reference_0064_kib(b: &mut Bencher) {
    bench_reference(b, 64 * KIB);
}

/// Benchmark: Reference implementation hashing of 128 KiB.
/// 基准测试：参考实现哈希 128 KiB。
#[bench]
fn bench_reference_0128_kib(b: &mut Bencher) {
    bench_reference(b, 128 * KIB);
}

/// Benchmark: Reference implementation hashing of 256 KiB.
/// 基准测试：参考实现哈希 256 KiB。
#[bench]
fn bench_reference_0256_kib(b: &mut Bencher) {
    bench_reference(b, 256 * KIB);
}

/// Benchmark: Reference implementation hashing of 512 KiB.
/// 基准测试：参考实现哈希 512 KiB。
#[bench]
fn bench_reference_0512_kib(b: &mut Bencher) {
    bench_reference(b, 512 * KIB);
}

/// Benchmark: Reference implementation hashing of 1024 KiB (1 MiB).
/// 基准测试：参考实现哈希 1024 KiB（1 MiB）。
#[bench]
fn bench_reference_1024_kib(b: &mut Bencher) {
    bench_reference(b, 1024 * KIB);
}

// =============================================================================
// Rayon (Multithreaded) Benchmarks / Rayon（多线程）基准测试
// =============================================================================

/// Benchmarks multithreaded hashing using Rayon.
/// 使用 Rayon 对多线程哈希进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Cache Warming / 缓存预热**: Input data is pre-warmed.
///   输入数据预热。
///
/// - **Thread Pool / 线程池**: Rayon's thread pool is warmed up during initial
///   iterations, reducing first-call overhead variance.
///   Rayon 的线程池在初始迭代期间预热，减少首次调用开销变化。
///
/// # Performance Notes / 性能说明
///
/// Multithreading has overhead. For small inputs (< 128 KiB on x86_64),
/// single-threaded hashing may be faster. Benchmark your specific use case.
/// 多线程有开销。对于小输入（x86_64 上 < 128 KiB），单线程哈希可能更快。
/// 请针对您的特定用例进行基准测试。
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
/// 基准测试：Rayon 多线程哈希 1 个块（64 字节）。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_block(b: &mut Bencher) {
    bench_rayon(b, BLOCK_LEN);
}

/// Benchmark: Rayon multithreaded hashing of 1 KiB.
/// 基准测试：Rayon 多线程哈希 1 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0001_kib(b: &mut Bencher) {
    bench_rayon(b, 1 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 2 KiB.
/// 基准测试：Rayon 多线程哈希 2 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0002_kib(b: &mut Bencher) {
    bench_rayon(b, 2 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 4 KiB.
/// 基准测试：Rayon 多线程哈希 4 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0004_kib(b: &mut Bencher) {
    bench_rayon(b, 4 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 8 KiB.
/// 基准测试：Rayon 多线程哈希 8 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0008_kib(b: &mut Bencher) {
    bench_rayon(b, 8 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 16 KiB.
/// 基准测试：Rayon 多线程哈希 16 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0016_kib(b: &mut Bencher) {
    bench_rayon(b, 16 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 32 KiB.
/// 基准测试：Rayon 多线程哈希 32 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0032_kib(b: &mut Bencher) {
    bench_rayon(b, 32 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 64 KiB.
/// 基准测试：Rayon 多线程哈希 64 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0064_kib(b: &mut Bencher) {
    bench_rayon(b, 64 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 128 KiB.
/// 基准测试：Rayon 多线程哈希 128 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0128_kib(b: &mut Bencher) {
    bench_rayon(b, 128 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 256 KiB.
/// 基准测试：Rayon 多线程哈希 256 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0256_kib(b: &mut Bencher) {
    bench_rayon(b, 256 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 512 KiB.
/// 基准测试：Rayon 多线程哈希 512 KiB。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_0512_kib(b: &mut Bencher) {
    bench_rayon(b, 512 * KIB);
}

/// Benchmark: Rayon multithreaded hashing of 1024 KiB (1 MiB).
/// 基准测试：Rayon 多线程哈希 1024 KiB（1 MiB）。
#[bench]
#[cfg(feature = "rayon")]
fn bench_rayon_1024_kib(b: &mut Bencher) {
    bench_rayon(b, 1024 * KIB);
}

// =============================================================================
// Two-Update Parallelism Recovery Benchmark / 双更新并行恢复基准测试
// =============================================================================

/// Benchmark: Tests parallelism recovery after an odd-sized initial update.
/// 基准测试：测试奇数大小初始更新后的并行恢复。
///
/// # Purpose / 目的
///
/// This checks that update() splits up its input in increasing powers of 2, so
/// that it can recover a high degree of parallelism when the number of bytes
/// hashed so far is uneven.
/// 这检查 update() 是否以 2 的幂次方分割其输入，以便在到目前为止哈希的字节数不均匀时
/// 能够恢复高度并行性。
///
/// # Expected Performance / 预期性能
///
/// The performance of this benchmark should be reasonably close to
/// bench_incremental_0064_kib, within 80% or so.
/// 此基准测试的性能应该与 bench_incremental_0064_kib 相当接近，大约在 80% 以内。
///
/// # History / 历史
///
/// When we had a bug in this logic (https://github.com/BLAKE3-team/BLAKE3/issues/69),
/// performance was less than half.
/// 当我们在此逻辑中有一个错误时（https://github.com/BLAKE3-team/BLAKE3/issues/69），
/// 性能不到一半。
///
/// # Stability Optimizations / 稳定性优化
///
/// Same as other benchmarks: cache warming and black_box.
/// 与其他基准测试相同：缓存预热和 black_box。
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
        // 第一次更新 1 字节，然后更新其余部分。
        // 这测试并行恢复机制。
        hasher.update(&input_data[..1]);
        hasher.update(&input_data[1..]);

        let hash = hasher.finalize();
        test::black_box(hash)
    });
}

// =============================================================================
// Extended Output (XOF) Benchmarks / 扩展输出（XOF）基准测试
// =============================================================================

/// Benchmarks the Extended Output Function (XOF) for various output sizes.
/// 对各种输出大小的扩展输出函数（XOF）进行基准测试。
///
/// # Stability Optimizations / 稳定性优化
///
/// - **Pre-allocated Output / 预分配输出**: Output buffer is allocated once,
///   avoiding allocation during benchmark iterations.
///   输出缓冲区只分配一次，避免基准测试迭代期间的分配。
///
/// - **XOF Reuse / XOF 重用**: The OutputReader is created once and reused,
///   which tests the fill() method's performance directly.
///   OutputReader 创建一次并重用，直接测试 fill() 方法的性能。
///
/// # Performance Notes / 性能说明
///
/// XOF performance scales with output size. For optimal performance when
/// reading in a loop, use a buffer size that's a multiple of BLOCK_LEN (64 bytes).
/// XOF 性能与输出大小成比例。为了在循环中获得最佳性能，
/// 请使用 BLOCK_LEN（64 字节）倍数的缓冲区大小。
fn bench_xof(b: &mut Bencher, len: usize) {
    b.bytes = len as u64;

    // Pre-allocate output buffer to avoid allocation in hot path.
    // 预分配输出缓冲区，避免热路径中的分配。
    let mut output = [0u8; 64 * BLOCK_LEN];
    let output_slice = &mut output[..len];

    let mut xof = blake3::Hasher::new().finalize_xof();

    b.iter(|| {
        xof.fill(output_slice);
        test::black_box(&output_slice);
    });
}

/// Benchmark: XOF output of 1 block (64 bytes).
/// 基准测试：XOF 输出 1 个块（64 字节）。
#[bench]
fn bench_xof_01_block(b: &mut Bencher) {
    bench_xof(b, 1 * BLOCK_LEN);
}

/// Benchmark: XOF output of 2 blocks (128 bytes).
/// 基准测试：XOF 输出 2 个块（128 字节）。
#[bench]
fn bench_xof_02_blocks(b: &mut Bencher) {
    bench_xof(b, 2 * BLOCK_LEN);
}

/// Benchmark: XOF output of 3 blocks (192 bytes).
/// 基准测试：XOF 输出 3 个块（192 字节）。
#[bench]
fn bench_xof_03_blocks(b: &mut Bencher) {
    bench_xof(b, 3 * BLOCK_LEN);
}

/// Benchmark: XOF output of 4 blocks (256 bytes).
/// 基准测试：XOF 输出 4 个块（256 字节）。
#[bench]
fn bench_xof_04_blocks(b: &mut Bencher) {
    bench_xof(b, 4 * BLOCK_LEN);
}

/// Benchmark: XOF output of 5 blocks (320 bytes).
/// 基准测试：XOF 输出 5 个块（320 字节）。
#[bench]
fn bench_xof_05_blocks(b: &mut Bencher) {
    bench_xof(b, 5 * BLOCK_LEN);
}

/// Benchmark: XOF output of 6 blocks (384 bytes).
/// 基准测试：XOF 输出 6 个块（384 字节）。
#[bench]
fn bench_xof_06_blocks(b: &mut Bencher) {
    bench_xof(b, 6 * BLOCK_LEN);
}

/// Benchmark: XOF output of 7 blocks (448 bytes).
/// 基准测试：XOF 输出 7 个块（448 字节）。
#[bench]
fn bench_xof_07_blocks(b: &mut Bencher) {
    bench_xof(b, 7 * BLOCK_LEN);
}

/// Benchmark: XOF output of 8 blocks (512 bytes).
/// 基准测试：XOF 输出 8 个块（512 字节）。
#[bench]
fn bench_xof_08_blocks(b: &mut Bencher) {
    bench_xof(b, 8 * BLOCK_LEN);
}

/// Benchmark: XOF output of 9 blocks (576 bytes).
/// 基准测试：XOF 输出 9 个块（576 字节）。
#[bench]
fn bench_xof_09_blocks(b: &mut Bencher) {
    bench_xof(b, 9 * BLOCK_LEN);
}

/// Benchmark: XOF output of 10 blocks (640 bytes).
/// 基准测试：XOF 输出 10 个块（640 字节）。
#[bench]
fn bench_xof_10_blocks(b: &mut Bencher) {
    bench_xof(b, 10 * BLOCK_LEN);
}

/// Benchmark: XOF output of 11 blocks (704 bytes).
/// 基准测试：XOF 输出 11 个块（704 字节）。
#[bench]
fn bench_xof_11_blocks(b: &mut Bencher) {
    bench_xof(b, 11 * BLOCK_LEN);
}

/// Benchmark: XOF output of 12 blocks (768 bytes).
/// 基准测试：XOF 输出 12 个块（768 字节）。
#[bench]
fn bench_xof_12_blocks(b: &mut Bencher) {
    bench_xof(b, 12 * BLOCK_LEN);
}

/// Benchmark: XOF output of 13 blocks (832 bytes).
/// 基准测试：XOF 输出 13 个块（832 字节）。
#[bench]
fn bench_xof_13_blocks(b: &mut Bencher) {
    bench_xof(b, 13 * BLOCK_LEN);
}

/// Benchmark: XOF output of 14 blocks (896 bytes).
/// 基准测试：XOF 输出 14 个块（896 字节）。
#[bench]
fn bench_xof_14_blocks(b: &mut Bencher) {
    bench_xof(b, 14 * BLOCK_LEN);
}

/// Benchmark: XOF output of 15 blocks (960 bytes).
/// 基准测试：XOF 输出 15 个块（960 字节）。
#[bench]
fn bench_xof_15_blocks(b: &mut Bencher) {
    bench_xof(b, 15 * BLOCK_LEN);
}

/// Benchmark: XOF output of 16 blocks (1024 bytes).
/// 基准测试：XOF 输出 16 个块（1024 字节）。
#[bench]
fn bench_xof_16_blocks(b: &mut Bencher) {
    bench_xof(b, 16 * BLOCK_LEN);
}

/// Benchmark: XOF output of 32 blocks (2048 bytes).
/// 基准测试：XOF 输出 32 个块（2048 字节）。
#[bench]
fn bench_xof_32_blocks(b: &mut Bencher) {
    bench_xof(b, 32 * BLOCK_LEN);
}

/// Benchmark: XOF output of 64 blocks (4096 bytes).
/// 基准测试：XOF 输出 64 个块（4096 字节）。
#[bench]
fn bench_xof_64_blocks(b: &mut Bencher) {
    bench_xof(b, 64 * BLOCK_LEN);
}
