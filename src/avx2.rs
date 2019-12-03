#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::{offset_high, offset_low, BLOCK_LEN, IV, MSG_SCHEDULE, OUT_LEN};
use arrayref::{array_mut_ref, mut_array_refs};

pub const DEGREE: usize = 8;

#[inline(always)]
unsafe fn loadu(src: *const u8) -> __m256i {
    // This is an unaligned load, so the pointer cast is allowed.
    _mm256_loadu_si256(src as *const __m256i)
}

#[inline(always)]
unsafe fn storeu(src: __m256i, dest: *mut u8) {
    // This is an unaligned store, so the pointer cast is allowed.
    _mm256_storeu_si256(dest as *mut __m256i, src)
}

#[inline(always)]
unsafe fn add(a: __m256i, b: __m256i) -> __m256i {
    _mm256_add_epi32(a, b)
}

#[inline(always)]
unsafe fn xor(a: __m256i, b: __m256i) -> __m256i {
    _mm256_xor_si256(a, b)
}

#[inline(always)]
unsafe fn set1(x: u32) -> __m256i {
    _mm256_set1_epi32(x as i32)
}

#[inline(always)]
unsafe fn set8(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> __m256i {
    _mm256_setr_epi32(
        a as i32, b as i32, c as i32, d as i32, e as i32, f as i32, g as i32, h as i32,
    )
}

#[inline(always)]
unsafe fn rot16(x: __m256i) -> __m256i {
    _mm256_shuffle_epi8(
        x,
        _mm256_set_epi8(
            13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2, 13, 12, 15, 14, 9, 8, 11, 10, 5,
            4, 7, 6, 1, 0, 3, 2,
        ),
    )
}

#[inline(always)]
unsafe fn rot12(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi32(x, 12), _mm256_slli_epi32(x, 20))
}

#[inline(always)]
unsafe fn rot8(x: __m256i) -> __m256i {
    _mm256_shuffle_epi8(
        x,
        _mm256_set_epi8(
            12, 15, 14, 13, 8, 11, 10, 9, 4, 7, 6, 5, 0, 3, 2, 1, 12, 15, 14, 13, 8, 11, 10, 9, 4,
            7, 6, 5, 0, 3, 2, 1,
        ),
    )
}

#[inline(always)]
unsafe fn rot7(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi32(x, 7), _mm256_slli_epi32(x, 25))
}

#[inline(always)]
unsafe fn round(v: &mut [__m256i; 16], m: &[__m256i; 16], r: usize) {
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

#[inline(always)]
unsafe fn interleave128(a: __m256i, b: __m256i) -> (__m256i, __m256i) {
    (
        _mm256_permute2x128_si256(a, b, 0x20),
        _mm256_permute2x128_si256(a, b, 0x31),
    )
}

// There are several ways to do a transposition. We could do it naively, with 8 separate
// _mm256_set_epi32 instructions, referencing each of the 32 words explicitly. Or we could copy
// the vecs into contiguous storage and then use gather instructions. This third approach is to use
// a series of unpack instructions to interleave the vectors. In my benchmarks, interleaving is the
// fastest approach. To test this, run `cargo +nightly bench --bench libtest load_8` in the
// https://github.com/oconnor663/bao_experiments repo.
#[inline(always)]
unsafe fn transpose_vecs(vecs: &mut [__m256i; DEGREE]) {
    // Interleave 32-bit lanes. The low unpack is lanes 00/11/44/55, and the high is 22/33/66/77.
    let ab_0145 = _mm256_unpacklo_epi32(vecs[0], vecs[1]);
    let ab_2367 = _mm256_unpackhi_epi32(vecs[0], vecs[1]);
    let cd_0145 = _mm256_unpacklo_epi32(vecs[2], vecs[3]);
    let cd_2367 = _mm256_unpackhi_epi32(vecs[2], vecs[3]);
    let ef_0145 = _mm256_unpacklo_epi32(vecs[4], vecs[5]);
    let ef_2367 = _mm256_unpackhi_epi32(vecs[4], vecs[5]);
    let gh_0145 = _mm256_unpacklo_epi32(vecs[6], vecs[7]);
    let gh_2367 = _mm256_unpackhi_epi32(vecs[6], vecs[7]);

    // Interleave 64-bit lates. The low unpack is lanes 00/22 and the high is 11/33.
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

#[inline(always)]
unsafe fn transpose_msg_vecs(inputs: &[*const u8; DEGREE], block_offset: usize) -> [__m256i; 16] {
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
    let squares = mut_array_refs!(&mut vecs, DEGREE, DEGREE);
    transpose_vecs(squares.0);
    transpose_vecs(squares.1);
    vecs
}

#[inline(always)]
unsafe fn load_offsets(offset: u64, offset_deltas: &[u64; 16]) -> (__m256i, __m256i) {
    (
        set8(
            offset_low(offset + offset_deltas[0]),
            offset_low(offset + offset_deltas[1]),
            offset_low(offset + offset_deltas[2]),
            offset_low(offset + offset_deltas[3]),
            offset_low(offset + offset_deltas[4]),
            offset_low(offset + offset_deltas[5]),
            offset_low(offset + offset_deltas[6]),
            offset_low(offset + offset_deltas[7]),
        ),
        set8(
            offset_high(offset + offset_deltas[0]),
            offset_high(offset + offset_deltas[1]),
            offset_high(offset + offset_deltas[2]),
            offset_high(offset + offset_deltas[3]),
            offset_high(offset + offset_deltas[4]),
            offset_high(offset + offset_deltas[5]),
            offset_high(offset + offset_deltas[6]),
            offset_high(offset + offset_deltas[7]),
        ),
    )
}

#[target_feature(enable = "avx2")]
pub unsafe fn hash8(
    inputs: &[*const u8; DEGREE],
    blocks: usize,
    key: &[u32; 8],
    offset: u64,
    offset_deltas: &[u64; 16],
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8; DEGREE * OUT_LEN],
) {
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
    let (offset_low_vec, offset_high_vec) = load_offsets(offset, offset_deltas);
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
            offset_low_vec,
            offset_high_vec,
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

#[target_feature(enable = "avx2")]
pub unsafe fn hash_many<A: arrayvec::Array<Item = u8>>(
    mut inputs: &[&A],
    key: &[u32; 8],
    mut offset: u64,
    offset_deltas: &[u64; 16],
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    mut out: &mut [u8],
) {
    debug_assert!(out.len() >= inputs.len() * OUT_LEN, "out too short");
    while inputs.len() >= DEGREE && out.len() >= DEGREE * OUT_LEN {
        // Safe because the layout of arrays is guaranteed, and because the
        // `blocks` count is determined statically from the argument type.
        let input_ptrs: &[*const u8; DEGREE] = &*(inputs.as_ptr() as *const [*const u8; DEGREE]);
        let blocks = A::CAPACITY / BLOCK_LEN;
        hash8(
            input_ptrs,
            blocks,
            key,
            offset,
            offset_deltas,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(out, 0, DEGREE * OUT_LEN),
        );
        inputs = &inputs[DEGREE..];
        offset += DEGREE as u64 * offset_deltas[1];
        out = &mut out[DEGREE * OUT_LEN..];
    }
    crate::sse41::hash_many(
        inputs,
        key,
        offset,
        offset_deltas,
        flags,
        flags_start,
        flags_end,
        out,
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use arrayref::array_ref;

    #[test]
    fn test_transpose() {
        if !crate::platform::avx2_detected() {
            return;
        }

        #[target_feature(enable = "avx2")]
        unsafe fn transpose_wrapper(vecs: &mut [__m256i; DEGREE]) {
            transpose_vecs(vecs);
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

    #[test]
    fn test_parents() {
        if !crate::platform::avx2_detected() {
            return;
        }

        let mut input = [0; DEGREE * BLOCK_LEN];
        crate::test::paint_test_input(&mut input);
        let parents = [
            array_ref!(input, 0 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 1 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 2 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 3 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 4 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 5 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 6 * BLOCK_LEN, BLOCK_LEN),
            array_ref!(input, 7 * BLOCK_LEN, BLOCK_LEN),
        ];
        let key = [99, 98, 97, 96, 95, 94, 93, 92];

        let mut portable_out = [0; DEGREE * OUT_LEN];
        for (parent, out) in parents.iter().zip(portable_out.chunks_exact_mut(OUT_LEN)) {
            let out_words =
                portable::compress(&key, parent, BLOCK_LEN as u8, 0, Flags::PARENT.bits());
            out.copy_from_slice(&bytes_from_state_words(array_ref!(out_words, 0, 8)));
        }

        let mut simd_out = [0; DEGREE * OUT_LEN];
        let inputs = [
            parents[0].as_ptr(),
            parents[1].as_ptr(),
            parents[2].as_ptr(),
            parents[3].as_ptr(),
            parents[4].as_ptr(),
            parents[5].as_ptr(),
            parents[6].as_ptr(),
            parents[7].as_ptr(),
        ];
        unsafe {
            hash8(
                &inputs,
                1,
                &key,
                0,
                PARENT_OFFSET_DELTAS,
                0,
                Flags::PARENT.bits(),
                0,
                &mut simd_out,
            );
        }

        assert_eq!(&portable_out[..], &simd_out[..]);
    }

    #[test]
    fn test_chunks() {
        if !crate::platform::avx2_detected() {
            return;
        }

        let mut input = [0; DEGREE * CHUNK_LEN];
        crate::test::paint_test_input(&mut input);
        let chunks = [
            array_ref!(input, 0 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 1 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 2 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 3 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 4 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 5 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 6 * CHUNK_LEN, CHUNK_LEN),
            array_ref!(input, 7 * CHUNK_LEN, CHUNK_LEN),
        ];
        let key = [108, 107, 106, 105, 104, 103, 102, 101];
        // Use an offset with set bits in both 32-bit words.
        let initial_offset = ((5 * CHUNK_LEN as u64) << 32) + 6 * CHUNK_LEN as u64;

        let mut portable_out = [0; DEGREE * OUT_LEN];
        for ((chunk_index, chunk), out) in chunks
            .iter()
            .enumerate()
            .zip(portable_out.chunks_exact_mut(OUT_LEN))
        {
            let mut cv = key;
            for (block_index, block) in chunk.chunks_exact(BLOCK_LEN).enumerate() {
                let mut flags = Flags::KEYED_HASH;
                if block_index == 0 {
                    flags |= Flags::CHUNK_START;
                }
                if block_index == CHUNK_LEN / BLOCK_LEN - 1 {
                    flags |= Flags::CHUNK_END;
                }
                let out = portable::compress(
                    &cv,
                    array_ref!(block, 0, BLOCK_LEN),
                    BLOCK_LEN as u8,
                    initial_offset + (chunk_index * CHUNK_LEN) as u64,
                    flags.bits(),
                );
                cv = *array_ref!(out, 0, 8);
            }
            out.copy_from_slice(&bytes_from_state_words(&cv));
        }

        let mut simd_out = [0; DEGREE * OUT_LEN];
        let inputs = [
            chunks[0].as_ptr(),
            chunks[1].as_ptr(),
            chunks[2].as_ptr(),
            chunks[3].as_ptr(),
            chunks[4].as_ptr(),
            chunks[5].as_ptr(),
            chunks[6].as_ptr(),
            chunks[7].as_ptr(),
        ];
        unsafe {
            hash8(
                &inputs,
                CHUNK_LEN / BLOCK_LEN,
                &key,
                initial_offset,
                CHUNK_OFFSET_DELTAS,
                Flags::KEYED_HASH.bits(),
                Flags::CHUNK_START.bits(),
                Flags::CHUNK_END.bits(),
                &mut simd_out,
            );
        }

        assert_eq!(&portable_out[..], &simd_out[..]);
    }

    #[test]
    fn test_hash_many() {
        if !crate::platform::avx2_detected() {
            return;
        }

        // 31 = 16 + 8 + 4 + 2 + 1
        const INPUT_LEN: usize = 3 * BLOCK_LEN;
        const NUM_INPUTS: usize = 31;
        let mut input_buf = [0; NUM_INPUTS * INPUT_LEN];
        crate::test::paint_test_input(&mut input_buf);
        let mut inputs = arrayvec::ArrayVec::<[&[u8; INPUT_LEN]; NUM_INPUTS]>::new();
        for i in 0..NUM_INPUTS {
            inputs.push(array_ref!(input_buf, i * INPUT_LEN, INPUT_LEN));
        }
        let key = [2; 8];
        let offset = 3 * CHUNK_LEN as u64;
        let flags = 4;
        let flags_start = 8;
        let flags_end = 16;

        let mut portable_out = [0; OUT_LEN * NUM_INPUTS];
        portable::hash_many(
            &inputs,
            &key,
            offset,
            CHUNK_OFFSET_DELTAS,
            flags,
            flags_start,
            flags_end,
            &mut portable_out,
        );

        let mut test_out = [0; OUT_LEN * NUM_INPUTS];
        unsafe {
            hash_many(
                &inputs,
                &key,
                offset,
                CHUNK_OFFSET_DELTAS,
                flags,
                flags_start,
                flags_end,
                &mut test_out,
            );
        }

        assert_eq!(&portable_out[..], &test_out[..]);
    }
}
