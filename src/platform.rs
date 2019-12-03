use crate::{portable, BLOCK_LEN, KEY_LEN};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::{avx2, sse41};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const MAX_SIMD_DEGREE: usize = 8;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const MAX_SIMD_DEGREE: usize = 1;

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Portable,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE41,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX2,
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if avx2_detected() {
                return Platform::AVX2;
            }
            if sse41_detected() {
                return Platform::SSE41;
            }
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
        };
        debug_assert!(degree <= MAX_SIMD_DEGREE);
        degree
    }

    pub fn compress(
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
        }
    }

    pub fn hash_many<A: arrayvec::Array<Item = u8>>(
        &self,
        inputs: &[&A],
        key: &[u8; KEY_LEN],
        offset: u64,
        offset_deltas: &[u64; 16],
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
        }
    }
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
