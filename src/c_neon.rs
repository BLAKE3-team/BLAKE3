use crate::{OffsetDeltas, BLOCK_LEN, KEY_LEN, OUT_LEN};

// Unsafe because this may only be called on platforms supporting NEON.
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
    ffi::blake3_hash_many_neon(
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
        // Exposed here for benchmarks.
        pub fn blake3_hash4_neon(
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
        pub fn blake3_hash_many_neon(
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
