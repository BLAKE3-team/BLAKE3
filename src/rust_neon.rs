use core::arch::aarch64::*;

use crate::{
    BLOCK_LEN, CVBytes, CVWords, IV, IncrementCounter, MSG_SCHEDULE, OUT_LEN, counter_high,
    counter_low, portable,
};

pub const DEGREE: usize = 4;

// The index vector for a right rotation by 8 bits within each 32-bit lane.
// `vqtbl1q_u8` compiles to a single NEON `tbl` instruction.
const ROT8_INDEX: [u8; 16] = [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12];

#[inline]
#[target_feature(enable = "neon")]
unsafe fn add(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vaddq_u32(a, b)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn xor(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    veorq_u32(a, b)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn set1(x: u32) -> uint32x4_t {
    vdupq_n_u32(x)
}

#[inline]
/// # Safety
///
/// `src` must be valid for reads of at least `offset + 16` bytes.
unsafe fn loadu(src: *const u8, offset: usize) -> uint32x4_t {
    // SAFETY: The caller guarantees that `src` is valid for reads of at least
    // `offset + 16` bytes.
    let ptr = unsafe { src.add(offset) };
    // SAFETY: `ptr` is in bounds of the same input allocation, per the caller
    // contract above. NEON unaligned byte loads do not require alignment.
    unsafe { vreinterpretq_u32_u8(vld1q_u8(ptr)) }
}

#[inline]
/// # Safety
///
/// `dest` must be valid for writes of at least `offset + 16` bytes.
unsafe fn storeu(src: uint32x4_t, dest: *mut u8, offset: usize) {
    // SAFETY: The caller guarantees that `dest` is valid for writes of at
    // least `offset + 16` bytes.
    let ptr = unsafe { dest.add(offset) };
    // SAFETY: `ptr` is in bounds of the same output allocation, per the
    // caller contract above. NEON unaligned byte stores do not require
    // alignment.
    unsafe { vst1q_u8(ptr, vreinterpretq_u8_u32(src)) }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn rot16(x: uint32x4_t) -> uint32x4_t {
    vreinterpretq_u32_u16(vrev32q_u16(vreinterpretq_u16_u32(x)))
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn rot12(x: uint32x4_t) -> uint32x4_t {
    vsriq_n_u32(vshlq_n_u32(x, 32 - 12), x, 12)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn rot8(x: uint32x4_t) -> uint32x4_t {
    // SAFETY: The table is a static byte array, so its pointer is valid for
    // the 16-byte NEON load.
    unsafe {
        let index = vld1q_u8(ROT8_INDEX.as_ptr());
        vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x), index))
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn rot7(x: uint32x4_t) -> uint32x4_t {
    vsriq_n_u32(vshlq_n_u32(x, 32 - 7), x, 7)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn g(
    v: &mut [uint32x4_t; 16],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    mx: uint32x4_t,
    my: uint32x4_t,
) {
    unsafe {
        v[a] = add(add(v[a], v[b]), mx);
        v[d] = rot16(xor(v[d], v[a]));
        v[c] = add(v[c], v[d]);
        v[b] = rot12(xor(v[b], v[c]));
        v[a] = add(add(v[a], v[b]), my);
        v[d] = rot8(xor(v[d], v[a]));
        v[c] = add(v[c], v[d]);
        v[b] = rot7(xor(v[b], v[c]));
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn round(v: &mut [uint32x4_t; 16], m: &[uint32x4_t; 16], round_index: usize) {
    let s = MSG_SCHEDULE[round_index];
    unsafe {
        // Mix the columns.
        g(v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
        // Mix the diagonals.
        g(v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn transpose_vecs(vecs: &mut [uint32x4_t; DEGREE]) {
    // Interleave 32-bit lanes. `vtrnq_u32` lowers to the `trn1`/`trn2` pair.
    let rows01 = vtrnq_u32(vecs[0], vecs[1]);
    let rows23 = vtrnq_u32(vecs[2], vecs[3]);

    vecs[0] = vcombine_u32(vget_low_u32(rows01.0), vget_low_u32(rows23.0));
    vecs[1] = vcombine_u32(vget_low_u32(rows01.1), vget_low_u32(rows23.1));
    vecs[2] = vcombine_u32(vget_high_u32(rows01.0), vget_high_u32(rows23.0));
    vecs[3] = vcombine_u32(vget_high_u32(rows01.1), vget_high_u32(rows23.1));
}

#[inline]
/// # Safety
///
/// Every input pointer must be valid for reads of the full fixed-size input.
/// NEON must be supported by the CPU.
#[target_feature(enable = "neon")]
unsafe fn transpose_msg_vecs(
    inputs: &[*const u8; DEGREE],
    block_offset: usize,
) -> [uint32x4_t; 16] {
    // SAFETY: The caller guarantees that every input pointer is valid for
    // the full fixed-size input. Each offset stays within one 64-byte block.
    let mut vecs = unsafe {
        [
            loadu(inputs[0], block_offset + 0 * 4 * DEGREE),
            loadu(inputs[1], block_offset + 0 * 4 * DEGREE),
            loadu(inputs[2], block_offset + 0 * 4 * DEGREE),
            loadu(inputs[3], block_offset + 0 * 4 * DEGREE),
            loadu(inputs[0], block_offset + 1 * 4 * DEGREE),
            loadu(inputs[1], block_offset + 1 * 4 * DEGREE),
            loadu(inputs[2], block_offset + 1 * 4 * DEGREE),
            loadu(inputs[3], block_offset + 1 * 4 * DEGREE),
            loadu(inputs[0], block_offset + 2 * 4 * DEGREE),
            loadu(inputs[1], block_offset + 2 * 4 * DEGREE),
            loadu(inputs[2], block_offset + 2 * 4 * DEGREE),
            loadu(inputs[3], block_offset + 2 * 4 * DEGREE),
            loadu(inputs[0], block_offset + 3 * 4 * DEGREE),
            loadu(inputs[1], block_offset + 3 * 4 * DEGREE),
            loadu(inputs[2], block_offset + 3 * 4 * DEGREE),
            loadu(inputs[3], block_offset + 3 * 4 * DEGREE),
        ]
    };

    let (square0, rest) = vecs.split_at_mut(DEGREE);
    let (square1, rest) = rest.split_at_mut(DEGREE);
    let (square2, square3) = rest.split_at_mut(DEGREE);
    unsafe {
        transpose_vecs(square0.try_into().unwrap());
        transpose_vecs(square1.try_into().unwrap());
        transpose_vecs(square2.try_into().unwrap());
        transpose_vecs(square3.try_into().unwrap());
    }
    vecs
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn load_counters(
    counter: u64,
    increment_counter: IncrementCounter,
) -> (uint32x4_t, uint32x4_t) {
    let increment = u64::from(increment_counter.yes());
    unsafe {
        (
            set4(
                counter_low(counter),
                counter_low(counter + increment),
                counter_low(counter + 2 * increment),
                counter_low(counter + 3 * increment),
            ),
            set4(
                counter_high(counter),
                counter_high(counter + increment),
                counter_high(counter + 2 * increment),
                counter_high(counter + 3 * increment),
            ),
        )
    }
}

/// # Safety
///
/// Every input pointer must be valid for reads of `blocks * BLOCK_LEN` bytes,
/// and `out` must be exactly `DEGREE * OUT_LEN` bytes. NEON must be supported
/// by the CPU.
#[target_feature(enable = "neon")]
unsafe fn hash4(
    inputs: &[*const u8; DEGREE],
    blocks: usize,
    key: &CVWords,
    counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8; DEGREE * OUT_LEN],
) {
    let mut h_vecs = unsafe {
        [
            set1(key[0]),
            set1(key[1]),
            set1(key[2]),
            set1(key[3]),
            set1(key[4]),
            set1(key[5]),
            set1(key[6]),
            set1(key[7]),
        ]
    };
    let (counter_low_vec, counter_high_vec) = unsafe { load_counters(counter, increment_counter) };
    let mut block_flags = flags | flags_start;

    for block in 0..blocks {
        if block + 1 == blocks {
            block_flags |= flags_end;
        }
        let block_len_vec = unsafe { set1(BLOCK_LEN as u32) }; // full blocks only
        let block_flags_vec = unsafe { set1(block_flags as u32) };
        // SAFETY: The caller guarantees that each input is valid for
        // `blocks * BLOCK_LEN` bytes. The `blocks` count passed by `hash_many`
        // is derived from the fixed input array length.
        let msg_vecs = unsafe { transpose_msg_vecs(inputs, block * BLOCK_LEN) };

        let mut v = unsafe {
            [
                h_vecs[0],
                h_vecs[1],
                h_vecs[2],
                h_vecs[3],
                h_vecs[4],
                h_vecs[5],
                h_vecs[6],
                h_vecs[7],
                set1(IV[0]),
                set1(IV[1]),
                set1(IV[2]),
                set1(IV[3]),
                counter_low_vec,
                counter_high_vec,
                block_len_vec,
                block_flags_vec,
            ]
        };
        unsafe {
            round(&mut v, &msg_vecs, 0);
            round(&mut v, &msg_vecs, 1);
            round(&mut v, &msg_vecs, 2);
            round(&mut v, &msg_vecs, 3);
            round(&mut v, &msg_vecs, 4);
            round(&mut v, &msg_vecs, 5);
            round(&mut v, &msg_vecs, 6);
            h_vecs[0] = xor(v[0], v[8]);
            h_vecs[1] = xor(v[1], v[9]);
            h_vecs[2] = xor(v[2], v[10]);
            h_vecs[3] = xor(v[3], v[11]);
            h_vecs[4] = xor(v[4], v[12]);
            h_vecs[5] = xor(v[5], v[13]);
            h_vecs[6] = xor(v[6], v[14]);
            h_vecs[7] = xor(v[7], v[15]);
        }

        block_flags = flags;
    }

    let (square0, square1) = h_vecs.split_at_mut(DEGREE);
    unsafe {
        transpose_vecs(square0.try_into().unwrap());
        transpose_vecs(square1.try_into().unwrap());
    }
    // The first four vecs now contain the first half of each output, and the
    // second four vecs contain the second half of each output.
    // SAFETY: `out` is exactly `DEGREE * OUT_LEN` bytes, and each store
    // writes one non-overlapping 16-byte quarter of that array.
    unsafe {
        storeu(h_vecs[0], out.as_mut_ptr(), 0 * 4 * DEGREE);
        storeu(h_vecs[4], out.as_mut_ptr(), 1 * 4 * DEGREE);
        storeu(h_vecs[1], out.as_mut_ptr(), 2 * 4 * DEGREE);
        storeu(h_vecs[5], out.as_mut_ptr(), 3 * 4 * DEGREE);
        storeu(h_vecs[2], out.as_mut_ptr(), 4 * 4 * DEGREE);
        storeu(h_vecs[6], out.as_mut_ptr(), 5 * 4 * DEGREE);
        storeu(h_vecs[3], out.as_mut_ptr(), 6 * 4 * DEGREE);
        storeu(h_vecs[7], out.as_mut_ptr(), 7 * 4 * DEGREE);
    }
}

fn hash1<const N: usize>(
    input: &[u8; N],
    key: &CVWords,
    counter: u64,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut CVBytes,
) {
    debug_assert_eq!(N % BLOCK_LEN, 0, "uneven blocks");
    let mut cv = *key;
    let mut block_flags = flags | flags_start;
    let mut slice = &input[..];
    while slice.len() >= BLOCK_LEN {
        if slice.len() == BLOCK_LEN {
            block_flags |= flags_end;
        }
        portable::compress_in_place(
            &mut cv,
            (&slice[..BLOCK_LEN]).try_into().unwrap(),
            BLOCK_LEN as u8,
            counter,
            block_flags,
        );
        block_flags = flags;
        slice = &slice[BLOCK_LEN..];
    }
    *out = crate::platform::le_bytes_from_words_32(&cv);
}

/// # Safety
///
/// `out` must be valid for writes of `inputs.len() * OUT_LEN` bytes, and every
/// input array must remain valid for the duration of the call. NEON must be
/// supported by the CPU. On AArch64 NEON is architecturally required, and this
/// module is only compiled when the `blake3_neon` configuration is enabled.
#[target_feature(enable = "neon")]
pub unsafe fn hash_many<const N: usize>(
    mut inputs: &[&[u8; N]],
    key: &CVWords,
    mut counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    mut out: &mut [u8],
) {
    debug_assert!(out.len() >= inputs.len() * OUT_LEN, "out too short");
    debug_assert_eq!(N % BLOCK_LEN, 0, "uneven blocks");
    while inputs.len() >= DEGREE && out.len() >= DEGREE * OUT_LEN {
        let input_ptrs: [*const u8; DEGREE] = [
            inputs[0].as_ptr(),
            inputs[1].as_ptr(),
            inputs[2].as_ptr(),
            inputs[3].as_ptr(),
        ];
        let blocks = N / BLOCK_LEN;
        // SAFETY: `input_ptrs` point at fixed-size arrays of length `N`, and
        // `blocks` is exactly `N / BLOCK_LEN`. The output chunk has length
        // `DEGREE * OUT_LEN`.
        unsafe {
            hash4(
                &input_ptrs,
                blocks,
                key,
                counter,
                increment_counter,
                flags,
                flags_start,
                flags_end,
                (&mut out[..DEGREE * OUT_LEN]).try_into().unwrap(),
            );
        }
        if increment_counter.yes() {
            counter += DEGREE as u64;
        }
        inputs = &inputs[DEGREE..];
        out = &mut out[DEGREE * OUT_LEN..];
    }
    for (&input, output) in inputs.iter().zip(out.chunks_exact_mut(OUT_LEN)) {
        hash1(
            input,
            key,
            counter,
            flags,
            flags_start,
            flags_end,
            (&mut output[..OUT_LEN]).try_into().unwrap(),
        );
        if increment_counter.yes() {
            counter += 1;
        }
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn set4(a: u32, b: u32, c: u32, d: u32) -> uint32x4_t {
    let array = [a, b, c, d];
    // SAFETY: The local array is valid for the full 16-byte vector load.
    unsafe { vld1q_u32(array.as_ptr()) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash_many() {
        crate::test::test_hash_many_fn(
            hash_many::<{ crate::CHUNK_LEN }>,
            hash_many::<{ 2 * OUT_LEN }>,
        );
    }
}
