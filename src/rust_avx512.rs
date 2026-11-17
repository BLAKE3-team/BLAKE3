use core::arch::x86_64::*;

use crate::{
    BLOCK_LEN, CVWords, IV, IncrementCounter, MSG_SCHEDULE, OUT_LEN, counter_high, counter_low,
};
use arrayref::array_mut_ref;

pub const DEGREE: usize = 16;

#[inline(always)]
unsafe fn loadu_128(src: *const u8) -> __m128i {
    // This is an unaligned load, so the pointer cast is allowed.
    unsafe { _mm_loadu_si128(src as *const __m128i) }
}

#[inline(always)]
unsafe fn storeu_128(src: __m128i, dest: *mut u8) {
    // This is an unaligned store, so the pointer cast is allowed.
    unsafe { _mm_storeu_si128(dest as *mut __m128i, src) }
}

#[inline(always)]
unsafe fn add_128(a: __m128i, b: __m128i) -> __m128i {
    unsafe { _mm_add_epi32(a, b) }
}

#[inline(always)]
unsafe fn xor_128(a: __m128i, b: __m128i) -> __m128i {
    unsafe { _mm_xor_si128(a, b) }
}

#[inline(always)]
unsafe fn set4(a: u32, b: u32, c: u32, d: u32) -> __m128i {
    unsafe { _mm_setr_epi32(a as i32, b as i32, c as i32, d as i32) }
}

#[inline(always)]
unsafe fn rot16_128(x: __m128i) -> __m128i {
    unsafe { _mm_ror_epi32::<16>(x) }
}

#[inline(always)]
unsafe fn rot12_128(x: __m128i) -> __m128i {
    unsafe { _mm_ror_epi32::<12>(x) }
}

#[inline(always)]
unsafe fn rot8_128(x: __m128i) -> __m128i {
    unsafe { _mm_ror_epi32::<8>(x) }
}

#[inline(always)]
unsafe fn rot7_128(x: __m128i) -> __m128i {
    unsafe { _mm_ror_epi32::<7>(x) }
}

#[inline(always)]
unsafe fn storeu_512(src: __m512i, dest: *mut u8) {
    // This is an unaligned store, so the pointer cast is allowed.
    unsafe { _mm512_storeu_si512(dest as *mut __m512i, src) }
}

#[inline(always)]
unsafe fn add_512(a: __m512i, b: __m512i) -> __m512i {
    unsafe { _mm512_add_epi32(a, b) }
}

#[inline(always)]
unsafe fn xor_512(a: __m512i, b: __m512i) -> __m512i {
    unsafe { _mm512_xor_si512(a, b) }
}

#[inline(always)]
unsafe fn set1_512(x: u32) -> __m512i {
    unsafe { _mm512_set1_epi32(x as i32) }
}

#[inline(always)]
unsafe fn set16(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
    i: u32,
    j: u32,
    k: u32,
    l: u32,
    m: u32,
    n: u32,
    o: u32,
    p: u32,
) -> __m512i {
    unsafe {
        _mm512_setr_epi32(
            a as i32, b as i32, c as i32, d as i32, e as i32, f as i32, g as i32, h as i32,
            i as i32, j as i32, k as i32, l as i32, m as i32, n as i32, o as i32, p as i32,
        )
    }
}

#[inline(always)]
unsafe fn rot16_512(x: __m512i) -> __m512i {
    unsafe { _mm512_ror_epi32::<16>(x) }
}

#[inline(always)]
unsafe fn rot12_512(x: __m512i) -> __m512i {
    unsafe { _mm512_ror_epi32::<12>(x) }
}

#[inline(always)]
unsafe fn rot8_512(x: __m512i) -> __m512i {
    unsafe { _mm512_ror_epi32::<8>(x) }
}

#[inline(always)]
unsafe fn rot7_512(x: __m512i) -> __m512i {
    unsafe { _mm512_ror_epi32::<7>(x) }
}

#[inline(always)]
unsafe fn g1(
    row0: &mut __m128i,
    row1: &mut __m128i,
    row2: &mut __m128i,
    row3: &mut __m128i,
    m: __m128i,
) {
    unsafe {
        *row0 = add_128(add_128(*row0, m), *row1);
        *row3 = xor_128(*row3, *row0);
        *row3 = rot16_128(*row3);
        *row2 = add_128(*row2, *row3);
        *row1 = xor_128(*row1, *row2);
        *row1 = rot12_128(*row1);
    }
}

#[inline(always)]
unsafe fn g2(
    row0: &mut __m128i,
    row1: &mut __m128i,
    row2: &mut __m128i,
    row3: &mut __m128i,
    m: __m128i,
) {
    unsafe {
        *row0 = add_128(add_128(*row0, m), *row1);
        *row3 = xor_128(*row3, *row0);
        *row3 = rot8_128(*row3);
        *row2 = add_128(*row2, *row3);
        *row1 = xor_128(*row1, *row2);
        *row1 = rot7_128(*row1);
    }
}

// Adapted from https://github.com/rust-lang-nursery/stdsimd/pull/479.
macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

macro_rules! shuffle2 {
    ($a:expr, $b:expr, $c:expr) => {
        _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps($a),
            _mm_castsi128_ps($b),
            $c,
        ))
    };
}

macro_rules! shuffle2_512 {
    ($a:expr, $b:expr, $c:expr) => {
        _mm512_castps_si512(_mm512_shuffle_ps(
            _mm512_castsi512_ps($a),
            _mm512_castsi512_ps($b),
            $c,
        ))
    };
}

// Note the optimization here of leaving row1 as the unrotated row, rather than
// row0. All the message loads below are adjusted to compensate for this. See
// discussion at https://github.com/sneves/blake2-avx2/pull/4
#[inline(always)]
unsafe fn diagonalize(row0: &mut __m128i, row2: &mut __m128i, row3: &mut __m128i) {
    unsafe {
        *row0 = _mm_shuffle_epi32(*row0, _MM_SHUFFLE!(2, 1, 0, 3));
        *row3 = _mm_shuffle_epi32(*row3, _MM_SHUFFLE!(1, 0, 3, 2));
        *row2 = _mm_shuffle_epi32(*row2, _MM_SHUFFLE!(0, 3, 2, 1));
    }
}

#[inline(always)]
unsafe fn undiagonalize(row0: &mut __m128i, row2: &mut __m128i, row3: &mut __m128i) {
    unsafe {
        *row0 = _mm_shuffle_epi32(*row0, _MM_SHUFFLE!(0, 3, 2, 1));
        *row3 = _mm_shuffle_epi32(*row3, _MM_SHUFFLE!(1, 0, 3, 2));
        *row2 = _mm_shuffle_epi32(*row2, _MM_SHUFFLE!(2, 1, 0, 3));
    }
}

#[inline(always)]
unsafe fn compress_pre(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) -> [__m128i; 4] {
    unsafe {
        let row0 = &mut loadu_128(cv.as_ptr().add(0) as *const u8);
        let row1 = &mut loadu_128(cv.as_ptr().add(4) as *const u8);
        let row2 = &mut set4(IV[0], IV[1], IV[2], IV[3]);
        let row3 = &mut set4(
            counter_low(counter),
            counter_high(counter),
            block_len as u32,
            flags as u32,
        );

        let mut m0 = loadu_128(block.as_ptr().add(0 * 4 * 4));
        let mut m1 = loadu_128(block.as_ptr().add(1 * 4 * 4));
        let mut m2 = loadu_128(block.as_ptr().add(2 * 4 * 4));
        let mut m3 = loadu_128(block.as_ptr().add(3 * 4 * 4));

        let mut t0;
        let mut t1;
        let mut t2;
        let mut t3;
        let mut tt;

        // Round 1. The first round permutes the message words from the original
        // input order, into the groups that get mixed in parallel.
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(2, 0, 2, 0)); //  6  4  2  0
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 3, 1)); //  7  5  3  1
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = shuffle2!(m2, m3, _MM_SHUFFLE!(2, 0, 2, 0)); // 14 12 10  8
        t2 = _mm_shuffle_epi32(t2, _MM_SHUFFLE!(2, 1, 0, 3)); // 12 10  8 14
        g1(row0, row1, row2, row3, t2);
        t3 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 1, 3, 1)); // 15 13 11  9
        t3 = _mm_shuffle_epi32(t3, _MM_SHUFFLE!(2, 1, 0, 3)); // 13 11  9 15
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 2. This round and all following rounds apply a fixed permutation
        // to the message words from the round before.
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 3
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 4
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 5
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 6
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);
        m0 = t0;
        m1 = t1;
        m2 = t2;
        m3 = t3;

        // Round 7
        t0 = shuffle2!(m0, m1, _MM_SHUFFLE!(3, 1, 1, 2));
        t0 = _mm_shuffle_epi32(t0, _MM_SHUFFLE!(0, 3, 2, 1));
        g1(row0, row1, row2, row3, t0);
        t1 = shuffle2!(m2, m3, _MM_SHUFFLE!(3, 3, 2, 2));
        tt = _mm_shuffle_epi32(m0, _MM_SHUFFLE!(0, 0, 3, 3));
        t1 = _mm_blend_epi16(tt, t1, 0xCC);
        g2(row0, row1, row2, row3, t1);
        diagonalize(row0, row2, row3);
        t2 = _mm_unpacklo_epi64(m3, m1);
        tt = _mm_blend_epi16(t2, m2, 0xC0);
        t2 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(1, 3, 2, 0));
        g1(row0, row1, row2, row3, t2);
        t3 = _mm_unpackhi_epi32(m1, m3);
        tt = _mm_unpacklo_epi32(m2, t3);
        t3 = _mm_shuffle_epi32(tt, _MM_SHUFFLE!(0, 1, 3, 2));
        g2(row0, row1, row2, row3, t3);
        undiagonalize(row0, row2, row3);

        [*row0, *row1, *row2, *row3]
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn compress_in_place(
    cv: &mut CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) {
    unsafe {
        let [row0, row1, row2, row3] = compress_pre(cv, block, block_len, counter, flags);
        storeu_128(xor_128(row0, row2), cv.as_mut_ptr().add(0) as *mut u8);
        storeu_128(xor_128(row1, row3), cv.as_mut_ptr().add(4) as *mut u8);
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn compress_xof(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) -> [u8; 64] {
    unsafe {
        let [mut row0, mut row1, mut row2, mut row3] =
            compress_pre(cv, block, block_len, counter, flags);
        row0 = xor_128(row0, row2);
        row1 = xor_128(row1, row3);
        row2 = xor_128(row2, loadu_128(cv.as_ptr().add(0) as *const u8));
        row3 = xor_128(row3, loadu_128(cv.as_ptr().add(4) as *const u8));
        core::mem::transmute([row0, row1, row2, row3])
    }
}

#[inline(always)]
unsafe fn round(v: &mut [__m512i; 16], m: &[__m512i; 16], r: usize) {
    unsafe {
        v[0] = add_512(v[0], m[MSG_SCHEDULE[r][0] as usize]);
        v[1] = add_512(v[1], m[MSG_SCHEDULE[r][2] as usize]);
        v[2] = add_512(v[2], m[MSG_SCHEDULE[r][4] as usize]);
        v[3] = add_512(v[3], m[MSG_SCHEDULE[r][6] as usize]);
        v[0] = add_512(v[0], v[4]);
        v[1] = add_512(v[1], v[5]);
        v[2] = add_512(v[2], v[6]);
        v[3] = add_512(v[3], v[7]);
        v[12] = xor_512(v[12], v[0]);
        v[13] = xor_512(v[13], v[1]);
        v[14] = xor_512(v[14], v[2]);
        v[15] = xor_512(v[15], v[3]);
        v[12] = rot16_512(v[12]);
        v[13] = rot16_512(v[13]);
        v[14] = rot16_512(v[14]);
        v[15] = rot16_512(v[15]);
        v[8] = add_512(v[8], v[12]);
        v[9] = add_512(v[9], v[13]);
        v[10] = add_512(v[10], v[14]);
        v[11] = add_512(v[11], v[15]);
        v[4] = xor_512(v[4], v[8]);
        v[5] = xor_512(v[5], v[9]);
        v[6] = xor_512(v[6], v[10]);
        v[7] = xor_512(v[7], v[11]);
        v[4] = rot12_512(v[4]);
        v[5] = rot12_512(v[5]);
        v[6] = rot12_512(v[6]);
        v[7] = rot12_512(v[7]);
        v[0] = add_512(v[0], m[MSG_SCHEDULE[r][1] as usize]);
        v[1] = add_512(v[1], m[MSG_SCHEDULE[r][3] as usize]);
        v[2] = add_512(v[2], m[MSG_SCHEDULE[r][5] as usize]);
        v[3] = add_512(v[3], m[MSG_SCHEDULE[r][7] as usize]);
        v[0] = add_512(v[0], v[4]);
        v[1] = add_512(v[1], v[5]);
        v[2] = add_512(v[2], v[6]);
        v[3] = add_512(v[3], v[7]);
        v[12] = xor_512(v[12], v[0]);
        v[13] = xor_512(v[13], v[1]);
        v[14] = xor_512(v[14], v[2]);
        v[15] = xor_512(v[15], v[3]);
        v[12] = rot8_512(v[12]);
        v[13] = rot8_512(v[13]);
        v[14] = rot8_512(v[14]);
        v[15] = rot8_512(v[15]);
        v[8] = add_512(v[8], v[12]);
        v[9] = add_512(v[9], v[13]);
        v[10] = add_512(v[10], v[14]);
        v[11] = add_512(v[11], v[15]);
        v[4] = xor_512(v[4], v[8]);
        v[5] = xor_512(v[5], v[9]);
        v[6] = xor_512(v[6], v[10]);
        v[7] = xor_512(v[7], v[11]);
        v[4] = rot7_512(v[4]);
        v[5] = rot7_512(v[5]);
        v[6] = rot7_512(v[6]);
        v[7] = rot7_512(v[7]);

        v[0] = add_512(v[0], m[MSG_SCHEDULE[r][8] as usize]);
        v[1] = add_512(v[1], m[MSG_SCHEDULE[r][10] as usize]);
        v[2] = add_512(v[2], m[MSG_SCHEDULE[r][12] as usize]);
        v[3] = add_512(v[3], m[MSG_SCHEDULE[r][14] as usize]);
        v[0] = add_512(v[0], v[5]);
        v[1] = add_512(v[1], v[6]);
        v[2] = add_512(v[2], v[7]);
        v[3] = add_512(v[3], v[4]);
        v[15] = xor_512(v[15], v[0]);
        v[12] = xor_512(v[12], v[1]);
        v[13] = xor_512(v[13], v[2]);
        v[14] = xor_512(v[14], v[3]);
        v[15] = rot16_512(v[15]);
        v[12] = rot16_512(v[12]);
        v[13] = rot16_512(v[13]);
        v[14] = rot16_512(v[14]);
        v[10] = add_512(v[10], v[15]);
        v[11] = add_512(v[11], v[12]);
        v[8] = add_512(v[8], v[13]);
        v[9] = add_512(v[9], v[14]);
        v[5] = xor_512(v[5], v[10]);
        v[6] = xor_512(v[6], v[11]);
        v[7] = xor_512(v[7], v[8]);
        v[4] = xor_512(v[4], v[9]);
        v[5] = rot12_512(v[5]);
        v[6] = rot12_512(v[6]);
        v[7] = rot12_512(v[7]);
        v[4] = rot12_512(v[4]);
        v[0] = add_512(v[0], m[MSG_SCHEDULE[r][9] as usize]);
        v[1] = add_512(v[1], m[MSG_SCHEDULE[r][11] as usize]);
        v[2] = add_512(v[2], m[MSG_SCHEDULE[r][13] as usize]);
        v[3] = add_512(v[3], m[MSG_SCHEDULE[r][15] as usize]);
        v[0] = add_512(v[0], v[5]);
        v[1] = add_512(v[1], v[6]);
        v[2] = add_512(v[2], v[7]);
        v[3] = add_512(v[3], v[4]);
        v[15] = xor_512(v[15], v[0]);
        v[12] = xor_512(v[12], v[1]);
        v[13] = xor_512(v[13], v[2]);
        v[14] = xor_512(v[14], v[3]);
        v[15] = rot8_512(v[15]);
        v[12] = rot8_512(v[12]);
        v[13] = rot8_512(v[13]);
        v[14] = rot8_512(v[14]);
        v[10] = add_512(v[10], v[15]);
        v[11] = add_512(v[11], v[12]);
        v[8] = add_512(v[8], v[13]);
        v[9] = add_512(v[9], v[14]);
        v[5] = xor_512(v[5], v[10]);
        v[6] = xor_512(v[6], v[11]);
        v[7] = xor_512(v[7], v[8]);
        v[4] = xor_512(v[4], v[9]);
        v[5] = rot7_512(v[5]);
        v[6] = rot7_512(v[6]);
        v[7] = rot7_512(v[7]);
        v[4] = rot7_512(v[4]);
    }
}

// 0b10001000, or lanes a0/a2/b0/b2 in little-endian order
const LO_IMM8: i32 = 0x88;

#[inline(always)]
unsafe fn unpack_lo_128(a: __m512i, b: __m512i) -> __m512i {
    unsafe { _mm512_shuffle_i32x4::<LO_IMM8>(a, b) }
}

// 0b11011101, or lanes a1/a3/b1/b3 in little-endian order
const HI_IMM8: i32 = 0xdd;

#[inline(always)]
unsafe fn unpack_hi_128(a: __m512i, b: __m512i) -> __m512i {
    unsafe { _mm512_shuffle_i32x4::<HI_IMM8>(a, b) }
}

#[inline(always)]
unsafe fn transpose_vecs(vecs: &mut [__m512i; DEGREE]) {
    unsafe {
        // Interleave 32-bit lanes. The _0 unpack is lanes
        // 0/0/1/1/4/4/5/5/8/8/9/9/12/12/13/13, and the _2 unpack is lanes
        // 2/2/3/3/6/6/7/7/10/10/11/11/14/14/15/15.
        let ab_0 = _mm512_unpacklo_epi32(vecs[0], vecs[1]);
        let ab_2 = _mm512_unpackhi_epi32(vecs[0], vecs[1]);
        let cd_0 = _mm512_unpacklo_epi32(vecs[2], vecs[3]);
        let cd_2 = _mm512_unpackhi_epi32(vecs[2], vecs[3]);
        let ef_0 = _mm512_unpacklo_epi32(vecs[4], vecs[5]);
        let ef_2 = _mm512_unpackhi_epi32(vecs[4], vecs[5]);
        let gh_0 = _mm512_unpacklo_epi32(vecs[6], vecs[7]);
        let gh_2 = _mm512_unpackhi_epi32(vecs[6], vecs[7]);
        let ij_0 = _mm512_unpacklo_epi32(vecs[8], vecs[9]);
        let ij_2 = _mm512_unpackhi_epi32(vecs[8], vecs[9]);
        let kl_0 = _mm512_unpacklo_epi32(vecs[10], vecs[11]);
        let kl_2 = _mm512_unpackhi_epi32(vecs[10], vecs[11]);
        let mn_0 = _mm512_unpacklo_epi32(vecs[12], vecs[13]);
        let mn_2 = _mm512_unpackhi_epi32(vecs[12], vecs[13]);
        let op_0 = _mm512_unpacklo_epi32(vecs[14], vecs[15]);
        let op_2 = _mm512_unpackhi_epi32(vecs[14], vecs[15]);

        // Interleave 64-bit lanes. The _0 unpack is lanes
        // 0/0/0/0/4/4/4/4/8/8/8/8/12/12/12/12, the _1 unpack is lanes
        // 1/1/1/1/5/5/5/5/9/9/9/9/13/13/13/13, the _2 unpack is lanes
        // 2/2/2/2/6/6/6/6/10/10/10/10/14/14/14/14, and the _3 unpack is lanes
        // 3/3/3/3/7/7/7/7/11/11/11/11/15/15/15/15.
        let abcd_0 = _mm512_unpacklo_epi64(ab_0, cd_0);
        let abcd_1 = _mm512_unpackhi_epi64(ab_0, cd_0);
        let abcd_2 = _mm512_unpacklo_epi64(ab_2, cd_2);
        let abcd_3 = _mm512_unpackhi_epi64(ab_2, cd_2);
        let efgh_0 = _mm512_unpacklo_epi64(ef_0, gh_0);
        let efgh_1 = _mm512_unpackhi_epi64(ef_0, gh_0);
        let efgh_2 = _mm512_unpacklo_epi64(ef_2, gh_2);
        let efgh_3 = _mm512_unpackhi_epi64(ef_2, gh_2);
        let ijkl_0 = _mm512_unpacklo_epi64(ij_0, kl_0);
        let ijkl_1 = _mm512_unpackhi_epi64(ij_0, kl_0);
        let ijkl_2 = _mm512_unpacklo_epi64(ij_2, kl_2);
        let ijkl_3 = _mm512_unpackhi_epi64(ij_2, kl_2);
        let mnop_0 = _mm512_unpacklo_epi64(mn_0, op_0);
        let mnop_1 = _mm512_unpackhi_epi64(mn_0, op_0);
        let mnop_2 = _mm512_unpacklo_epi64(mn_2, op_2);
        let mnop_3 = _mm512_unpackhi_epi64(mn_2, op_2);

        // Interleave 128-bit lanes. The _0 unpack is
        // 0/0/0/0/8/8/8/8/0/0/0/0/8/8/8/8, the _1 unpack is
        // 1/1/1/1/9/9/9/9/1/1/1/1/9/9/9/9, and so on.
        let abcdefgh_0 = unpack_lo_128(abcd_0, efgh_0);
        let abcdefgh_1 = unpack_lo_128(abcd_1, efgh_1);
        let abcdefgh_2 = unpack_lo_128(abcd_2, efgh_2);
        let abcdefgh_3 = unpack_lo_128(abcd_3, efgh_3);
        let abcdefgh_4 = unpack_hi_128(abcd_0, efgh_0);
        let abcdefgh_5 = unpack_hi_128(abcd_1, efgh_1);
        let abcdefgh_6 = unpack_hi_128(abcd_2, efgh_2);
        let abcdefgh_7 = unpack_hi_128(abcd_3, efgh_3);
        let ijklmnop_0 = unpack_lo_128(ijkl_0, mnop_0);
        let ijklmnop_1 = unpack_lo_128(ijkl_1, mnop_1);
        let ijklmnop_2 = unpack_lo_128(ijkl_2, mnop_2);
        let ijklmnop_3 = unpack_lo_128(ijkl_3, mnop_3);
        let ijklmnop_4 = unpack_hi_128(ijkl_0, mnop_0);
        let ijklmnop_5 = unpack_hi_128(ijkl_1, mnop_1);
        let ijklmnop_6 = unpack_hi_128(ijkl_2, mnop_2);
        let ijklmnop_7 = unpack_hi_128(ijkl_3, mnop_3);

        // Interleave 128-bit lanes again for the final outputs.
        vecs[0] = unpack_lo_128(abcdefgh_0, ijklmnop_0);
        vecs[1] = unpack_lo_128(abcdefgh_1, ijklmnop_1);
        vecs[2] = unpack_lo_128(abcdefgh_2, ijklmnop_2);
        vecs[3] = unpack_lo_128(abcdefgh_3, ijklmnop_3);
        vecs[4] = unpack_lo_128(abcdefgh_4, ijklmnop_4);
        vecs[5] = unpack_lo_128(abcdefgh_5, ijklmnop_5);
        vecs[6] = unpack_lo_128(abcdefgh_6, ijklmnop_6);
        vecs[7] = unpack_lo_128(abcdefgh_7, ijklmnop_7);
        vecs[8] = unpack_hi_128(abcdefgh_0, ijklmnop_0);
        vecs[9] = unpack_hi_128(abcdefgh_1, ijklmnop_1);
        vecs[10] = unpack_hi_128(abcdefgh_2, ijklmnop_2);
        vecs[11] = unpack_hi_128(abcdefgh_3, ijklmnop_3);
        vecs[12] = unpack_hi_128(abcdefgh_4, ijklmnop_4);
        vecs[13] = unpack_hi_128(abcdefgh_5, ijklmnop_5);
        vecs[14] = unpack_hi_128(abcdefgh_6, ijklmnop_6);
        vecs[15] = unpack_hi_128(abcdefgh_7, ijklmnop_7);
    }
}

// Transpose 16 message blocks of 16 words into 16 vectors of 16 lanes.
//
// Rather than loading each 64-byte block whole and then running four butterfly
// stages over the result (64 shuffles, all of which compete for the single
// shuffle port), load each block as two 256-bit halves and pair input i with
// input i+8 in one register. That performs the 256-bit stage of the transpose
// as part of the load, leaving three stages, and the final lane gather then
// collapses into a single vpermt2d per output vector instead of a pair of
// vshufi64x2. The instruction count is about the same; what changes is that 16
// operations per block move off the shuffle port onto the load ports, which is
// where the win comes from. This is the shape blake3_avx512_x86-64_unix.S uses,
// down to the two permute index constants.
#[inline(always)]
unsafe fn transpose_msg_vecs(inputs: &[*const u8; DEGREE], block_offset: usize) -> [__m512i; 16] {
    unsafe {
        let gather0 = _mm512_setr_epi32(0, 1, 2, 3, 16, 17, 18, 19, 8, 9, 10, 11, 24, 25, 26, 27);
        let gather1 = _mm512_setr_epi32(4, 5, 6, 7, 20, 21, 22, 23, 12, 13, 14, 15, 28, 29, 30, 31);

        let mut vecs = [_mm512_setzero_si512(); 16];
        for half in 0..2 {
            let off = block_offset + half * 32;

            // Low 256 bits from input i, high 256 bits from input i+8.
            let mut t = [_mm512_setzero_si512(); 8];
            for i in 0..8 {
                let lo = _mm256_loadu_si256(inputs[i].add(off) as *const __m256i);
                let hi = _mm256_loadu_si256(inputs[i + 8].add(off) as *const __m256i);
                t[i] = _mm512_inserti64x4(_mm512_castsi256_si512(lo), hi, 1);
            }

            // Interleave qwords within each 128-bit lane.
            let mut u = [_mm512_setzero_si512(); 8];
            for i in 0..4 {
                u[2 * i] = _mm512_unpacklo_epi64(t[2 * i], t[2 * i + 1]);
                u[2 * i + 1] = _mm512_unpackhi_epi64(t[2 * i], t[2 * i + 1]);
            }

            // Pick one word per input out of each lane, then gather the four
            // 128-bit lanes of the two halves with one permute each.
            let lo_a = shuffle2_512!(u[0], u[2], LO_IMM8);
            let lo_b = shuffle2_512!(u[4], u[6], LO_IMM8);
            vecs[half * 8] = _mm512_permutex2var_epi32(lo_a, gather0, lo_b);
            vecs[half * 8 + 4] = _mm512_permutex2var_epi32(lo_a, gather1, lo_b);

            let hi_a = shuffle2_512!(u[0], u[2], HI_IMM8);
            let hi_b = shuffle2_512!(u[4], u[6], HI_IMM8);
            vecs[half * 8 + 1] = _mm512_permutex2var_epi32(hi_a, gather0, hi_b);
            vecs[half * 8 + 5] = _mm512_permutex2var_epi32(hi_a, gather1, hi_b);

            let lo_c = shuffle2_512!(u[1], u[3], LO_IMM8);
            let lo_d = shuffle2_512!(u[5], u[7], LO_IMM8);
            vecs[half * 8 + 2] = _mm512_permutex2var_epi32(lo_c, gather0, lo_d);
            vecs[half * 8 + 6] = _mm512_permutex2var_epi32(lo_c, gather1, lo_d);

            let hi_c = shuffle2_512!(u[1], u[3], HI_IMM8);
            let hi_d = shuffle2_512!(u[5], u[7], HI_IMM8);
            vecs[half * 8 + 3] = _mm512_permutex2var_epi32(hi_c, gather0, hi_d);
            vecs[half * 8 + 7] = _mm512_permutex2var_epi32(hi_c, gather1, hi_d);
        }

        for i in 0..DEGREE {
            _mm_prefetch(
                inputs[i].wrapping_add(block_offset + 128) as *const i8,
                _MM_HINT_T0,
            );
        }
        vecs
    }
}

#[inline(always)]
unsafe fn load_counters(counter: u64, increment_counter: IncrementCounter) -> (__m512i, __m512i) {
    let mask = if increment_counter.yes() { !0 } else { 0 };
    unsafe {
        (
            set16(
                counter_low(counter + (mask & 0)),
                counter_low(counter + (mask & 1)),
                counter_low(counter + (mask & 2)),
                counter_low(counter + (mask & 3)),
                counter_low(counter + (mask & 4)),
                counter_low(counter + (mask & 5)),
                counter_low(counter + (mask & 6)),
                counter_low(counter + (mask & 7)),
                counter_low(counter + (mask & 8)),
                counter_low(counter + (mask & 9)),
                counter_low(counter + (mask & 10)),
                counter_low(counter + (mask & 11)),
                counter_low(counter + (mask & 12)),
                counter_low(counter + (mask & 13)),
                counter_low(counter + (mask & 14)),
                counter_low(counter + (mask & 15)),
            ),
            set16(
                counter_high(counter + (mask & 0)),
                counter_high(counter + (mask & 1)),
                counter_high(counter + (mask & 2)),
                counter_high(counter + (mask & 3)),
                counter_high(counter + (mask & 4)),
                counter_high(counter + (mask & 5)),
                counter_high(counter + (mask & 6)),
                counter_high(counter + (mask & 7)),
                counter_high(counter + (mask & 8)),
                counter_high(counter + (mask & 9)),
                counter_high(counter + (mask & 10)),
                counter_high(counter + (mask & 11)),
                counter_high(counter + (mask & 12)),
                counter_high(counter + (mask & 13)),
                counter_high(counter + (mask & 14)),
                counter_high(counter + (mask & 15)),
            ),
        )
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn hash16(
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
            set1_512(key[0]),
            set1_512(key[1]),
            set1_512(key[2]),
            set1_512(key[3]),
            set1_512(key[4]),
            set1_512(key[5]),
            set1_512(key[6]),
            set1_512(key[7]),
        ];
        let (counter_low_vec, counter_high_vec) = load_counters(counter, increment_counter);
        let mut block_flags = flags | flags_start;

        for block in 0..blocks {
            if block + 1 == blocks {
                block_flags |= flags_end;
            }
            let block_len_vec = set1_512(BLOCK_LEN as u32); // full blocks only
            let block_flags_vec = set1_512(block_flags as u32);
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
                set1_512(IV[0]),
                set1_512(IV[1]),
                set1_512(IV[2]),
                set1_512(IV[3]),
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
            h_vecs[0] = xor_512(v[0], v[8]);
            h_vecs[1] = xor_512(v[1], v[9]);
            h_vecs[2] = xor_512(v[2], v[10]);
            h_vecs[3] = xor_512(v[3], v[11]);
            h_vecs[4] = xor_512(v[4], v[12]);
            h_vecs[5] = xor_512(v[5], v[13]);
            h_vecs[6] = xor_512(v[6], v[14]);
            h_vecs[7] = xor_512(v[7], v[15]);

            block_flags = flags;
        }

        // transpose_vecs operates on a 16x16 matrix of words, but we only have 8
        // state vectors. Pad the matrix with zeros. After transposition, store the
        // lower half of each vector.
        let mut padded = [
            h_vecs[0],
            h_vecs[1],
            h_vecs[2],
            h_vecs[3],
            h_vecs[4],
            h_vecs[5],
            h_vecs[6],
            h_vecs[7],
            set1_512(0),
            set1_512(0),
            set1_512(0),
            set1_512(0),
            set1_512(0),
            set1_512(0),
            set1_512(0),
            set1_512(0),
        ];
        transpose_vecs(&mut padded);
        for i in 0..DEGREE {
            let half = _mm512_castsi512_si256(padded[i]);
            _mm256_storeu_si256(out.as_mut_ptr().add(i * OUT_LEN) as *mut __m256i, half);
        }
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn xof16(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
    out: &mut [u8; DEGREE * BLOCK_LEN],
) {
    unsafe {
        let h_vecs = [
            set1_512(cv[0]),
            set1_512(cv[1]),
            set1_512(cv[2]),
            set1_512(cv[3]),
            set1_512(cv[4]),
            set1_512(cv[5]),
            set1_512(cv[6]),
            set1_512(cv[7]),
        ];
        let block_words = crate::platform::words_from_le_bytes_64(block);
        let msg_vecs = [
            set1_512(block_words[0]),
            set1_512(block_words[1]),
            set1_512(block_words[2]),
            set1_512(block_words[3]),
            set1_512(block_words[4]),
            set1_512(block_words[5]),
            set1_512(block_words[6]),
            set1_512(block_words[7]),
            set1_512(block_words[8]),
            set1_512(block_words[9]),
            set1_512(block_words[10]),
            set1_512(block_words[11]),
            set1_512(block_words[12]),
            set1_512(block_words[13]),
            set1_512(block_words[14]),
            set1_512(block_words[15]),
        ];
        let (counter_low_vec, counter_high_vec) = load_counters(counter, IncrementCounter::Yes);
        let block_len_vec = set1_512(block_len as u32);
        let block_flags_vec = set1_512(flags as u32);
        let mut v = [
            h_vecs[0],
            h_vecs[1],
            h_vecs[2],
            h_vecs[3],
            h_vecs[4],
            h_vecs[5],
            h_vecs[6],
            h_vecs[7],
            set1_512(IV[0]),
            set1_512(IV[1]),
            set1_512(IV[2]),
            set1_512(IV[3]),
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
        for i in 0..8 {
            let low = xor_512(v[i], v[i + 8]);
            let high = xor_512(v[i + 8], h_vecs[i]);
            v[i] = low;
            v[i + 8] = high;
        }
        transpose_vecs(&mut v);
        for i in 0..DEGREE {
            storeu_512(v[i], out.as_mut_ptr().add(i * BLOCK_LEN));
        }
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
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
            hash16(
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
        crate::avx2::hash_many(
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

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn xof_many(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    mut counter: u64,
    flags: u8,
    mut out: &mut [u8],
) {
    debug_assert_eq!(0, out.len() % BLOCK_LEN, "whole blocks only");
    while out.len() >= DEGREE * BLOCK_LEN {
        unsafe {
            xof16(
                cv,
                block,
                block_len,
                counter,
                flags,
                array_mut_ref!(out, 0, DEGREE * BLOCK_LEN),
            );
        }
        counter += DEGREE as u64;
        out = &mut out[DEGREE * BLOCK_LEN..];
    }
    // Unlike hash_many, xof_many doesn't have a lower-degree implementation to fall back on
    // below DEGREE. Fall back to the single-instance compression function instead.
    for out_block in out.chunks_exact_mut(BLOCK_LEN) {
        let out_array: &mut [u8; BLOCK_LEN] = out_block.try_into().unwrap();
        *out_array = unsafe { compress_xof(cv, block, block_len, counter, flags) };
        counter += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transpose() {
        if !crate::platform::avx512_detected() {
            return;
        }

        #[target_feature(enable = "avx512f,avx512vl")]
        unsafe fn transpose_wrapper(vecs: &mut [__m512i; DEGREE]) {
            unsafe { transpose_vecs(vecs) };
        }

        let mut matrix = [[0 as u32; DEGREE]; DEGREE];
        for i in 0..DEGREE {
            for j in 0..DEGREE {
                matrix[i][j] = (i * DEGREE + j) as u32;
            }
        }

        unsafe {
            let mut vecs: [__m512i; DEGREE] = core::mem::transmute(matrix);
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
    fn test_compress() {
        if !crate::platform::avx512_detected() {
            return;
        }
        crate::test::test_compress_fn(compress_in_place, compress_xof);
    }

    #[test]
    fn test_hash_many() {
        if !crate::platform::avx512_detected() {
            return;
        }
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }

    #[test]
    fn test_xof_many() {
        if !crate::platform::avx512_detected() {
            return;
        }
        crate::test::test_xof_many_fn(xof_many);
    }
}
