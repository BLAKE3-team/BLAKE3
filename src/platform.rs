use crate::{portable, OffsetDeltas, BLOCK_LEN, KEY_LEN};

#[cfg(feature = "c_avx512")]
use crate::c_avx512;
#[cfg(feature = "c_neon")]
use crate::c_neon;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::{avx2, sse41};

cfg_if::cfg_if! {
    if #[cfg(feature = "c_avx512")] {
        pub const MAX_SIMD_DEGREE: usize = 16;
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub const MAX_SIMD_DEGREE: usize = 8;
    } else if #[cfg(feature = "c_neon")] {
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
    if #[cfg(feature = "c_avx512")] {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 16;
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 8;
    } else if #[cfg(feature = "c_neon")] {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 4;
    } else {
        pub const MAX_SIMD_DEGREE_OR_2: usize = 2;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Portable,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE41,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX2,
    #[cfg(feature = "c_avx512")]
    AVX512,
    #[cfg(feature = "c_neon")]
    NEON,
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            #[cfg(feature = "c_avx512")]
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
        }
        // We don't use dynamic feature detection for NEON. If the "c_neon"
        // feature is on, NEON is assumed to be supported.
        #[cfg(feature = "c_neon")]
        {
            return Platform::NEON;
        }
        Platform::Portable
    }

    pub fn simd_degree(&self) -> usize {
        let degree = match self {
            Platform::Portable => 1,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => 4,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => 8,
            #[cfg(feature = "c_avx512")]
            Platform::AVX512 => 16,
            #[cfg(feature = "c_neon")]
            Platform::NEON => 4,
        };
        debug_assert!(degree <= MAX_SIMD_DEGREE);
        degree
    }

    pub(crate) fn compress(
        &self,
        cv: &[u8; 32],
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        offset: u64,
        flags: u8,
    ) -> [u8; 64] {
        match self {
            Platform::Portable => portable::compress(cv, block, block_len, offset, flags),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                sse41::compress(cv, block, block_len, offset, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(feature = "c_avx512")]
            Platform::AVX512 => unsafe { c_avx512::compress(cv, block, block_len, offset, flags) },
            // No NEON compress() implementation yet.
            #[cfg(feature = "c_neon")]
            Platform::NEON => portable::compress(cv, block, block_len, offset, flags),
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

    pub(crate) fn hash_many<A: arrayvec::Array<Item = u8>>(
        &self,
        inputs: &[&A],
        key: &[u8; KEY_LEN],
        offset: u64,
        offset_deltas: &OffsetDeltas,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: &mut [u8],
    ) {
        match self {
            Platform::Portable => portable::hash_many(
                inputs,
                key,
                offset,
                offset_deltas,
                flags,
                flags_start,
                flags_end,
                out,
            ),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => unsafe {
                sse41::hash_many(
                    inputs,
                    key,
                    offset,
                    offset_deltas,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => unsafe {
                avx2::hash_many(
                    inputs,
                    key,
                    offset,
                    offset_deltas,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(feature = "c_avx512")]
            Platform::AVX512 => unsafe {
                c_avx512::hash_many(
                    inputs,
                    key,
                    offset,
                    offset_deltas,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Assumed to be safe if the "c_neon" feature is on.
            #[cfg(feature = "c_neon")]
            Platform::NEON => unsafe {
                c_neon::hash_many(
                    inputs,
                    key,
                    offset,
                    offset_deltas,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
        }
    }
}

// Note that AVX-512 is divided into multiple featuresets, and we use two of
// them, F and VL.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx512_detected() -> bool {
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(all(target_feature = "avx512f", target_feature = "avx512vl"))]
    {
        return true;
    }
    // Dyanmic check, if std is enabled.
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
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "avx2")]
    {
        return true;
    }
    // Dyanmic check, if std is enabled.
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
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "sse4.1")]
    {
        return true;
    }
    // Dyanmic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("sse4.1") {
            return true;
        }
    }
    false
}
