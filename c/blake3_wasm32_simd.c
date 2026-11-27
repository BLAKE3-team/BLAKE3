#include "blake3_impl.h"

#include <wasm_simd128.h>

#define DEGREE 4

#define shuffle_epi32(__a, __c3, __c2, __c1, __c0)                             \
  (wasm_i32x4_shuffle(__a, __a, __c0, __c1, __c2, __c3))

#define shuffle_ps2(__a, __b, __z, __y, __x, __w)                              \
  (wasm_i32x4_shuffle(__a, __b, __w, __x, __y + 4, __z + 4))

#define unpacklo_epi64(__a, __b)                                               \
  (wasm_i64x2_shuffle(__a, __b, 0, 2))

#define unpackhi_epi64(__a, __b)                                               \
  (wasm_i64x2_shuffle(__a, __b, 1, 3))

#define unpacklo_epi32(__a, __b)                                               \
  (wasm_i32x4_shuffle(__a, __b, 0, 4, 1, 5))

#define unpackhi_epi32(__a, __b)                                               \
  (wasm_i32x4_shuffle(__a, __b, 2, 6, 3, 7))

INLINE v128_t loadu(const uint8_t src[16]) {
  return wasm_v128_load((const v128_t *)src);
}

INLINE void storeu(v128_t src, uint8_t dest[16]) {
  wasm_v128_store((v128_t *)dest, src);
}

INLINE v128_t addv(v128_t a, v128_t b) { return wasm_i32x4_add(a, b); }

// Note that clang-format doesn't like the name "xor" for some reason.
INLINE v128_t xorv(v128_t a, v128_t b) { return wasm_v128_xor(a, b); }

INLINE v128_t set1(uint32_t x) { return wasm_i32x4_splat((int32_t)x); }

INLINE v128_t set4(uint32_t a, uint32_t b, uint32_t c, uint32_t d) {
  return wasm_i32x4_make((int32_t)a, (int32_t)b, (int32_t)c, (int32_t)d);
}

INLINE v128_t rot16(v128_t x) {
  return xorv(wasm_u32x4_shr(x, 16), wasm_i32x4_shl(x, 32 - 16));
}

INLINE v128_t rot12(v128_t x) {
  return xorv(wasm_u32x4_shr(x, 12), wasm_i32x4_shl(x, 32 - 12));
}

INLINE v128_t rot8(v128_t x) {
  return xorv(wasm_u32x4_shr(x, 8), wasm_i32x4_shl(x, 32 - 8));
}

INLINE v128_t rot7(v128_t x) {
  return xorv(wasm_u32x4_shr(x, 7), wasm_i32x4_shl(x, 32 - 7));
}

INLINE void g1(v128_t *row0, v128_t *row1, v128_t *row2, v128_t *row3,
               v128_t m) {
  *row0 = addv(addv(*row0, m), *row1);
  *row3 = xorv(*row3, *row0);
  *row3 = rot16(*row3);
  *row2 = addv(*row2, *row3);
  *row1 = xorv(*row1, *row2);
  *row1 = rot12(*row1);
}

INLINE void g2(v128_t *row0, v128_t *row1, v128_t *row2, v128_t *row3,
               v128_t m) {
  *row0 = addv(addv(*row0, m), *row1);
  *row3 = xorv(*row3, *row0);
  *row3 = rot8(*row3);
  *row2 = addv(*row2, *row3);
  *row1 = xorv(*row1, *row2);
  *row1 = rot7(*row1);
}

// Note the optimization here of leaving row1 as the unrotated row, rather than
// row0. All the message loads below are adjusted to compensate for this. See
// discussion at https://github.com/sneves/blake2-avx2/pull/4
INLINE void diagonalize(v128_t *row0, v128_t *row2, v128_t *row3) {
  *row0 = shuffle_epi32(*row0, 2, 1, 0, 3);
  *row3 = shuffle_epi32(*row3, 1, 0, 3, 2);
  *row2 = shuffle_epi32(*row2, 0, 3, 2, 1);
}

INLINE void undiagonalize(v128_t *row0, v128_t *row2, v128_t *row3) {
  *row0 = shuffle_epi32(*row0, 0, 3, 2, 1);
  *row3 = shuffle_epi32(*row3, 1, 0, 3, 2);
  *row2 = shuffle_epi32(*row2, 2, 1, 0, 3);
}

INLINE v128_t blend_epi16(v128_t a, v128_t b, const int16_t imm8) {
  const v128_t bits = wasm_i16x8_make(0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80);
  v128_t mask = wasm_i16x8_splat(imm8);
  mask = wasm_v128_and(mask, bits);
  mask = wasm_i16x8_eq(mask, bits);
  return wasm_v128_bitselect(b, a, mask);
}

INLINE void compress_pre(v128_t rows[4], const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags) {
  rows[0] = loadu((uint8_t *)&cv[0]);
  rows[1] = loadu((uint8_t *)&cv[4]);
  rows[2] = set4(IV[0], IV[1], IV[2], IV[3]);
  rows[3] = set4(counter_low(counter), counter_high(counter),
                 (uint32_t)block_len, (uint32_t)flags);

  v128_t m0 = loadu(&block[sizeof(v128_t) * 0]);
  v128_t m1 = loadu(&block[sizeof(v128_t) * 1]);
  v128_t m2 = loadu(&block[sizeof(v128_t) * 2]);
  v128_t m3 = loadu(&block[sizeof(v128_t) * 3]);

  v128_t t0, t1, t2, t3, tt;

  // Round 1. The first round permutes the message words from the original
  // input order, into the groups that get mixed in parallel.
  t0 = shuffle_ps2(m0, m1, 2, 0, 2, 0); //  6  4  2  0
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m0, m1, 3, 1, 3, 1); //  7  5  3  1
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = shuffle_ps2(m2, m3, 2, 0, 2, 0); // 14 12 10  8
  t2 = shuffle_epi32(t2, 2, 1, 0, 3);   // 12 10  8 14
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = shuffle_ps2(m2, m3, 3, 1, 3, 1); // 15 13 11  9
  t3 = shuffle_epi32(t3, 2, 1, 0, 3);   // 13 11  9 15
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 2. This round and all following rounds apply a fixed permutation
  // to the message words from the round before.
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 3
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 4
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 5
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 6
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 7
  t0 = shuffle_ps2(m0, m1, 3, 1, 1, 2);
  t0 = shuffle_epi32(t0, 0, 3, 2, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_ps2(m2, m3, 3, 3, 2, 2);
  tt = shuffle_epi32(m0, 0, 0, 3, 3);
  t1 = blend_epi16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_epi64(m3, m1);
  tt = blend_epi16(t2, m2, 0xC0);
  t2 = shuffle_epi32(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_epi32(m1, m3);
  tt = unpacklo_epi32(m2, t3);
  t3 = shuffle_epi32(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
}

void blake3_compress_in_place_wasm32_simd(uint32_t cv[8],
                                          const uint8_t block[BLAKE3_BLOCK_LEN],
                                          uint8_t block_len, uint64_t counter,
                                          uint8_t flags) {
  v128_t rows[4];
  compress_pre(rows, cv, block, block_len, counter, flags);
  storeu(xorv(rows[0], rows[2]), (uint8_t *)&cv[0]);
  storeu(xorv(rows[1], rows[3]), (uint8_t *)&cv[4]);
}

void blake3_compress_xof_wasm32_simd(const uint32_t cv[8],
                                     const uint8_t block[BLAKE3_BLOCK_LEN],
                                     uint8_t block_len, uint64_t counter,
                                     uint8_t flags, uint8_t out[64]) {
  v128_t rows[4];
  compress_pre(rows, cv, block, block_len, counter, flags);
  storeu(xorv(rows[0], rows[2]), &out[0]);
  storeu(xorv(rows[1], rows[3]), &out[16]);
  storeu(xorv(rows[2], loadu((uint8_t *)&cv[0])), &out[32]);
  storeu(xorv(rows[3], loadu((uint8_t *)&cv[4])), &out[48]);
}

INLINE void round_fn(v128_t v[16], v128_t m[16], size_t r) {
  v[0] = addv(v[0], m[(size_t)MSG_SCHEDULE[r][0]]);
  v[1] = addv(v[1], m[(size_t)MSG_SCHEDULE[r][2]]);
  v[2] = addv(v[2], m[(size_t)MSG_SCHEDULE[r][4]]);
  v[3] = addv(v[3], m[(size_t)MSG_SCHEDULE[r][6]]);
  v[0] = addv(v[0], v[4]);
  v[1] = addv(v[1], v[5]);
  v[2] = addv(v[2], v[6]);
  v[3] = addv(v[3], v[7]);
  v[12] = xorv(v[12], v[0]);
  v[13] = xorv(v[13], v[1]);
  v[14] = xorv(v[14], v[2]);
  v[15] = xorv(v[15], v[3]);
  v[12] = rot16(v[12]);
  v[13] = rot16(v[13]);
  v[14] = rot16(v[14]);
  v[15] = rot16(v[15]);
  v[8] = addv(v[8], v[12]);
  v[9] = addv(v[9], v[13]);
  v[10] = addv(v[10], v[14]);
  v[11] = addv(v[11], v[15]);
  v[4] = xorv(v[4], v[8]);
  v[5] = xorv(v[5], v[9]);
  v[6] = xorv(v[6], v[10]);
  v[7] = xorv(v[7], v[11]);
  v[4] = rot12(v[4]);
  v[5] = rot12(v[5]);
  v[6] = rot12(v[6]);
  v[7] = rot12(v[7]);
  v[0] = addv(v[0], m[(size_t)MSG_SCHEDULE[r][1]]);
  v[1] = addv(v[1], m[(size_t)MSG_SCHEDULE[r][3]]);
  v[2] = addv(v[2], m[(size_t)MSG_SCHEDULE[r][5]]);
  v[3] = addv(v[3], m[(size_t)MSG_SCHEDULE[r][7]]);
  v[0] = addv(v[0], v[4]);
  v[1] = addv(v[1], v[5]);
  v[2] = addv(v[2], v[6]);
  v[3] = addv(v[3], v[7]);
  v[12] = xorv(v[12], v[0]);
  v[13] = xorv(v[13], v[1]);
  v[14] = xorv(v[14], v[2]);
  v[15] = xorv(v[15], v[3]);
  v[12] = rot8(v[12]);
  v[13] = rot8(v[13]);
  v[14] = rot8(v[14]);
  v[15] = rot8(v[15]);
  v[8] = addv(v[8], v[12]);
  v[9] = addv(v[9], v[13]);
  v[10] = addv(v[10], v[14]);
  v[11] = addv(v[11], v[15]);
  v[4] = xorv(v[4], v[8]);
  v[5] = xorv(v[5], v[9]);
  v[6] = xorv(v[6], v[10]);
  v[7] = xorv(v[7], v[11]);
  v[4] = rot7(v[4]);
  v[5] = rot7(v[5]);
  v[6] = rot7(v[6]);
  v[7] = rot7(v[7]);

  v[0] = addv(v[0], m[(size_t)MSG_SCHEDULE[r][8]]);
  v[1] = addv(v[1], m[(size_t)MSG_SCHEDULE[r][10]]);
  v[2] = addv(v[2], m[(size_t)MSG_SCHEDULE[r][12]]);
  v[3] = addv(v[3], m[(size_t)MSG_SCHEDULE[r][14]]);
  v[0] = addv(v[0], v[5]);
  v[1] = addv(v[1], v[6]);
  v[2] = addv(v[2], v[7]);
  v[3] = addv(v[3], v[4]);
  v[15] = xorv(v[15], v[0]);
  v[12] = xorv(v[12], v[1]);
  v[13] = xorv(v[13], v[2]);
  v[14] = xorv(v[14], v[3]);
  v[15] = rot16(v[15]);
  v[12] = rot16(v[12]);
  v[13] = rot16(v[13]);
  v[14] = rot16(v[14]);
  v[10] = addv(v[10], v[15]);
  v[11] = addv(v[11], v[12]);
  v[8] = addv(v[8], v[13]);
  v[9] = addv(v[9], v[14]);
  v[5] = xorv(v[5], v[10]);
  v[6] = xorv(v[6], v[11]);
  v[7] = xorv(v[7], v[8]);
  v[4] = xorv(v[4], v[9]);
  v[5] = rot12(v[5]);
  v[6] = rot12(v[6]);
  v[7] = rot12(v[7]);
  v[4] = rot12(v[4]);
  v[0] = addv(v[0], m[(size_t)MSG_SCHEDULE[r][9]]);
  v[1] = addv(v[1], m[(size_t)MSG_SCHEDULE[r][11]]);
  v[2] = addv(v[2], m[(size_t)MSG_SCHEDULE[r][13]]);
  v[3] = addv(v[3], m[(size_t)MSG_SCHEDULE[r][15]]);
  v[0] = addv(v[0], v[5]);
  v[1] = addv(v[1], v[6]);
  v[2] = addv(v[2], v[7]);
  v[3] = addv(v[3], v[4]);
  v[15] = xorv(v[15], v[0]);
  v[12] = xorv(v[12], v[1]);
  v[13] = xorv(v[13], v[2]);
  v[14] = xorv(v[14], v[3]);
  v[15] = rot8(v[15]);
  v[12] = rot8(v[12]);
  v[13] = rot8(v[13]);
  v[14] = rot8(v[14]);
  v[10] = addv(v[10], v[15]);
  v[11] = addv(v[11], v[12]);
  v[8] = addv(v[8], v[13]);
  v[9] = addv(v[9], v[14]);
  v[5] = xorv(v[5], v[10]);
  v[6] = xorv(v[6], v[11]);
  v[7] = xorv(v[7], v[8]);
  v[4] = xorv(v[4], v[9]);
  v[5] = rot7(v[5]);
  v[6] = rot7(v[6]);
  v[7] = rot7(v[7]);
  v[4] = rot7(v[4]);
}

INLINE void transpose_vecs(v128_t vecs[DEGREE]) {
  // Interleave 32-bit lanes. The low unpack is lanes 00/11 and the high is
  // 22/33. Note that this doesn't split the vector into two lanes, as the
  // AVX2 counterparts do.
  v128_t ab_01 = unpacklo_epi32(vecs[0], vecs[1]);
  v128_t ab_23 = unpackhi_epi32(vecs[0], vecs[1]);
  v128_t cd_01 = unpacklo_epi32(vecs[2], vecs[3]);
  v128_t cd_23 = unpackhi_epi32(vecs[2], vecs[3]);

  // Interleave 64-bit lanes.
  v128_t abcd_0 = unpacklo_epi64(ab_01, cd_01);
  v128_t abcd_1 = unpackhi_epi64(ab_01, cd_01);
  v128_t abcd_2 = unpacklo_epi64(ab_23, cd_23);
  v128_t abcd_3 = unpackhi_epi64(ab_23, cd_23);

  vecs[0] = abcd_0;
  vecs[1] = abcd_1;
  vecs[2] = abcd_2;
  vecs[3] = abcd_3;
}

INLINE void transpose_msg_vecs(const uint8_t *const *inputs,
                               size_t block_offset, v128_t out[16]) {
  out[0] = loadu(&inputs[0][block_offset + 0 * sizeof(v128_t)]);
  out[1] = loadu(&inputs[1][block_offset + 0 * sizeof(v128_t)]);
  out[2] = loadu(&inputs[2][block_offset + 0 * sizeof(v128_t)]);
  out[3] = loadu(&inputs[3][block_offset + 0 * sizeof(v128_t)]);
  out[4] = loadu(&inputs[0][block_offset + 1 * sizeof(v128_t)]);
  out[5] = loadu(&inputs[1][block_offset + 1 * sizeof(v128_t)]);
  out[6] = loadu(&inputs[2][block_offset + 1 * sizeof(v128_t)]);
  out[7] = loadu(&inputs[3][block_offset + 1 * sizeof(v128_t)]);
  out[8] = loadu(&inputs[0][block_offset + 2 * sizeof(v128_t)]);
  out[9] = loadu(&inputs[1][block_offset + 2 * sizeof(v128_t)]);
  out[10] = loadu(&inputs[2][block_offset + 2 * sizeof(v128_t)]);
  out[11] = loadu(&inputs[3][block_offset + 2 * sizeof(v128_t)]);
  out[12] = loadu(&inputs[0][block_offset + 3 * sizeof(v128_t)]);
  out[13] = loadu(&inputs[1][block_offset + 3 * sizeof(v128_t)]);
  out[14] = loadu(&inputs[2][block_offset + 3 * sizeof(v128_t)]);
  out[15] = loadu(&inputs[3][block_offset + 3 * sizeof(v128_t)]);

  transpose_vecs(&out[0]);
  transpose_vecs(&out[4]);
  transpose_vecs(&out[8]);
  transpose_vecs(&out[12]);
}

INLINE void load_counters(uint64_t counter, bool increment_counter,
                          v128_t *out_lo, v128_t *out_hi) {
  const v128_t mask = wasm_i32x4_splat(-(int32_t)increment_counter);
  const v128_t add0 = wasm_i32x4_make(0, 1, 2, 3);
  const v128_t add1 = wasm_v128_and(mask, add0);
  v128_t l = wasm_i32x4_add(wasm_i32x4_splat((int32_t)counter), add1);
  v128_t carry = wasm_i32x4_gt(
                            wasm_v128_xor(add1, wasm_i32x4_splat(0x80000000)),
                            wasm_v128_xor(l, wasm_i32x4_splat(0x80000000)));
  v128_t h = wasm_i32x4_sub(wasm_i32x4_splat((int32_t)(counter >> 32)), carry);
  *out_lo = l;
  *out_hi = h;
}

static
void blake3_hash4_wasm32_simd(const uint8_t *const *inputs, size_t blocks,
                              const uint32_t key[8], uint64_t counter,
                              bool increment_counter, uint8_t flags,
                              uint8_t flags_start, uint8_t flags_end,
                              uint8_t *out) {
  v128_t h_vecs[8] = {
      set1(key[0]), set1(key[1]), set1(key[2]), set1(key[3]),
      set1(key[4]), set1(key[5]), set1(key[6]), set1(key[7]),
  };
  v128_t counter_low_vec, counter_high_vec;
  load_counters(counter, increment_counter, &counter_low_vec,
                &counter_high_vec);
  uint8_t block_flags = flags | flags_start;

  for (size_t block = 0; block < blocks; block++) {
    if (block + 1 == blocks) {
      block_flags |= flags_end;
    }
    v128_t block_len_vec = set1(BLAKE3_BLOCK_LEN);
    v128_t block_flags_vec = set1(block_flags);
    v128_t msg_vecs[16];
    transpose_msg_vecs(inputs, block * BLAKE3_BLOCK_LEN, msg_vecs);

    v128_t v[16] = {
        h_vecs[0],       h_vecs[1],        h_vecs[2],     h_vecs[3],
        h_vecs[4],       h_vecs[5],        h_vecs[6],     h_vecs[7],
        set1(IV[0]),     set1(IV[1]),      set1(IV[2]),   set1(IV[3]),
        counter_low_vec, counter_high_vec, block_len_vec, block_flags_vec,
    };
    round_fn(v, msg_vecs, 0);
    round_fn(v, msg_vecs, 1);
    round_fn(v, msg_vecs, 2);
    round_fn(v, msg_vecs, 3);
    round_fn(v, msg_vecs, 4);
    round_fn(v, msg_vecs, 5);
    round_fn(v, msg_vecs, 6);
    h_vecs[0] = xorv(v[0], v[8]);
    h_vecs[1] = xorv(v[1], v[9]);
    h_vecs[2] = xorv(v[2], v[10]);
    h_vecs[3] = xorv(v[3], v[11]);
    h_vecs[4] = xorv(v[4], v[12]);
    h_vecs[5] = xorv(v[5], v[13]);
    h_vecs[6] = xorv(v[6], v[14]);
    h_vecs[7] = xorv(v[7], v[15]);

    block_flags = flags;
  }

  transpose_vecs(&h_vecs[0]);
  transpose_vecs(&h_vecs[4]);
  // The first four vecs now contain the first half of each output, and the
  // second four vecs contain the second half of each output.
  storeu(h_vecs[0], &out[0 * sizeof(v128_t)]);
  storeu(h_vecs[4], &out[1 * sizeof(v128_t)]);
  storeu(h_vecs[1], &out[2 * sizeof(v128_t)]);
  storeu(h_vecs[5], &out[3 * sizeof(v128_t)]);
  storeu(h_vecs[2], &out[4 * sizeof(v128_t)]);
  storeu(h_vecs[6], &out[5 * sizeof(v128_t)]);
  storeu(h_vecs[3], &out[6 * sizeof(v128_t)]);
  storeu(h_vecs[7], &out[7 * sizeof(v128_t)]);
}

INLINE void hash_one_wasm32_simd(const uint8_t *input, size_t blocks,
                                 const uint32_t key[8], uint64_t counter,
                                 uint8_t flags, uint8_t flags_start,
                                 uint8_t flags_end,
                                 uint8_t out[BLAKE3_OUT_LEN]) {
  uint32_t cv[8];
  memcpy(cv, key, BLAKE3_KEY_LEN);
  uint8_t block_flags = flags | flags_start;
  while (blocks > 0) {
    if (blocks == 1) {
      block_flags |= flags_end;
    }
    blake3_compress_in_place_wasm32_simd(cv, input, BLAKE3_BLOCK_LEN, counter,
                                         block_flags);
    input = &input[BLAKE3_BLOCK_LEN];
    blocks -= 1;
    block_flags = flags;
  }
  memcpy(out, cv, BLAKE3_OUT_LEN);
}

void blake3_hash_many_wasm32_simd(const uint8_t *const *inputs,
                                  size_t num_inputs, size_t blocks,
                                  const uint32_t key[8], uint64_t counter,
                                  bool increment_counter, uint8_t flags,
                                  uint8_t flags_start, uint8_t flags_end,
                                  uint8_t *out) {
  while (num_inputs >= DEGREE) {
    blake3_hash4_wasm32_simd(inputs, blocks, key, counter, increment_counter,
                             flags, flags_start, flags_end, out);
    if (increment_counter) {
      counter += DEGREE;
    }
    inputs += DEGREE;
    num_inputs -= DEGREE;
    out = &out[DEGREE * BLAKE3_OUT_LEN];
  }
  while (num_inputs > 0) {
    hash_one_wasm32_simd(inputs[0], blocks, key, counter, flags, flags_start,
                         flags_end, out);
    if (increment_counter) {
      counter += 1;
    }
    inputs += 1;
    num_inputs -= 1;
    out = &out[BLAKE3_OUT_LEN];
  }
}
