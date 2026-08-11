#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::{BLOCK_LEN, CVWords, IV, MSG_SCHEDULE, counter_high, counter_low};
#[cfg(blake3_avx2_rust)]
use crate::{IncrementCounter, OUT_LEN};
#[cfg(blake3_avx2_rust)]
use arrayref::array_mut_ref;
use arrayref::mut_array_refs;

pub const DEGREE: usize = 8;

#[inline(always)]
unsafe fn loadu(src: *const u8) -> __m256i {
    // This is an unaligned load, so the pointer cast is allowed.
    unsafe { _mm256_loadu_si256(src as *const __m256i) }
}

#[inline(always)]
unsafe fn storeu(src: __m256i, dest: *mut u8) {
    // This is an unaligned store, so the pointer cast is allowed.
    unsafe { _mm256_storeu_si256(dest as *mut __m256i, src) }
}

#[inline(always)]
unsafe fn add(a: __m256i, b: __m256i) -> __m256i {
    unsafe { _mm256_add_epi32(a, b) }
}

#[inline(always)]
unsafe fn xor(a: __m256i, b: __m256i) -> __m256i {
    unsafe { _mm256_xor_si256(a, b) }
}

#[inline(always)]
unsafe fn set1(x: u32) -> __m256i {
    unsafe { _mm256_set1_epi32(x as i32) }
}

#[cfg(blake3_avx2_rust)]
#[inline(always)]
unsafe fn set8(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> __m256i {
    unsafe {
        _mm256_setr_epi32(
            a as i32, b as i32, c as i32, d as i32, e as i32, f as i32, g as i32, h as i32,
        )
    }
}

// These rotations are the "simple/shifts version". For the
// "complicated/shuffles version", see
// https://github.com/sneves/blake2-avx2/blob/b3723921f668df09ece52dcd225a36d4a4eea1d9/blake2s-common.h#L63-L66.
// For a discussion of the tradeoffs, see
// https://github.com/sneves/blake2-avx2/pull/5. Due to an LLVM bug
// (https://bugs.llvm.org/show_bug.cgi?id=44379), this version performs better
// on recent x86 chips.

#[inline(always)]
unsafe fn rot16(x: __m256i) -> __m256i {
    unsafe { _mm256_or_si256(_mm256_srli_epi32(x, 16), _mm256_slli_epi32(x, 32 - 16)) }
}

#[inline(always)]
unsafe fn rot12(x: __m256i) -> __m256i {
    unsafe { _mm256_or_si256(_mm256_srli_epi32(x, 12), _mm256_slli_epi32(x, 32 - 12)) }
}

#[inline(always)]
unsafe fn rot8(x: __m256i) -> __m256i {
    unsafe { _mm256_or_si256(_mm256_srli_epi32(x, 8), _mm256_slli_epi32(x, 32 - 8)) }
}

#[inline(always)]
unsafe fn rot7(x: __m256i) -> __m256i {
    unsafe { _mm256_or_si256(_mm256_srli_epi32(x, 7), _mm256_slli_epi32(x, 32 - 7)) }
}

#[inline(always)]
unsafe fn round(v: &mut [__m256i; 16], m: &[__m256i; 16], r: usize) {
    unsafe {
        v[0] = add(v[0], m[MSG_SCHEDULE[r][0] as usize]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][2] as usize]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][4] as usize]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][6] as usize]);
        v[0] = add(v[0], v[4]);
        v[1] = add(v[1], v[5]);
        v[2] = add(v[2], v[6]);
        v[3] = add(v[3], v[7]);
        v[12] = xor(v[12], v[0]);
        v[13] = xor(v[13], v[1]);
        v[14] = xor(v[14], v[2]);
        v[15] = xor(v[15], v[3]);
        v[12] = rot16(v[12]);
        v[13] = rot16(v[13]);
        v[14] = rot16(v[14]);
        v[15] = rot16(v[15]);
        v[8] = add(v[8], v[12]);
        v[9] = add(v[9], v[13]);
        v[10] = add(v[10], v[14]);
        v[11] = add(v[11], v[15]);
        v[4] = xor(v[4], v[8]);
        v[5] = xor(v[5], v[9]);
        v[6] = xor(v[6], v[10]);
        v[7] = xor(v[7], v[11]);
        v[4] = rot12(v[4]);
        v[5] = rot12(v[5]);
        v[6] = rot12(v[6]);
        v[7] = rot12(v[7]);
        v[0] = add(v[0], m[MSG_SCHEDULE[r][1] as usize]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][3] as usize]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][5] as usize]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][7] as usize]);
        v[0] = add(v[0], v[4]);
        v[1] = add(v[1], v[5]);
        v[2] = add(v[2], v[6]);
        v[3] = add(v[3], v[7]);
        v[12] = xor(v[12], v[0]);
        v[13] = xor(v[13], v[1]);
        v[14] = xor(v[14], v[2]);
        v[15] = xor(v[15], v[3]);
        v[12] = rot8(v[12]);
        v[13] = rot8(v[13]);
        v[14] = rot8(v[14]);
        v[15] = rot8(v[15]);
        v[8] = add(v[8], v[12]);
        v[9] = add(v[9], v[13]);
        v[10] = add(v[10], v[14]);
        v[11] = add(v[11], v[15]);
        v[4] = xor(v[4], v[8]);
        v[5] = xor(v[5], v[9]);
        v[6] = xor(v[6], v[10]);
        v[7] = xor(v[7], v[11]);
        v[4] = rot7(v[4]);
        v[5] = rot7(v[5]);
        v[6] = rot7(v[6]);
        v[7] = rot7(v[7]);

        v[0] = add(v[0], m[MSG_SCHEDULE[r][8] as usize]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][10] as usize]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][12] as usize]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][14] as usize]);
        v[0] = add(v[0], v[5]);
        v[1] = add(v[1], v[6]);
        v[2] = add(v[2], v[7]);
        v[3] = add(v[3], v[4]);
        v[15] = xor(v[15], v[0]);
        v[12] = xor(v[12], v[1]);
        v[13] = xor(v[13], v[2]);
        v[14] = xor(v[14], v[3]);
        v[15] = rot16(v[15]);
        v[12] = rot16(v[12]);
        v[13] = rot16(v[13]);
        v[14] = rot16(v[14]);
        v[10] = add(v[10], v[15]);
        v[11] = add(v[11], v[12]);
        v[8] = add(v[8], v[13]);
        v[9] = add(v[9], v[14]);
        v[5] = xor(v[5], v[10]);
        v[6] = xor(v[6], v[11]);
        v[7] = xor(v[7], v[8]);
        v[4] = xor(v[4], v[9]);
        v[5] = rot12(v[5]);
        v[6] = rot12(v[6]);
        v[7] = rot12(v[7]);
        v[4] = rot12(v[4]);
        v[0] = add(v[0], m[MSG_SCHEDULE[r][9] as usize]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][11] as usize]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][13] as usize]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][15] as usize]);
        v[0] = add(v[0], v[5]);
        v[1] = add(v[1], v[6]);
        v[2] = add(v[2], v[7]);
        v[3] = add(v[3], v[4]);
        v[15] = xor(v[15], v[0]);
        v[12] = xor(v[12], v[1]);
        v[13] = xor(v[13], v[2]);
        v[14] = xor(v[14], v[3]);
        v[15] = rot8(v[15]);
        v[12] = rot8(v[12]);
        v[13] = rot8(v[13]);
        v[14] = rot8(v[14]);
        v[10] = add(v[10], v[15]);
        v[11] = add(v[11], v[12]);
        v[8] = add(v[8], v[13]);
        v[9] = add(v[9], v[14]);
        v[5] = xor(v[5], v[10]);
        v[6] = xor(v[6], v[11]);
        v[7] = xor(v[7], v[8]);
        v[4] = xor(v[4], v[9]);
        v[5] = rot7(v[5]);
        v[6] = rot7(v[6]);
        v[7] = rot7(v[7]);
        v[4] = rot7(v[4]);
    }
}

#[inline(always)]
unsafe fn interleave128(a: __m256i, b: __m256i) -> (__m256i, __m256i) {
    unsafe {
        (
            _mm256_permute2x128_si256(a, b, 0x20),
            _mm256_permute2x128_si256(a, b, 0x31),
        )
    }
}

// There are several ways to do a transposition. We could do it naively, with 8 separate
// _mm256_set_epi32 instructions, referencing each of the 32 words explicitly. Or we could copy
// the vecs into contiguous storage and then use gather instructions. This third approach is to use
// a series of unpack instructions to interleave the vectors. In my benchmarks, interleaving is the
// fastest approach. To test this, run `cargo +nightly bench --bench libtest load_8` in the
// https://github.com/oconnor663/bao_experiments repo.
#[inline(always)]
unsafe fn transpose_vecs(vecs: &mut [__m256i; DEGREE]) {
    unsafe {
        // Interleave 32-bit lanes. The low unpack is lanes 00/11/44/55, and the high is 22/33/66/77.
        let ab_0145 = _mm256_unpacklo_epi32(vecs[0], vecs[1]);
        let ab_2367 = _mm256_unpackhi_epi32(vecs[0], vecs[1]);
        let cd_0145 = _mm256_unpacklo_epi32(vecs[2], vecs[3]);
        let cd_2367 = _mm256_unpackhi_epi32(vecs[2], vecs[3]);
        let ef_0145 = _mm256_unpacklo_epi32(vecs[4], vecs[5]);
        let ef_2367 = _mm256_unpackhi_epi32(vecs[4], vecs[5]);
        let gh_0145 = _mm256_unpacklo_epi32(vecs[6], vecs[7]);
        let gh_2367 = _mm256_unpackhi_epi32(vecs[6], vecs[7]);

        // Interleave 64-bit lanes. The low unpack is lanes 00/22 and the high is 11/33.
        let abcd_04 = _mm256_unpacklo_epi64(ab_0145, cd_0145);
        let abcd_15 = _mm256_unpackhi_epi64(ab_0145, cd_0145);
        let abcd_26 = _mm256_unpacklo_epi64(ab_2367, cd_2367);
        let abcd_37 = _mm256_unpackhi_epi64(ab_2367, cd_2367);
        let efgh_04 = _mm256_unpacklo_epi64(ef_0145, gh_0145);
        let efgh_15 = _mm256_unpackhi_epi64(ef_0145, gh_0145);
        let efgh_26 = _mm256_unpacklo_epi64(ef_2367, gh_2367);
        let efgh_37 = _mm256_unpackhi_epi64(ef_2367, gh_2367);

        // Interleave 128-bit lanes.
        let (abcdefgh_0, abcdefgh_4) = interleave128(abcd_04, efgh_04);
        let (abcdefgh_1, abcdefgh_5) = interleave128(abcd_15, efgh_15);
        let (abcdefgh_2, abcdefgh_6) = interleave128(abcd_26, efgh_26);
        let (abcdefgh_3, abcdefgh_7) = interleave128(abcd_37, efgh_37);

        vecs[0] = abcdefgh_0;
        vecs[1] = abcdefgh_1;
        vecs[2] = abcdefgh_2;
        vecs[3] = abcdefgh_3;
        vecs[4] = abcdefgh_4;
        vecs[5] = abcdefgh_5;
        vecs[6] = abcdefgh_6;
        vecs[7] = abcdefgh_7;
    }
}

#[inline(always)]
unsafe fn transpose_msg_vecs(inputs: &[*const u8; DEGREE], block_offset: usize) -> [__m256i; 16] {
    unsafe {
        let mut vecs = [
            loadu(inputs[0].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[1].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[2].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[3].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[4].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[5].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[6].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[7].add(block_offset + 0 * 4 * DEGREE)),
            loadu(inputs[0].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[1].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[2].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[3].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[4].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[5].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[6].add(block_offset + 1 * 4 * DEGREE)),
            loadu(inputs[7].add(block_offset + 1 * 4 * DEGREE)),
        ];
        for i in 0..DEGREE {
            _mm_prefetch(
                inputs[i].wrapping_add(block_offset + 256) as *const i8,
                _MM_HINT_T0,
            );
        }
        let squares = mut_array_refs!(&mut vecs, DEGREE, DEGREE);
        transpose_vecs(squares.0);
        transpose_vecs(squares.1);
        vecs
    }
}

#[cfg(blake3_avx2_rust)]
#[inline(always)]
unsafe fn load_counters(counter: u64, increment_counter: IncrementCounter) -> (__m256i, __m256i) {
    let mask = if increment_counter.yes() { !0 } else { 0 };
    unsafe {
        (
            set8(
                counter_low(counter + (mask & 0)),
                counter_low(counter + (mask & 1)),
                counter_low(counter + (mask & 2)),
                counter_low(counter + (mask & 3)),
                counter_low(counter + (mask & 4)),
                counter_low(counter + (mask & 5)),
                counter_low(counter + (mask & 6)),
                counter_low(counter + (mask & 7)),
            ),
            set8(
                counter_high(counter + (mask & 0)),
                counter_high(counter + (mask & 1)),
                counter_high(counter + (mask & 2)),
                counter_high(counter + (mask & 3)),
                counter_high(counter + (mask & 4)),
                counter_high(counter + (mask & 5)),
                counter_high(counter + (mask & 6)),
                counter_high(counter + (mask & 7)),
            ),
        )
    }
}

#[cfg(blake3_avx2_rust)]
#[target_feature(enable = "avx2")]
pub unsafe fn hash8(
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
    unsafe {
        let mut h_vecs = [
            set1(key[0]),
            set1(key[1]),
            set1(key[2]),
            set1(key[3]),
            set1(key[4]),
            set1(key[5]),
            set1(key[6]),
            set1(key[7]),
        ];
        let (counter_low_vec, counter_high_vec) = load_counters(counter, increment_counter);
        let mut block_flags = flags | flags_start;

        for block in 0..blocks {
            if block + 1 == blocks {
                block_flags |= flags_end;
            }
            let block_len_vec = set1(BLOCK_LEN as u32); // full blocks only
            let block_flags_vec = set1(block_flags as u32);
            let msg_vecs = transpose_msg_vecs(inputs, block * BLOCK_LEN);

            // The transposed compression function. Note that inlining this
            // manually here improves compile times by a lot, compared to factoring
            // it out into its own function and making it #[inline(always)]. Just
            // guessing, it might have something to do with loop unrolling.
            let mut v = [
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
            ];
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

            block_flags = flags;
        }

        transpose_vecs(&mut h_vecs);
        storeu(h_vecs[0], out.as_mut_ptr().add(0 * 4 * DEGREE));
        storeu(h_vecs[1], out.as_mut_ptr().add(1 * 4 * DEGREE));
        storeu(h_vecs[2], out.as_mut_ptr().add(2 * 4 * DEGREE));
        storeu(h_vecs[3], out.as_mut_ptr().add(3 * 4 * DEGREE));
        storeu(h_vecs[4], out.as_mut_ptr().add(4 * 4 * DEGREE));
        storeu(h_vecs[5], out.as_mut_ptr().add(5 * 4 * DEGREE));
        storeu(h_vecs[6], out.as_mut_ptr().add(6 * 4 * DEGREE));
        storeu(h_vecs[7], out.as_mut_ptr().add(7 * 4 * DEGREE));
    }
}

#[cfg(blake3_avx2_rust)]
#[target_feature(enable = "avx2")]
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
    while inputs.len() >= DEGREE && out.len() >= DEGREE * OUT_LEN {
        // Safe because the layout of arrays is guaranteed, and because the
        // `blocks` count is determined statically from the argument type.
        let input_ptrs: &[*const u8; DEGREE] =
            unsafe { &*(inputs.as_ptr() as *const [*const u8; DEGREE]) };
        let blocks = N / BLOCK_LEN;
        unsafe {
            hash8(
                input_ptrs,
                blocks,
                key,
                counter,
                increment_counter,
                flags,
                flags_start,
                flags_end,
                array_mut_ref!(out, 0, DEGREE * OUT_LEN),
            );
        }
        if increment_counter.yes() {
            counter += DEGREE as u64;
        }
        inputs = &inputs[DEGREE..];
        out = &mut out[DEGREE * OUT_LEN..];
    }
    unsafe {
        crate::sse41::hash_many(
            inputs,
            key,
            counter,
            increment_counter,
            flags,
            flags_start,
            flags_end,
            out,
        );
    }
}

/// Compresses one (possibly partial) block for up to [`DEGREE`] (8) independent lanes at once.
/// `cvs[i]` is lane `i`'s incoming chaining value, overwritten in place with the compression
/// result, and `blocks[i]` points to at least `BLOCK_LEN` readable bytes for lane `i`. The
/// `block_len`, `counter`, and `flags` are shared by every lane.
#[target_feature(enable = "avx2")]
pub unsafe fn compress8(
    cvs: &mut [CVWords; DEGREE],
    blocks: &[[u8; BLOCK_LEN]; DEGREE],
    block_len: u8,
    counter: u64,
    flags: u8,
) {
    unsafe {
        let mut h_vecs: [__m256i; DEGREE] =
            core::array::from_fn(|lane| loadu(cvs[lane].as_ptr() as *const u8));
        transpose_vecs(&mut h_vecs);

        let block_ptrs: [*const u8; DEGREE] = core::array::from_fn(|lane| blocks[lane].as_ptr());
        let msg_vecs = transpose_msg_vecs(&block_ptrs, 0);

        let counter_low_vec = set1(counter_low(counter));
        let counter_high_vec = set1(counter_high(counter));
        let block_len_vec = set1(block_len as u32);
        let block_flags_vec = set1(flags as u32);

        let mut v = [
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
        ];
        round(&mut v, &msg_vecs, 0);
        round(&mut v, &msg_vecs, 1);
        round(&mut v, &msg_vecs, 2);
        round(&mut v, &msg_vecs, 3);
        round(&mut v, &msg_vecs, 4);
        round(&mut v, &msg_vecs, 5);
        round(&mut v, &msg_vecs, 6);

        let mut h_vecs_out = [
            xor(v[0], v[8]),
            xor(v[1], v[9]),
            xor(v[2], v[10]),
            xor(v[3], v[11]),
            xor(v[4], v[12]),
            xor(v[5], v[13]),
            xor(v[6], v[14]),
            xor(v[7], v[15]),
        ];
        transpose_vecs(&mut h_vecs_out);
        for lane in 0..DEGREE {
            storeu(h_vecs_out[lane], cvs[lane].as_mut_ptr() as *mut u8);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transpose() {
        if !crate::platform::avx2_detected() {
            return;
        }

        #[target_feature(enable = "avx2")]
        unsafe fn transpose_wrapper(vecs: &mut [__m256i; DEGREE]) {
            unsafe { transpose_vecs(vecs) };
        }

        let mut matrix = [[0 as u32; DEGREE]; DEGREE];
        for i in 0..DEGREE {
            for j in 0..DEGREE {
                matrix[i][j] = (i * DEGREE + j) as u32;
            }
        }

        unsafe {
            let mut vecs: [__m256i; DEGREE] = core::mem::transmute(matrix);
            transpose_wrapper(&mut vecs);
            matrix = core::mem::transmute(vecs);
        }

        for i in 0..DEGREE {
            for j in 0..DEGREE {
                // Reversed indexes from above.
                assert_eq!(matrix[j][i], (i * DEGREE + j) as u32);
            }
        }
    }

    #[cfg(blake3_avx2_rust)]
    #[test]
    fn test_hash_many() {
        if !crate::platform::avx2_detected() {
            return;
        }
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }

    #[test]
    fn test_compress8_matches_compress_in_place() {
        if !crate::platform::avx2_detected() {
            return;
        }
        // Give each lane a different slice of the painted buffer, so a lane mixup fails the test.
        let mut input_buf = [0u8; DEGREE * BLOCK_LEN];
        crate::test::paint_test_input(&mut input_buf);

        for block_len in [0u8, 1, 17, 32, 63, 64] {
            // See test_hash_many_fn in src/test.rs for why these particular counters matter.
            for counter in [0u64, u32::MAX as u64, i32::MAX as u64] {
                for flags in [
                    crate::CHUNK_START | crate::CHUNK_END | crate::ROOT,
                    crate::KEYED_HASH | crate::CHUNK_START | crate::CHUNK_END,
                ] {
                    let mut cvs = [[0u32; 8]; DEGREE];
                    let mut blocks = [[0u8; BLOCK_LEN]; DEGREE];
                    for lane in 0..DEGREE {
                        // A rotation of TEST_KEY_WORDS, distinct for every lane.
                        for w in 0..8 {
                            cvs[lane][w] = crate::test::TEST_KEY_WORDS[(w + lane) % 8];
                        }
                        blocks[lane].copy_from_slice(&input_buf[lane * BLOCK_LEN..][..BLOCK_LEN]);
                    }

                    let mut want_cvs = cvs;
                    for lane in 0..DEGREE {
                        crate::portable::compress_in_place(
                            &mut want_cvs[lane],
                            &blocks[lane],
                            block_len,
                            counter,
                            flags,
                        );
                    }

                    unsafe {
                        compress8(&mut cvs, &blocks, block_len, counter, flags);
                    }

                    assert_eq!(
                        cvs, want_cvs,
                        "block_len {block_len} counter {counter} flags {flags}"
                    );
                }
            }
        }
    }
}
