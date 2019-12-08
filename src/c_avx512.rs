use crate::{OffsetDeltas, BLOCK_LEN, KEY_LEN, OUT_LEN};

// Unsafe because this may only be called on platforms supporting AVX-512.
pub unsafe fn compress(
    cv: &[u8; 32],
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    offset: u64,
    flags: u8,
) -> [u8; 64] {
    let mut out = [0u8; 64];
    ffi::blake3_compress_avx512(
        cv.as_ptr(),
        block.as_ptr(),
        block_len,
        offset,
        flags,
        out.as_mut_ptr(),
    );
    out
}

// Unsafe because this may only be called on platforms supporting AVX-512.
pub unsafe fn hash_many<A: arrayvec::Array<Item = u8>>(
    inputs: &[&A],
    key: &[u8; KEY_LEN],
    offset: u64,
    offset_deltas: &OffsetDeltas,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8],
) {
    // The Rust hash_many implementations do bounds checking on the `out`
    // array, but the C implementations don't. Even though this is an unsafe
    // function, assert the bounds here.
    assert!(out.len() >= inputs.len() * OUT_LEN);
    ffi::blake3_hash_many_avx512(
        inputs.as_ptr() as *const *const u8,
        inputs.len(),
        A::CAPACITY / BLOCK_LEN,
        key.as_ptr(),
        offset,
        offset_deltas.as_ptr(),
        flags,
        flags_start,
        flags_end,
        out.as_mut_ptr(),
    )
}

pub mod ffi {
    extern "C" {
        pub fn blake3_compress_avx512(
            cv: *const u8,
            block: *const u8,
            block_len: u8,
            offset: u64,
            flags: u8,
            out: *mut u8,
        );
        // hash4/hash8/hash16 are exposed here for benchmarks.
        pub fn blake3_hash4_avx512(
            inputs: *const *const u8,
            blocks: usize,
            key: *const u8,
            offset: u64,
            offset_deltas: *const u64,
            flags: u8,
            flags_start: u8,
            flags_end: u8,
            out: *mut u8,
        );
        pub fn blake3_hash8_avx512(
            inputs: *const *const u8,
            blocks: usize,
            key: *const u8,
            offset: u64,
            offset_deltas: *const u64,
            flags: u8,
            flags_start: u8,
            flags_end: u8,
            out: *mut u8,
        );
        pub fn blake3_hash16_avx512(
            inputs: *const *const u8,
            blocks: usize,
            key: *const u8,
            offset: u64,
            offset_deltas: *const u64,
            flags: u8,
            flags_start: u8,
            flags_end: u8,
            out: *mut u8,
        );
        pub fn blake3_hash_many_avx512(
            inputs: *const *const u8,
            num_inputs: usize,
            blocks: usize,
            key: *const u8,
            offset: u64,
            offset_deltas: *const u64,
            flags: u8,
            flags_start: u8,
            flags_end: u8,
            out: *mut u8,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress() {
        if !crate::platform::avx512_detected() {
            return;
        }
        crate::test::test_compress_fn(compress);
    }

    #[test]
    fn test_hash_many() {
        if !crate::platform::avx512_detected() {
            return;
        }
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }
}
