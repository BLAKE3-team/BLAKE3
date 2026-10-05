use crate::{BLOCK_LEN, CVWords, IncrementCounter, OUT_LEN};

// Unsafe because this may only be called on platforms supporting NEON.
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
    unsafe {
        ffi::blake3_hash_many_neon(
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
}

// Unsafe because this may only be called on platforms supporting NEON.
pub unsafe fn xof_many(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
    out: &mut [u8],
) {
    debug_assert_eq!(0, out.len() % BLOCK_LEN, "whole blocks only");
    unsafe {
        ffi::blake3_xof_many_neon(
            cv.as_ptr(),
            block.as_ptr(),
            block_len,
            counter,
            flags,
            out.as_mut_ptr(),
            out.len() / BLOCK_LEN,
        );
    }
}

// blake3_neon.c normally depends on blake3_portable.c, because the NEON
// implementation only provides 4x compression, and it relies on the portable
// implementation for 1x compression (blake3_compress_in_place_portable for
// hash_many, and blake3_compress_xof_portable for the last 1-3 blocks of
// xof_many). However, we expose the portable Rust implementations here
// instead, to avoid linking in unnecessary code.
#[unsafe(no_mangle)]
pub extern "C" fn blake3_compress_in_place_portable(
    cv: *mut u32,
    block: *const u8,
    block_len: u8,
    counter: u64,
    flags: u8,
) {
    unsafe {
        crate::portable::compress_in_place(
            &mut *(cv as *mut [u32; 8]),
            &*(block as *const [u8; 64]),
            block_len,
            counter,
            flags,
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn blake3_compress_xof_portable(
    cv: *const u32,
    block: *const u8,
    block_len: u8,
    counter: u64,
    flags: u8,
    out: *mut u8,
) {
    unsafe {
        *(out as *mut [u8; 64]) = crate::portable::compress_xof(
            &*(cv as *const [u32; 8]),
            &*(block as *const [u8; 64]),
            block_len,
            counter,
            flags,
        );
    }
}

pub mod ffi {
    unsafe extern "C" {
        pub fn blake3_hash_many_neon(
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
        pub fn blake3_xof_many_neon(
            cv: *const u32,
            block: *const u8,
            block_len: u8,
            counter: u64,
            flags: u8,
            out: *mut u8,
            outblocks: usize,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash_many() {
        // This entire file is gated on feature="neon", so NEON support is
        // assumed here.
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }

    #[test]
    fn test_xof_many() {
        crate::test::test_xof_many_fn(xof_many);
    }
}
