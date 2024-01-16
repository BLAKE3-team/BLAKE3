use crate::{CVWords, IncrementCounter, BLOCK_LEN, OUT_LEN};

// Unsafe because this may only be called on platforms supporting RVV.
pub unsafe fn compress_in_place(
    cv: &mut CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) {
    ffi::blake3_compress_in_place_rvv(cv.as_mut_ptr(), block.as_ptr(), block_len, counter, flags)
}

// Unsafe because this may only be called on platforms supporting RVV.
pub unsafe fn compress_xof(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) -> [u8; 64] {
    let mut out = [0u8; 64];
    ffi::blake3_compress_xof_rvv(
        cv.as_ptr(),
        block.as_ptr(),
        block_len,
        counter,
        flags,
        out.as_mut_ptr(),
    );
    out
}

// Unsafe because this may only be called on platforms supporting RVV.
pub unsafe fn hash_many<const N: usize>(
    inputs: &[&[u8; N]],
    key: &CVWords,
    counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8],
) {
    // The Rust hash_many implementations do bounds checking on the `out`
    // array, but the C implementations don't. Even though this is an unsafe
    // function, assert the bounds here.
    assert!(out.len() >= inputs.len() * OUT_LEN);
    ffi::blake3_hash_many_rvv(
        inputs.as_ptr() as *const *const u8,
        inputs.len(),
        N / BLOCK_LEN,
        key.as_ptr(),
        counter,
        increment_counter.yes(),
        flags,
        flags_start,
        flags_end,
        out.as_mut_ptr(),
    )
}

pub mod ffi {
    extern "C" {
        pub fn blake3_compress_in_place_rvv(
            cv: *mut u32,
            block: *const u8,
            block_len: u8,
            counter: u64,
            flags: u8,
        );
        pub fn blake3_compress_xof_rvv(
            cv: *const u32,
            block: *const u8,
            block_len: u8,
            counter: u64,
            flags: u8,
            out: *mut u8,
        );
        pub fn blake3_hash_many_rvv(
            inputs: *const *const u8,
            num_inputs: usize,
            blocks: usize,
            key: *const u32,
            counter: u64,
            increment_counter: bool,
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

    // This entire file is gated on feature="rvv", so RVV support is
    // assumed here.

    // TODO: test transpose?

    #[test]
    fn test_compress() {
        crate::test::test_compress_fn(compress_in_place, compress_xof);
    }

    #[test]
    fn test_hash_many() {
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }
}
