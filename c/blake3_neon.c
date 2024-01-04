#include "blake3_impl.h"

#include <arm_neon.h>

#ifdef __ARM_BIG_ENDIAN
#error "This implementation only supports little-endian ARM."
// It might be that all we need for big-endian support here is to get the loads
// and stores right, but step zero would be finding a way to test it in CI.
#endif

INLINE uint32x4_t loadu_128(const uint8_t src[16]) {
  // vld1q_u32 has alignment requirements. Don't use it.
  uint32x4_t x;
  memcpy(&x, src, 16);
  return x;
}

INLINE void storeu_128(uint32x4_t src, uint8_t dest[16]) {
  // vst1q_u32 has alignment requirements. Don't use it.
  memcpy(dest, &src, 16);
}

INLINE uint32x4_t add_128(uint32x4_t a, uint32x4_t b) {
  return vaddq_u32(a, b);
}

INLINE uint32x4_t xor_128(uint32x4_t a, uint32x4_t b) {
  return veorq_u32(a, b);
}

INLINE uint32x4_t set1_128(uint32_t x) { return vld1q_dup_u32(&x); }

INLINE uint32x4_t set4(uint32_t a, uint32_t b, uint32_t c, uint32_t d) {
  uint32_t array[4] = {a, b, c, d};
  return vld1q_u32(array);
}

INLINE uint32x4_t rot16_128(uint32x4_t x) {
  // The straightfoward implementation would be two shifts and an or, but that's
  // slower on microarchitectures we've tested. See
  // https://github.com/BLAKE3-team/BLAKE3/pull/319.
  // return vorrq_u32(vshrq_n_u32(x, 16), vshlq_n_u32(x, 32 - 16));
  return vreinterpretq_u32_u16(vrev32q_u16(vreinterpretq_u16_u32(x)));
}

INLINE uint32x4_t rot12_128(uint32x4_t x) {
  // See comment in rot16_128.
  // return vorrq_u32(vshrq_n_u32(x, 12), vshlq_n_u32(x, 32 - 12));
  return vsriq_n_u32(vshlq_n_u32(x, 32-12), x, 12);
}

INLINE uint32x4_t rot8_128(uint32x4_t x) {
  // See comment in rot16_128.
  // return vorrq_u32(vshrq_n_u32(x, 8), vshlq_n_u32(x, 32 - 8));
#if defined(__clang__)
  return vreinterpretq_u32_u8(__builtin_shufflevector(vreinterpretq_u8_u32(x), vreinterpretq_u8_u32(x), 1,2,3,0,5,6,7,4,9,10,11,8,13,14,15,12));
#elif __GNUC__ * 10000 + __GNUC_MINOR__ * 100 >=40700
  static const uint8x16_t r8 = {1,2,3,0,5,6,7,4,9,10,11,8,13,14,15,12};
  return vreinterpretq_u32_u8(__builtin_shuffle(vreinterpretq_u8_u32(x), vreinterpretq_u8_u32(x), r8));
#else 
  return vsriq_n_u32(vshlq_n_u32(x, 32-8), x, 8);
#endif
}

INLINE uint32x4_t rot7_128(uint32x4_t x) {
  // See comment in rot16_128.
  // return vorrq_u32(vshrq_n_u32(x, 7), vshlq_n_u32(x, 32 - 7));
  return vsriq_n_u32(vshlq_n_u32(x, 32-7), x, 7);
}

// TODO: hash2_neon

INLINE void g1(uint32x4_t *row0, uint32x4_t *row1, uint32x4_t *row2,
               uint32x4_t *row3, uint32x4_t m) {
  *row0 = vaddq_u32(vaddq_u32(*row0, m), *row1);
  *row3 = veorq_u32(*row3, *row0);
  *row3 = rot16_128(*row3);
  *row2 = vaddq_u32(*row2, *row3);
  *row1 = veorq_u32(*row1, *row2);
  *row1 = rot12_128(*row1);
}

INLINE void g2(uint32x4_t *row0, uint32x4_t *row1, uint32x4_t *row2,
               uint32x4_t *row3, uint32x4_t m) {
  *row0 = vaddq_u32(vaddq_u32(*row0, m), *row1);
  *row3 = veorq_u32(*row3, *row0);
  *row3 = rot8_128(*row3);
  *row2 = vaddq_u32(*row2, *row3);
  *row1 = veorq_u32(*row1, *row2);
  *row1 = rot7_128(*row1);
}

INLINE void diagonalize(uint32x4_t *row0, uint32x4_t *row2, uint32x4_t *row3) {
  *row0 = vextq_u32(*row0, *row0, 3);
  *row3 = vextq_u32(*row3, *row3, 2);
  *row2 = vextq_u32(*row2, *row2, 1);
}

INLINE void undiagonalize(uint32x4_t *row0, uint32x4_t *row2, uint32x4_t *row3) {
  *row0 = vextq_u32(*row0, *row0, 1);
  *row3 = vextq_u32(*row3, *row3, 2);
  *row2 = vextq_u32(*row2, *row2, 3);
}

#define unpacklo_32(a, b) \
  vzip1q_u32(a, b)

#define unpackhi_32(a, b) \
  vzip2q_u32(a, b)

#define unpacklo_64(a, b) \
  vreinterpretq_u64_u32(vzip1q_u64(vreinterpretq_u32_u64(a), vreinterpretq_u32_u64(b)))

#define shuffle_128(a, m3, m2, m1, m0) \
  (__builtin_shufflevector(a, a, m0, m1, m2, m3))

#define shuffle_256(a, b, m3, m2, m1, m0) \
  (__builtin_shufflevector(a, b, m0, m1, m2 + 4, m3 + 4))

#define blend_16(a, b, mask)      \
  (vreinterpretq_u32_u16(         \
    __builtin_shufflevector(      \
      vreinterpretq_u16_u32(a),   \
      vreinterpretq_u16_u32(b),   \
      0 + ((mask >> 0) & 1) * 8,  \
      1 + ((mask >> 1) & 1) * 8,  \
      2 + ((mask >> 2) & 1) * 8,  \
      3 + ((mask >> 3) & 1) * 8,  \
      4 + ((mask >> 4) & 1) * 8,  \
      5 + ((mask >> 5) & 1) * 8,  \
      6 + ((mask >> 6) & 1) * 8,  \
      7 + ((mask >> 7) & 1) * 8   \
      )))

INLINE void compress_pre(uint32x4_t rows[4], const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags) {
  rows[0] = loadu_128((uint8_t *)&cv[0]);
  rows[1] = loadu_128((uint8_t *)&cv[4]);
  rows[2] = set4(IV[0], IV[1], IV[2], IV[3]);
  rows[3] = set4(counter_low(counter), counter_high(counter),
                 (uint32_t)block_len, (uint32_t)flags);

  uint32x4_t m0 = loadu_128(&block[sizeof(uint32x4_t) * 0]);
  uint32x4_t m1 = loadu_128(&block[sizeof(uint32x4_t) * 1]);
  uint32x4_t m2 = loadu_128(&block[sizeof(uint32x4_t) * 2]);
  uint32x4_t m3 = loadu_128(&block[sizeof(uint32x4_t) * 3]);

  uint32x4_t t0, t1, t2, t3, tt;

  // Round 1. The first round permutes the message words from the original
  // input order, into the groups that get mixed in parallel.
  t0 = shuffle_256(m0, m1, 2, 0, 2, 0); //  6  4  2  0
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m0, m1, 3, 1, 3, 1); //  7  5  3  1
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = shuffle_256(m2, m3, 2, 0, 2, 0); // 14 12 10  8
  t2 = shuffle_128(t2, 2, 1, 0, 3);     // 12 10  8 14
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = shuffle_256(m2, m3, 3, 1, 3, 1); // 15 13 11  9
  t3 = vextq_u32(t3, t3, 3);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 2. This round and all following rounds apply a fixed permutation
  // to the message words from the round before.
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 3
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 4
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 5
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 6
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
  m0 = t0;
  m1 = t1;
  m2 = t2;
  m3 = t3;

  // Round 7
  t0 = shuffle_256(m0, m1, 3, 1, 1, 2);
  t0 = vextq_u32(t0, t0, 1);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t0);
  t1 = shuffle_256(m2, m3, 3, 3, 2, 2);
  tt = shuffle_128(m0, 0, 0, 3, 3);
  t1 = blend_16(tt, t1, 0xCC);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t1);
  diagonalize(&rows[0], &rows[2], &rows[3]);
  t2 = unpacklo_64(m3, m1);
  tt = blend_16(t2, m2, 0xC0);
  t2 = shuffle_128(tt, 1, 3, 2, 0);
  g1(&rows[0], &rows[1], &rows[2], &rows[3], t2);
  t3 = unpackhi_32(m1, m3);
  tt = unpacklo_32(m2, t3);
  t3 = shuffle_128(tt, 0, 1, 3, 2);
  g2(&rows[0], &rows[1], &rows[2], &rows[3], t3);
  undiagonalize(&rows[0], &rows[2], &rows[3]);
}

void blake3_compress_in_place_neon(uint32_t cv[8],
                                   const uint8_t block[BLAKE3_BLOCK_LEN],
                                   uint8_t block_len, uint64_t counter,
                                   uint8_t flags) {
  uint32x4_t rows[4];
  compress_pre(rows, cv, block, block_len, counter, flags);
  storeu_128(veorq_u32(rows[0], rows[2]), (uint8_t *)&cv[0]);
  storeu_128(veorq_u32(rows[1], rows[3]), (uint8_t *)&cv[4]);
}

void blake3_compress_xof_neon(const uint32_t cv[8],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t counter,
                              uint8_t flags, uint8_t out[64]) {
  uint32x4_t rows[4];
  compress_pre(rows, cv, block, block_len, counter, flags);
  storeu_128(veorq_u32(rows[0], rows[2]), &out[0]);
  storeu_128(veorq_u32(rows[1], rows[3]), &out[16]);
  storeu_128(veorq_u32(rows[2], loadu_128((uint8_t *)&cv[0])), &out[32]);
  storeu_128(veorq_u32(rows[3], loadu_128((uint8_t *)&cv[4])), &out[48]);
}

/*
 * ----------------------------------------------------------------------------
 * hash4_neon
 * ----------------------------------------------------------------------------
 */

INLINE void round_fn4(uint32x4_t v[16], uint32x4_t m[16], size_t r) {
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][0]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][2]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][4]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][6]]);
  v[0] = add_128(v[0], v[4]);
  v[1] = add_128(v[1], v[5]);
  v[2] = add_128(v[2], v[6]);
  v[3] = add_128(v[3], v[7]);
  v[12] = xor_128(v[12], v[0]);
  v[13] = xor_128(v[13], v[1]);
  v[14] = xor_128(v[14], v[2]);
  v[15] = xor_128(v[15], v[3]);
  v[12] = rot16_128(v[12]);
  v[13] = rot16_128(v[13]);
  v[14] = rot16_128(v[14]);
  v[15] = rot16_128(v[15]);
  v[8] = add_128(v[8], v[12]);
  v[9] = add_128(v[9], v[13]);
  v[10] = add_128(v[10], v[14]);
  v[11] = add_128(v[11], v[15]);
  v[4] = xor_128(v[4], v[8]);
  v[5] = xor_128(v[5], v[9]);
  v[6] = xor_128(v[6], v[10]);
  v[7] = xor_128(v[7], v[11]);
  v[4] = rot12_128(v[4]);
  v[5] = rot12_128(v[5]);
  v[6] = rot12_128(v[6]);
  v[7] = rot12_128(v[7]);
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][1]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][3]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][5]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][7]]);
  v[0] = add_128(v[0], v[4]);
  v[1] = add_128(v[1], v[5]);
  v[2] = add_128(v[2], v[6]);
  v[3] = add_128(v[3], v[7]);
  v[12] = xor_128(v[12], v[0]);
  v[13] = xor_128(v[13], v[1]);
  v[14] = xor_128(v[14], v[2]);
  v[15] = xor_128(v[15], v[3]);
  v[12] = rot8_128(v[12]);
  v[13] = rot8_128(v[13]);
  v[14] = rot8_128(v[14]);
  v[15] = rot8_128(v[15]);
  v[8] = add_128(v[8], v[12]);
  v[9] = add_128(v[9], v[13]);
  v[10] = add_128(v[10], v[14]);
  v[11] = add_128(v[11], v[15]);
  v[4] = xor_128(v[4], v[8]);
  v[5] = xor_128(v[5], v[9]);
  v[6] = xor_128(v[6], v[10]);
  v[7] = xor_128(v[7], v[11]);
  v[4] = rot7_128(v[4]);
  v[5] = rot7_128(v[5]);
  v[6] = rot7_128(v[6]);
  v[7] = rot7_128(v[7]);

  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][8]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][10]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][12]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][14]]);
  v[0] = add_128(v[0], v[5]);
  v[1] = add_128(v[1], v[6]);
  v[2] = add_128(v[2], v[7]);
  v[3] = add_128(v[3], v[4]);
  v[15] = xor_128(v[15], v[0]);
  v[12] = xor_128(v[12], v[1]);
  v[13] = xor_128(v[13], v[2]);
  v[14] = xor_128(v[14], v[3]);
  v[15] = rot16_128(v[15]);
  v[12] = rot16_128(v[12]);
  v[13] = rot16_128(v[13]);
  v[14] = rot16_128(v[14]);
  v[10] = add_128(v[10], v[15]);
  v[11] = add_128(v[11], v[12]);
  v[8] = add_128(v[8], v[13]);
  v[9] = add_128(v[9], v[14]);
  v[5] = xor_128(v[5], v[10]);
  v[6] = xor_128(v[6], v[11]);
  v[7] = xor_128(v[7], v[8]);
  v[4] = xor_128(v[4], v[9]);
  v[5] = rot12_128(v[5]);
  v[6] = rot12_128(v[6]);
  v[7] = rot12_128(v[7]);
  v[4] = rot12_128(v[4]);
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][9]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][11]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][13]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][15]]);
  v[0] = add_128(v[0], v[5]);
  v[1] = add_128(v[1], v[6]);
  v[2] = add_128(v[2], v[7]);
  v[3] = add_128(v[3], v[4]);
  v[15] = xor_128(v[15], v[0]);
  v[12] = xor_128(v[12], v[1]);
  v[13] = xor_128(v[13], v[2]);
  v[14] = xor_128(v[14], v[3]);
  v[15] = rot8_128(v[15]);
  v[12] = rot8_128(v[12]);
  v[13] = rot8_128(v[13]);
  v[14] = rot8_128(v[14]);
  v[10] = add_128(v[10], v[15]);
  v[11] = add_128(v[11], v[12]);
  v[8] = add_128(v[8], v[13]);
  v[9] = add_128(v[9], v[14]);
  v[5] = xor_128(v[5], v[10]);
  v[6] = xor_128(v[6], v[11]);
  v[7] = xor_128(v[7], v[8]);
  v[4] = xor_128(v[4], v[9]);
  v[5] = rot7_128(v[5]);
  v[6] = rot7_128(v[6]);
  v[7] = rot7_128(v[7]);
  v[4] = rot7_128(v[4]);
}

INLINE void transpose_vecs_128(uint32x4_t vecs[4]) {
  // Individually transpose the four 2x2 sub-matrices in each corner.
  uint32x4x2_t rows01 = vtrnq_u32(vecs[0], vecs[1]);
  uint32x4x2_t rows23 = vtrnq_u32(vecs[2], vecs[3]);

  // Swap the top-right and bottom-left 2x2s (which just got transposed).
  vecs[0] =
      vcombine_u32(vget_low_u32(rows01.val[0]), vget_low_u32(rows23.val[0]));
  vecs[1] =
      vcombine_u32(vget_low_u32(rows01.val[1]), vget_low_u32(rows23.val[1]));
  vecs[2] =
      vcombine_u32(vget_high_u32(rows01.val[0]), vget_high_u32(rows23.val[0]));
  vecs[3] =
      vcombine_u32(vget_high_u32(rows01.val[1]), vget_high_u32(rows23.val[1]));
}

INLINE void transpose_msg_vecs4(const uint8_t *const *inputs,
                                size_t block_offset, uint32x4_t out[16]) {
  out[0] = loadu_128(&inputs[0][block_offset + 0 * sizeof(uint32x4_t)]);
  out[1] = loadu_128(&inputs[1][block_offset + 0 * sizeof(uint32x4_t)]);
  out[2] = loadu_128(&inputs[2][block_offset + 0 * sizeof(uint32x4_t)]);
  out[3] = loadu_128(&inputs[3][block_offset + 0 * sizeof(uint32x4_t)]);
  out[4] = loadu_128(&inputs[0][block_offset + 1 * sizeof(uint32x4_t)]);
  out[5] = loadu_128(&inputs[1][block_offset + 1 * sizeof(uint32x4_t)]);
  out[6] = loadu_128(&inputs[2][block_offset + 1 * sizeof(uint32x4_t)]);
  out[7] = loadu_128(&inputs[3][block_offset + 1 * sizeof(uint32x4_t)]);
  out[8] = loadu_128(&inputs[0][block_offset + 2 * sizeof(uint32x4_t)]);
  out[9] = loadu_128(&inputs[1][block_offset + 2 * sizeof(uint32x4_t)]);
  out[10] = loadu_128(&inputs[2][block_offset + 2 * sizeof(uint32x4_t)]);
  out[11] = loadu_128(&inputs[3][block_offset + 2 * sizeof(uint32x4_t)]);
  out[12] = loadu_128(&inputs[0][block_offset + 3 * sizeof(uint32x4_t)]);
  out[13] = loadu_128(&inputs[1][block_offset + 3 * sizeof(uint32x4_t)]);
  out[14] = loadu_128(&inputs[2][block_offset + 3 * sizeof(uint32x4_t)]);
  out[15] = loadu_128(&inputs[3][block_offset + 3 * sizeof(uint32x4_t)]);
  transpose_vecs_128(&out[0]);
  transpose_vecs_128(&out[4]);
  transpose_vecs_128(&out[8]);
  transpose_vecs_128(&out[12]);
}

// NOTE: The version below avoids the explicit transposes by relying on the interleaving from
// `vst4q_u32` but it seems to make no difference, or perhaps might be even a little slower.

// INLINE void transpose_msg_vecs4(const uint8_t *const *inputs,
//                                 size_t block_offset, uint32x4_t out[4]) {
//   uint8x16x4_t l0 = vld1q_u8_x4(&inputs[0][block_offset]);
//   uint8x16x4_t l1 = vld1q_u8_x4(&inputs[1][block_offset]);
//   uint8x16x4_t l2 = vld1q_u8_x4(&inputs[2][block_offset]);
//   uint8x16x4_t l3 = vld1q_u8_x4(&inputs[3][block_offset]);

//   uint32x4x4_t s0 = {
//     vreinterpretq_u32_u8(l0.val[0]),
//     vreinterpretq_u32_u8(l1.val[0]),
//     vreinterpretq_u32_u8(l2.val[0]),
//     vreinterpretq_u32_u8(l3.val[0]),
//   };
//   uint32x4x4_t s1 = {
//     vreinterpretq_u32_u8(l0.val[1]),
//     vreinterpretq_u32_u8(l1.val[1]),
//     vreinterpretq_u32_u8(l2.val[1]),
//     vreinterpretq_u32_u8(l3.val[1]),
//   };
//   uint32x4x4_t s2 = {
//     vreinterpretq_u32_u8(l0.val[2]),
//     vreinterpretq_u32_u8(l1.val[2]),
//     vreinterpretq_u32_u8(l2.val[2]),
//     vreinterpretq_u32_u8(l3.val[2]),
//   };
//   uint32x4x4_t s3 = {
//     vreinterpretq_u32_u8(l0.val[3]),
//     vreinterpretq_u32_u8(l1.val[3]),
//     vreinterpretq_u32_u8(l2.val[3]),
//     vreinterpretq_u32_u8(l3.val[3]),
//   };

//   vst4q_u32((uint32_t *)&out[0], s0);
//   vst4q_u32((uint32_t *)&out[4], s1);
//   vst4q_u32((uint32_t *)&out[8], s2);
//   vst4q_u32((uint32_t *)&out[12], s3);
// }

INLINE void load_counters4(uint64_t counter, bool increment_counter,
                           uint32x4_t *out_low, uint32x4_t *out_high) {
  uint64_t mask = (increment_counter ? ~0 : 0);
  *out_low = set4(
      counter_low(counter + (mask & 0)), counter_low(counter + (mask & 1)),
      counter_low(counter + (mask & 2)), counter_low(counter + (mask & 3)));
  *out_high = set4(
      counter_high(counter + (mask & 0)), counter_high(counter + (mask & 1)),
      counter_high(counter + (mask & 2)), counter_high(counter + (mask & 3)));
}

void blake3_hash4_neon(const uint8_t *const *inputs, size_t blocks,
                       const uint32_t key[8], uint64_t counter,
                       bool increment_counter, uint8_t flags,
                       uint8_t flags_start, uint8_t flags_end, uint8_t *out) {
  uint32x4_t h_vecs[8] = {
      set1_128(key[0]), set1_128(key[1]), set1_128(key[2]), set1_128(key[3]),
      set1_128(key[4]), set1_128(key[5]), set1_128(key[6]), set1_128(key[7]),
  };
  uint32x4_t counter_low_vec, counter_high_vec;
  load_counters4(counter, increment_counter, &counter_low_vec,
                 &counter_high_vec);
  uint8_t block_flags = flags | flags_start;

  for (size_t block = 0; block < blocks; block++) {
    if (block + 1 == blocks) {
      block_flags |= flags_end;
    }
    uint32x4_t block_len_vec = set1_128(BLAKE3_BLOCK_LEN);
    uint32x4_t block_flags_vec = set1_128(block_flags);
    uint32x4_t msg_vecs[16];
    transpose_msg_vecs4(inputs, block * BLAKE3_BLOCK_LEN, msg_vecs);

    uint32x4_t v[16] = {
        h_vecs[0],       h_vecs[1],        h_vecs[2],       h_vecs[3],
        h_vecs[4],       h_vecs[5],        h_vecs[6],       h_vecs[7],
        set1_128(IV[0]), set1_128(IV[1]),  set1_128(IV[2]), set1_128(IV[3]),
        counter_low_vec, counter_high_vec, block_len_vec,   block_flags_vec,
    };
    round_fn4(v, msg_vecs, 0);
    round_fn4(v, msg_vecs, 1);
    round_fn4(v, msg_vecs, 2);
    round_fn4(v, msg_vecs, 3);
    round_fn4(v, msg_vecs, 4);
    round_fn4(v, msg_vecs, 5);
    round_fn4(v, msg_vecs, 6);
    h_vecs[0] = xor_128(v[0], v[8]);
    h_vecs[1] = xor_128(v[1], v[9]);
    h_vecs[2] = xor_128(v[2], v[10]);
    h_vecs[3] = xor_128(v[3], v[11]);
    h_vecs[4] = xor_128(v[4], v[12]);
    h_vecs[5] = xor_128(v[5], v[13]);
    h_vecs[6] = xor_128(v[6], v[14]);
    h_vecs[7] = xor_128(v[7], v[15]);

    block_flags = flags;
  }

  transpose_vecs_128(&h_vecs[0]);
  transpose_vecs_128(&h_vecs[4]);
  // The first four vecs now contain the first half of each output, and the
  // second four vecs contain the second half of each output.
  storeu_128(h_vecs[0], &out[0 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[4], &out[1 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[1], &out[2 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[5], &out[3 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[2], &out[4 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[6], &out[5 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[3], &out[6 * sizeof(uint32x4_t)]);
  storeu_128(h_vecs[7], &out[7 * sizeof(uint32x4_t)]);
}

/*
 * ----------------------------------------------------------------------------
 * hash_many_neon
 * ----------------------------------------------------------------------------
 */

INLINE void hash_one_neon(const uint8_t *input, size_t blocks,
                          const uint32_t key[8], uint64_t counter,
                          uint8_t flags, uint8_t flags_start, uint8_t flags_end,
                          uint8_t out[BLAKE3_OUT_LEN]) {
  uint32_t cv[8];
  memcpy(cv, key, BLAKE3_KEY_LEN);
  uint8_t block_flags = flags | flags_start;
  while (blocks > 0) {
    if (blocks == 1) {
      block_flags |= flags_end;
    }
    blake3_compress_in_place_neon(cv, input, BLAKE3_BLOCK_LEN, counter,
                                  block_flags);
    input = &input[BLAKE3_BLOCK_LEN];
    blocks -= 1;
    block_flags = flags;
  }
  memcpy(out, cv, BLAKE3_OUT_LEN);
}

void blake3_hash_many_neon(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out) {
  while (num_inputs >= 4) {
    blake3_hash4_neon(inputs, blocks, key, counter, increment_counter, flags,
                      flags_start, flags_end, out);
    if (increment_counter) {
      counter += 4;
    }
    inputs += 4;
    num_inputs -= 4;
    out = &out[4 * BLAKE3_OUT_LEN];
  }
  while (num_inputs > 0) {
    hash_one_neon(inputs[0], blocks, key, counter, flags, flags_start,
                  flags_end, out);
    if (increment_counter) {
      counter += 1;
    }
    inputs += 1;
    num_inputs -= 1;
    out = &out[BLAKE3_OUT_LEN];
  }
}
