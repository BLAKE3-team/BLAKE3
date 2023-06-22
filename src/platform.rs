use crate::{portable, CVWords, IncrementCounter, BLOCK_LEN, CHUNK_LEN};
use arrayref::{array_mut_ref, array_ref};
use atomic::Atomic;
use core::cmp;
use core::ops::{Deref, DerefMut};
use core::ptr;

const CHUNK_START: u32 = 1 << 0;
const CHUNK_END: u32 = 1 << 1;
const PARENT: u32 = 1 << 2;
const ROOT: u32 = 1 << 3;
const KEYED_HASH: u32 = 1 << 4;
const DERIVE_KEY_CONTEXT: u32 = 1 << 5;
const DERIVE_KEY_MATERIAL: u32 = 1 << 6;

struct Implementation {
    compress: Atomic<CompressFn>,
    hash_chunks: Atomic<HashChunksFn>,
    hash_parents: Atomic<HashParentsFn>,
    xof: Atomic<XofFn>,
    xof_xor: Atomic<XofFn>,
    universal_hash: Atomic<UniversalHashFn>,
}

impl Implementation {
    fn portable() -> Self {
        Self {
            compress: Atomic::new(portable::compress),
            hash_chunks: Atomic::new(portable::hash_chunks),
            hash_parents: Atomic::new(portable::hash_parents),
            xof: Atomic::new(portable::xof),
            xof_xor: Atomic::new(portable::xof_xor),
            universal_hash: Atomic::new(portable::universal_hash),
        }
    }
}

type CompressFn = unsafe extern "C" fn(
    block: *const [u8; 64], // zero padded to 64 bytes
    block_len: u32,
    cv: *const [u32; 8],
    counter: u64,
    flags: u32,
    out: *mut [u32; 16], // may overlap the input
);

type HashChunksFn = unsafe extern "C" fn(
    input: *const u8,
    input_len: usize,
    key: *const [u32; 8],
    counter: u64,
    flags: u32,
    transposed_output: *mut u32,
);

type HashParentsFn = unsafe extern "C" fn(
    transposed_input: *const u32,
    num_parents: usize,
    key: *const [u32; 8],
    flags: u32,
    transposed_output: *mut u32, // may overlap the input
);

// This signature covers both xof() and xof_xor().
type XofFn = unsafe extern "C" fn(
    block: *const [u8; 64], // zero padded to 64 bytes
    block_len: u32,
    cv: *const [u32; 8],
    counter: u64,
    flags: u32,
    out: *mut u8,
    out_len: usize,
);

type UniversalHashFn = unsafe extern "C" fn(
    input: *const u8,
    input_len: usize,
    key: *const [u32; 8],
    counter: u64,
    out: *mut [u8; 16],
);

// The implicit degree of this implementation is MAX_SIMD_DEGREE.
pub(crate) unsafe fn hash_chunks_using_compress(
    compress: CompressFn,
    mut input: *const u8,
    mut input_len: usize,
    key: *const [u32; 8],
    mut counter: u64,
    flags: u32,
    mut transposed_output: *mut u32,
) {
    debug_assert!(input_len > 0);
    debug_assert!(input_len <= MAX_SIMD_DEGREE * CHUNK_LEN);
    while input_len > 0 {
        let mut chunk_len = cmp::min(input_len, CHUNK_LEN);
        input_len -= chunk_len;
        // We only use 8 words of the CV, but compress returns 16.
        let mut cv = [0u32; 16];
        cv[..8].copy_from_slice(&*key);
        let cv_ptr: *mut [u32; 16] = &mut cv;
        let mut chunk_flags = flags | CHUNK_START;
        while chunk_len > BLOCK_LEN {
            compress(
                input as *const [u8; 64],
                BLOCK_LEN as u32,
                cv_ptr as *const [u32; 8],
                counter,
                chunk_flags,
                cv_ptr,
            );
            input = input.add(BLOCK_LEN);
            chunk_len -= BLOCK_LEN;
            chunk_flags &= !CHUNK_START;
        }
        let mut last_block = [0u8; BLOCK_LEN];
        ptr::copy_nonoverlapping(input, last_block.as_mut_ptr(), chunk_len);
        input = input.add(chunk_len);
        compress(
            &last_block,
            chunk_len as u32,
            cv_ptr as *const [u32; 8],
            counter,
            chunk_flags | CHUNK_END,
            cv_ptr,
        );
        for word_index in 0..8 {
            transposed_output
                .add(word_index * TRANSPOSED_STRIDE)
                .write(cv[word_index]);
        }
        transposed_output = transposed_output.add(1);
        counter += 1;
    }
}

// The implicit degree of this implementation is MAX_SIMD_DEGREE.
pub(crate) unsafe fn hash_parents_using_compress(
    compress: CompressFn,
    mut transposed_input: *const u32,
    mut num_parents: usize,
    key: *const [u32; 8],
    flags: u32,
    mut transposed_output: *mut u32, // may overlap the input
) {
    debug_assert!(num_parents > 0);
    debug_assert!(num_parents <= MAX_SIMD_DEGREE);
    while num_parents > 0 {
        let mut block_bytes = [0u8; 64];
        for word_index in 0..8 {
            let left_child_word = transposed_input.add(word_index * TRANSPOSED_STRIDE).read();
            block_bytes[4 * word_index..][..4].copy_from_slice(&left_child_word.to_le_bytes());
            let right_child_word = transposed_input
                .add(word_index * TRANSPOSED_STRIDE + 1)
                .read();
            block_bytes[4 * (word_index + 8)..][..4]
                .copy_from_slice(&right_child_word.to_le_bytes());
        }
        let mut cv = [0u32; 16];
        compress(&block_bytes, BLOCK_LEN as u32, key, 0, flags, &mut cv);
        for word_index in 0..8 {
            transposed_output
                .add(word_index * TRANSPOSED_STRIDE)
                .write(cv[word_index]);
        }
        transposed_input = transposed_input.add(2);
        transposed_output = transposed_output.add(1);
        num_parents -= 1;
    }
}

pub(crate) unsafe fn xof_using_compress(
    compress: CompressFn,
    block: *const [u8; 64],
    block_len: u32,
    cv: *const [u32; 8],
    mut counter: u64,
    flags: u32,
    mut out: *mut u8,
    mut out_len: usize,
) {
    while out_len > 0 {
        let mut block_output = [0u32; 16];
        compress(block, block_len, cv, counter, flags, &mut block_output);
        for output_word in block_output {
            let bytes = output_word.to_le_bytes();
            let take = cmp::min(bytes.len(), out_len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), out, take);
            out = out.add(take);
            out_len -= take;
        }
        counter += 1;
    }
}

pub(crate) unsafe fn xof_xor_using_compress(
    compress: CompressFn,
    block: *const [u8; 64],
    block_len: u32,
    cv: *const [u32; 8],
    mut counter: u64,
    flags: u32,
    mut out: *mut u8,
    mut out_len: usize,
) {
    while out_len > 0 {
        let mut block_output = [0u32; 16];
        compress(block, block_len, cv, counter, flags, &mut block_output);
        for output_word in block_output {
            let bytes = output_word.to_le_bytes();
            for i in 0..cmp::min(bytes.len(), out_len) {
                *out = *out ^ bytes[i];
                out = out.add(1);
                out_len -= 1;
            }
        }
        counter += 1;
    }
}

pub(crate) unsafe fn universal_hash_using_compress(
    compress: CompressFn,
    mut input: *const u8,
    mut input_len: usize,
    key: *const [u32; 8],
    mut counter: u64,
    out: *mut [u8; 16],
) {
    let flags = KEYED_HASH | CHUNK_START | CHUNK_END | ROOT;
    let mut result = [0u32; 4];
    while input_len > 0 {
        let block_len = cmp::min(input_len, BLOCK_LEN);
        let mut block = [0u8; BLOCK_LEN];
        ptr::copy_nonoverlapping(input, block.as_mut_ptr(), block_len);
        let mut block_output = [0u32; 16];
        compress(
            &block,
            BLOCK_LEN as u32,
            key,
            counter,
            flags,
            &mut block_output,
        );
        for i in 0..4 {
            result[i] ^= block_output[i];
        }
        input = input.add(block_len);
        input_len -= block_len;
        counter += 1;
    }
    for i in 0..4 {
        (*out)[4 * i..][..4].copy_from_slice(&result[i].to_le_bytes());
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        cfg_if::cfg_if! {
            if #[cfg(blake3_avx512_ffi)] {
                pub const MAX_SIMD_DEGREE: usize = 16;
            } else {
                pub const MAX_SIMD_DEGREE: usize = 8;
            }
        }
    } else if #[cfg(blake3_neon)] {
        pub const MAX_SIMD_DEGREE: usize = 4;
    } else {
        pub const MAX_SIMD_DEGREE: usize = 1;
    }
}

// There are some places where we want a static size that's equal to the
// MAX_SIMD_DEGREE, but also at least 2. Constant contexts aren't currently
// allowed to use cmp::max, so we have to hardcode this additional constant
// value. Get rid of this once cmp::max is a const fn.
cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        cfg_if::cfg_if! {
            if #[cfg(blake3_avx512_ffi)] {
                pub const MAX_SIMD_DEGREE_OR_2: usize = 16;
            } else {
                pub const MAX_SIMD_DEGREE_OR_2: usize = 8;
            }
        }
    } else if #[cfg(blake3_neon)] {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 4;
    } else {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 2;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Portable,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE2,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE41,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX2,
    #[cfg(blake3_avx512_ffi)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX512,
    #[cfg(blake3_neon)]
    NEON,
}

impl Platform {
    #[allow(unreachable_code)]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            #[cfg(blake3_avx512_ffi)]
            {
                if avx512_detected() {
                    return Platform::AVX512;
                }
            }
            if avx2_detected() {
                return Platform::AVX2;
            }
            if sse41_detected() {
                return Platform::SSE41;
            }
            if sse2_detected() {
                return Platform::SSE2;
            }
        }
        // We don't use dynamic feature detection for NEON. If the "neon"
        // feature is on, NEON is assumed to be supported.
        #[cfg(blake3_neon)]
        {
            return Platform::NEON;
        }
        Platform::Portable
    }

    pub fn simd_degree(&self) -> usize {
        let degree = match self {
            Platform::Portable => 1,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => 4,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => 4,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => 8,
            #[cfg(blake3_avx512_ffi)]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => 16,
            #[cfg(blake3_neon)]
            Platform::NEON => 4,
        };
        debug_assert!(degree <= MAX_SIMD_DEGREE);
        degree
    }

    pub fn compress_in_place(
        &self,
        cv: &mut CVWords,
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        counter: u64,
        flags: u8,
    ) {
        match self {
            Platform::Portable => portable::compress_in_place(cv, block, block_len, counter, flags),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                crate::sse2::compress_in_place(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                crate::sse41::compress_in_place(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(blake3_avx512_ffi)]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                crate::avx512::compress_in_place(cv, block, block_len, counter, flags)
            },
            // No NEON compress_in_place() implementation yet.
            #[cfg(blake3_neon)]
            Platform::NEON => portable::compress_in_place(cv, block, block_len, counter, flags),
        }
    }

    pub fn compress_xof(
        &self,
        cv: &CVWords,
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        counter: u64,
        flags: u8,
    ) -> [u8; 64] {
        match self {
            Platform::Portable => portable::compress_xof(cv, block, block_len, counter, flags),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                crate::sse2::compress_xof(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                crate::sse41::compress_xof(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(blake3_avx512_ffi)]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                crate::avx512::compress_xof(cv, block, block_len, counter, flags)
            },
            // No NEON compress_xof() implementation yet.
            #[cfg(blake3_neon)]
            Platform::NEON => portable::compress_xof(cv, block, block_len, counter, flags),
        }
    }

    // IMPLEMENTATION NOTE
    // ===================
    // hash_many() applies two optimizations. The critically important
    // optimization is the high-performance parallel SIMD hashing mode,
    // described in detail in the spec. This more than doubles throughput per
    // thread. Another optimization is keeping the state vectors transposed
    // from block to block within a chunk. When state vectors are transposed
    // after every block, there's a small but measurable performance loss.
    // Compressing chunks with a dedicated loop avoids this.

    pub fn hash_many<const N: usize>(
        &self,
        inputs: &[&[u8; N]],
        key: &CVWords,
        counter: u64,
        increment_counter: IncrementCounter,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: &mut [u8],
    ) {
        match self {
            Platform::Portable => portable::hash_many(
                inputs,
                key,
                counter,
                increment_counter,
                flags,
                flags_start,
                flags_end,
                out,
            ),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                crate::sse2::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => unsafe {
                crate::sse41::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => unsafe {
                crate::avx2::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(blake3_avx512_ffi)]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                crate::avx512::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Assumed to be safe if the "neon" feature is on.
            #[cfg(blake3_neon)]
            Platform::NEON => unsafe {
                crate::neon::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
        }
    }

    // Explicit platform constructors, for benchmarks.

    pub fn portable() -> Self {
        Self::Portable
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sse2() -> Option<Self> {
        if sse2_detected() {
            Some(Self::SSE2)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sse41() -> Option<Self> {
        if sse41_detected() {
            Some(Self::SSE41)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn avx2() -> Option<Self> {
        if avx2_detected() {
            Some(Self::AVX2)
        } else {
            None
        }
    }

    #[cfg(blake3_avx512_ffi)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn avx512() -> Option<Self> {
        if avx512_detected() {
            Some(Self::AVX512)
        } else {
            None
        }
    }

    #[cfg(blake3_neon)]
    pub fn neon() -> Option<Self> {
        // Assumed to be safe if the "neon" feature is on.
        Some(Self::NEON)
    }
}

// Note that AVX-512 is divided into multiple featuresets, and we use two of
// them, F and VL.
#[cfg(blake3_avx512_ffi)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx512_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_avx512") {
        return false;
    }
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(all(target_feature = "avx512f", target_feature = "avx512vl"))]
    {
        return true;
    }
    // Dynamic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512vl") {
            return true;
        }
    }
    false
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx2_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_avx2") {
        return false;
    }
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "avx2")]
    {
        return true;
    }
    // Dynamic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("avx2") {
            return true;
        }
    }
    false
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn sse41_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_sse41") {
        return false;
    }
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "sse4.1")]
    {
        return true;
    }
    // Dynamic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("sse4.1") {
            return true;
        }
    }
    false
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
#[allow(unreachable_code)]
pub fn sse2_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_sse2") {
        return false;
    }
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "sse2")]
    {
        return true;
    }
    // Dynamic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("sse2") {
            return true;
        }
    }
    false
}

#[inline(always)]
pub fn words_from_le_bytes_32(bytes: &[u8; 32]) -> [u32; 8] {
    let mut out = [0; 8];
    out[0] = u32::from_le_bytes(*array_ref!(bytes, 0 * 4, 4));
    out[1] = u32::from_le_bytes(*array_ref!(bytes, 1 * 4, 4));
    out[2] = u32::from_le_bytes(*array_ref!(bytes, 2 * 4, 4));
    out[3] = u32::from_le_bytes(*array_ref!(bytes, 3 * 4, 4));
    out[4] = u32::from_le_bytes(*array_ref!(bytes, 4 * 4, 4));
    out[5] = u32::from_le_bytes(*array_ref!(bytes, 5 * 4, 4));
    out[6] = u32::from_le_bytes(*array_ref!(bytes, 6 * 4, 4));
    out[7] = u32::from_le_bytes(*array_ref!(bytes, 7 * 4, 4));
    out
}

#[inline(always)]
pub fn words_from_le_bytes_64(bytes: &[u8; 64]) -> [u32; 16] {
    let mut out = [0; 16];
    out[0] = u32::from_le_bytes(*array_ref!(bytes, 0 * 4, 4));
    out[1] = u32::from_le_bytes(*array_ref!(bytes, 1 * 4, 4));
    out[2] = u32::from_le_bytes(*array_ref!(bytes, 2 * 4, 4));
    out[3] = u32::from_le_bytes(*array_ref!(bytes, 3 * 4, 4));
    out[4] = u32::from_le_bytes(*array_ref!(bytes, 4 * 4, 4));
    out[5] = u32::from_le_bytes(*array_ref!(bytes, 5 * 4, 4));
    out[6] = u32::from_le_bytes(*array_ref!(bytes, 6 * 4, 4));
    out[7] = u32::from_le_bytes(*array_ref!(bytes, 7 * 4, 4));
    out[8] = u32::from_le_bytes(*array_ref!(bytes, 8 * 4, 4));
    out[9] = u32::from_le_bytes(*array_ref!(bytes, 9 * 4, 4));
    out[10] = u32::from_le_bytes(*array_ref!(bytes, 10 * 4, 4));
    out[11] = u32::from_le_bytes(*array_ref!(bytes, 11 * 4, 4));
    out[12] = u32::from_le_bytes(*array_ref!(bytes, 12 * 4, 4));
    out[13] = u32::from_le_bytes(*array_ref!(bytes, 13 * 4, 4));
    out[14] = u32::from_le_bytes(*array_ref!(bytes, 14 * 4, 4));
    out[15] = u32::from_le_bytes(*array_ref!(bytes, 15 * 4, 4));
    out
}

#[inline(always)]
pub fn le_bytes_from_words_32(words: &[u32; 8]) -> [u8; 32] {
    let mut out = [0; 32];
    *array_mut_ref!(out, 0 * 4, 4) = words[0].to_le_bytes();
    *array_mut_ref!(out, 1 * 4, 4) = words[1].to_le_bytes();
    *array_mut_ref!(out, 2 * 4, 4) = words[2].to_le_bytes();
    *array_mut_ref!(out, 3 * 4, 4) = words[3].to_le_bytes();
    *array_mut_ref!(out, 4 * 4, 4) = words[4].to_le_bytes();
    *array_mut_ref!(out, 5 * 4, 4) = words[5].to_le_bytes();
    *array_mut_ref!(out, 6 * 4, 4) = words[6].to_le_bytes();
    *array_mut_ref!(out, 7 * 4, 4) = words[7].to_le_bytes();
    out
}

#[inline(always)]
pub fn le_bytes_from_words_64(words: &[u32; 16]) -> [u8; 64] {
    let mut out = [0; 64];
    *array_mut_ref!(out, 0 * 4, 4) = words[0].to_le_bytes();
    *array_mut_ref!(out, 1 * 4, 4) = words[1].to_le_bytes();
    *array_mut_ref!(out, 2 * 4, 4) = words[2].to_le_bytes();
    *array_mut_ref!(out, 3 * 4, 4) = words[3].to_le_bytes();
    *array_mut_ref!(out, 4 * 4, 4) = words[4].to_le_bytes();
    *array_mut_ref!(out, 5 * 4, 4) = words[5].to_le_bytes();
    *array_mut_ref!(out, 6 * 4, 4) = words[6].to_le_bytes();
    *array_mut_ref!(out, 7 * 4, 4) = words[7].to_le_bytes();
    *array_mut_ref!(out, 8 * 4, 4) = words[8].to_le_bytes();
    *array_mut_ref!(out, 9 * 4, 4) = words[9].to_le_bytes();
    *array_mut_ref!(out, 10 * 4, 4) = words[10].to_le_bytes();
    *array_mut_ref!(out, 11 * 4, 4) = words[11].to_le_bytes();
    *array_mut_ref!(out, 12 * 4, 4) = words[12].to_le_bytes();
    *array_mut_ref!(out, 13 * 4, 4) = words[13].to_le_bytes();
    *array_mut_ref!(out, 14 * 4, 4) = words[14].to_le_bytes();
    *array_mut_ref!(out, 15 * 4, 4) = words[15].to_le_bytes();
    out
}

// this is in units of *words*, for pointer operations on *const/mut u32
pub const TRANSPOSED_STRIDE: usize = 2 * MAX_SIMD_DEGREE;

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), repr(C, align(64)))]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TransposedVectors {
    pub(crate) vectors: [[u32; 2 * MAX_SIMD_DEGREE]; 8],
    // the number of CVs populated in each vector
    pub(crate) len: usize,
}

impl Deref for TransposedVectors {
    type Target = [[u32; 2 * MAX_SIMD_DEGREE]; 8];
    fn deref(&self) -> &Self::Target {
        &self.vectors
    }
}

impl DerefMut for TransposedVectors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vectors
    }
}

pub enum ParentInOut<'a> {
    InPlace {
        in_out: &'a mut TransposedVectors,
    },
    Separate {
        input: &'a TransposedVectors,
        output: &'a mut TransposedVectors,
    },
}

impl<'a> ParentInOut<'a> {
    pub(crate) fn promote_odd_child_and_update_len(&mut self) {
        match self {
            ParentInOut::InPlace { in_out } => {
                // After an in-place parent hashing step (i.e. reduction near the root), the number
                // of CVs needs to be halved, with a possible adjustment for an odd child.
                if in_out.len % 2 == 1 {
                    for i in 0..8 {
                        in_out.vectors[i][in_out.len / 2] = in_out.vectors[i][in_out.len - 1];
                    }
                    in_out.len = (in_out.len / 2) + 1;
                } else {
                    in_out.len /= 2;
                }
            }
            ParentInOut::Separate { input, output } => {
                // After an out-of-place parent hashing step (i.e. wide hashing near the leaves),
                // the output length is already correct, and all that's needed is the possible
                // adjustment for an odd child.
                if input.len % 2 == 1 {
                    for i in 0..8 {
                        output.vectors[i][output.len] = input.vectors[i][input.len - 1];
                    }
                    output.len += 1;
                }
            }
        }
    }
}
