use crate::{portable, CVWords, IncrementCounter, BLOCK_LEN};

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
    } else if #[cfg(blake3_wasm32_simd)] {
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
    } else if #[cfg(blake3_wasm32_simd)] {
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
    #[cfg(blake3_wasm32_simd)]
    #[allow(non_camel_case_types)]
    WASM32_SIMD,
}

impl Platform {
    #[allow(unreachable_code)]
    pub fn detect() -> Self {
        #[cfg(miri)]
        {
            return Platform::Portable;
        }

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
        #[cfg(blake3_wasm32_simd)]
        {
            return Platform::WASM32_SIMD;
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
            #[cfg(blake3_wasm32_simd)]
            Platform::WASM32_SIMD => 4,
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
            #[cfg(blake3_wasm32_simd)]
            Platform::WASM32_SIMD => {
                crate::wasm32_simd::compress_in_place(cv, block, block_len, counter, flags)
            }
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
            #[cfg(blake3_wasm32_simd)]
            Platform::WASM32_SIMD => {
                crate::wasm32_simd::compress_xof(cv, block, block_len, counter, flags)
            }
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
            // Assumed to be safe if the "wasm32_simd" feature is on.
            #[cfg(blake3_wasm32_simd)]
            Platform::WASM32_SIMD => unsafe {
                crate::wasm32_simd::hash_many(
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

    pub fn xof_many(
        &self,
        cv: &CVWords,
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        mut counter: u64,
        flags: u8,
        out: &mut [u8],
    ) {
        debug_assert_eq!(0, out.len() % BLOCK_LEN, "whole blocks only");
        if out.is_empty() {
            // The current assembly implementation always outputs at least 1 block.
            return;
        }
        match self {
            // Safe because detect() checked for platform support.
            #[cfg(blake3_avx512_ffi)]
            #[cfg(unix)]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                crate::avx512::xof_many(cv, block, block_len, counter, flags, out)
            },
            _ => {
                // For platforms without an optimized xof_many, fall back to a loop over
                // compress_xof. This is still faster than portable code.
                for out_block in out.chunks_exact_mut(BLOCK_LEN) {
                    // TODO: Use array_chunks_mut here once that's stable.
                    let out_array: &mut [u8; BLOCK_LEN] = out_block.try_into().unwrap();
                    *out_array = self.compress_xof(cv, block, block_len, counter, flags);
                    counter += 1;
                }
            }
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

    #[cfg(blake3_wasm32_simd)]
    pub fn wasm32_simd() -> Option<Self> {
        // Assumed to be safe if the "wasm32_simd" feature is on.
        Some(Self::WASM32_SIMD)
    }
}

// Note that AVX-512 is divided into multiple featuresets, and we use two of
// them, F and VL.
#[cfg(blake3_avx512_ffi)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
#[allow(unreachable_code)]
pub fn avx512_detected() -> bool {
    if cfg!(miri) {
        return false;
    }

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
#[allow(unreachable_code)]
pub fn avx2_detected() -> bool {
    if cfg!(miri) {
        return false;
    }

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
#[allow(unreachable_code)]
pub fn sse41_detected() -> bool {
    if cfg!(miri) {
        return false;
    }

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
    if cfg!(miri) {
        return false;
    }

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

macro_rules! extract_u32_from_byte_chunks {
    ($src:ident, $chunk_index:literal) => {
        u32::from_le_bytes([
            $src[$chunk_index * 4 + 0],
            $src[$chunk_index * 4 + 1],
            $src[$chunk_index * 4 + 2],
            $src[$chunk_index * 4 + 3],
        ])
    };
}

macro_rules! store_u32_to_by_chunks {
    ($src:ident, $dst:ident, $chunk_index:literal) => {
        [
            $dst[$chunk_index * 4 + 0],
            $dst[$chunk_index * 4 + 1],
            $dst[$chunk_index * 4 + 2],
            $dst[$chunk_index * 4 + 3],
        ] = $src[$chunk_index].to_le_bytes();
    };
}

#[inline(always)]
pub const fn words_from_le_bytes_32(bytes: &[u8; 32]) -> [u32; 8] {
    let mut out = [0; 8];
    out[0] = extract_u32_from_byte_chunks!(bytes, 0);
    out[1] = extract_u32_from_byte_chunks!(bytes, 1);
    out[2] = extract_u32_from_byte_chunks!(bytes, 2);
    out[3] = extract_u32_from_byte_chunks!(bytes, 3);
    out[4] = extract_u32_from_byte_chunks!(bytes, 4);
    out[5] = extract_u32_from_byte_chunks!(bytes, 5);
    out[6] = extract_u32_from_byte_chunks!(bytes, 6);
    out[7] = extract_u32_from_byte_chunks!(bytes, 7);
    out
}

#[inline(always)]
pub const fn words_from_le_bytes_64(bytes: &[u8; 64]) -> [u32; 16] {
    let mut out = [0; 16];
    out[0] = extract_u32_from_byte_chunks!(bytes, 0);
    out[1] = extract_u32_from_byte_chunks!(bytes, 1);
    out[2] = extract_u32_from_byte_chunks!(bytes, 2);
    out[3] = extract_u32_from_byte_chunks!(bytes, 3);
    out[4] = extract_u32_from_byte_chunks!(bytes, 4);
    out[5] = extract_u32_from_byte_chunks!(bytes, 5);
    out[6] = extract_u32_from_byte_chunks!(bytes, 6);
    out[7] = extract_u32_from_byte_chunks!(bytes, 7);
    out[8] = extract_u32_from_byte_chunks!(bytes, 8);
    out[9] = extract_u32_from_byte_chunks!(bytes, 9);
    out[10] = extract_u32_from_byte_chunks!(bytes, 10);
    out[11] = extract_u32_from_byte_chunks!(bytes, 11);
    out[12] = extract_u32_from_byte_chunks!(bytes, 12);
    out[13] = extract_u32_from_byte_chunks!(bytes, 13);
    out[14] = extract_u32_from_byte_chunks!(bytes, 14);
    out[15] = extract_u32_from_byte_chunks!(bytes, 15);
    out
}

#[inline(always)]
pub const fn le_bytes_from_words_32(words: &[u32; 8]) -> [u8; 32] {
    let mut out = [0; 32];
    store_u32_to_by_chunks!(words, out, 0);
    store_u32_to_by_chunks!(words, out, 1);
    store_u32_to_by_chunks!(words, out, 2);
    store_u32_to_by_chunks!(words, out, 3);
    store_u32_to_by_chunks!(words, out, 4);
    store_u32_to_by_chunks!(words, out, 5);
    store_u32_to_by_chunks!(words, out, 6);
    store_u32_to_by_chunks!(words, out, 7);
    out
}

#[inline(always)]
pub const fn le_bytes_from_words_64(words: &[u32; 16]) -> [u8; 64] {
    let mut out = [0; 64];
    store_u32_to_by_chunks!(words, out, 0);
    store_u32_to_by_chunks!(words, out, 1);
    store_u32_to_by_chunks!(words, out, 2);
    store_u32_to_by_chunks!(words, out, 3);
    store_u32_to_by_chunks!(words, out, 4);
    store_u32_to_by_chunks!(words, out, 5);
    store_u32_to_by_chunks!(words, out, 6);
    store_u32_to_by_chunks!(words, out, 7);
    store_u32_to_by_chunks!(words, out, 8);
    store_u32_to_by_chunks!(words, out, 9);
    store_u32_to_by_chunks!(words, out, 10);
    store_u32_to_by_chunks!(words, out, 11);
    store_u32_to_by_chunks!(words, out, 12);
    store_u32_to_by_chunks!(words, out, 13);
    store_u32_to_by_chunks!(words, out, 14);
    store_u32_to_by_chunks!(words, out, 15);
    out
}
