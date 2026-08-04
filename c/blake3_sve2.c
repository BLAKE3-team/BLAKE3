// SVE2 backend for BLAKE3, degree 4. This is a port of blake3_neon.c; the only
// difference is that every (xor, rotate-right) pair becomes a single SVE2 XAR:
//
//     v[12] = xor(v[12], v[0]); v[12] = rot16(v[12]);   // NEON
//     v[12] = svxar_n_u32(v[12], v[0], 16);             // SVE2
//
// Compiled with -msve-vector-bits=128, which is a promise that the runtime vector
// length is exactly 128 bits. blake3_dispatch.c checks that before dispatching
// here; see the comment there for why the check can't live in this file.

#include "blake3_impl.h"

// Checked before including arm_sve.h, because the pragma in that header
// re-establishes the target macros.
#ifdef __ARM_BIG_ENDIAN
#error "This implementation only supports little-endian ARM."
// Same situation as blake3_neon.c: the loads and stores would need fixing, and
// there is no way to test it in CI.
#endif

#include <arm_sve.h>

// Fixed-length SVE types, so that these can be used as array elements. Not named
// svuint32x4_t, because in ACLE that already means a tuple of four vectors (for
// LD4W) rather than four lanes; the underscore form follows Arm's own usage, as in
// svuint32_8_t in ARM-software/astc-encoder.
#if __ARM_FEATURE_SVE_BITS != 128
#error "blake3_sve2.c must be compiled with -msve-vector-bits=128"
#endif

typedef svuint32_t svuint32_4_t __attribute__((arm_sve_vector_bits(128)));
typedef svuint64_t svuint64_2_t __attribute__((arm_sve_vector_bits(128)));

#define PT svptrue_b32()
#define PT64 svptrue_b64()

INLINE svuint32_4_t loadu_128(const uint8_t src[16]) {
  // Load as bytes, like blake3_neon.c does, to avoid casting a possibly
  // unaligned uint8_t pointer to uint32_t.
  return svreinterpret_u32_u8(svld1_u8(svptrue_b8(), src));
}

INLINE void storeu_128(svuint32_4_t src, uint8_t dest[16]) {
  svst1_u8(svptrue_b8(), dest, svreinterpret_u8_u32(src));
}

INLINE svuint32_4_t add_128(svuint32_4_t a, svuint32_4_t b) {
  return svadd_u32_x(PT, a, b);
}

INLINE svuint32_4_t xor_128(svuint32_4_t a, svuint32_4_t b) {
  return sveor_u32_x(PT, a, b);
}

INLINE svuint32_4_t set1_128(uint32_t x) { return svdup_n_u32(x); }

INLINE svuint32_4_t set4(uint32_t a, uint32_t b, uint32_t c, uint32_t d) {
  uint32_t array[4] = {a, b, c, d};
  return svld1_u32(PT, array);
}

#define xar16_128(a, b) svxar_n_u32(a, b, 16)
#define xar12_128(a, b) svxar_n_u32(a, b, 12)
#define xar8_128(a, b) svxar_n_u32(a, b, 8)
#define xar7_128(a, b) svxar_n_u32(a, b, 7)

/*
 * ----------------------------------------------------------------------------
 * hash4_sve2
 * ----------------------------------------------------------------------------
 */

INLINE void round_fn4(svuint32_4_t v[16], svuint32_4_t m[16], size_t r) {
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][0]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][2]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][4]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][6]]);
  v[0] = add_128(v[0], v[4]);
  v[1] = add_128(v[1], v[5]);
  v[2] = add_128(v[2], v[6]);
  v[3] = add_128(v[3], v[7]);
  v[12] = xar16_128(v[12], v[0]);
  v[13] = xar16_128(v[13], v[1]);
  v[14] = xar16_128(v[14], v[2]);
  v[15] = xar16_128(v[15], v[3]);
  v[8] = add_128(v[8], v[12]);
  v[9] = add_128(v[9], v[13]);
  v[10] = add_128(v[10], v[14]);
  v[11] = add_128(v[11], v[15]);
  v[4] = xar12_128(v[4], v[8]);
  v[5] = xar12_128(v[5], v[9]);
  v[6] = xar12_128(v[6], v[10]);
  v[7] = xar12_128(v[7], v[11]);
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][1]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][3]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][5]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][7]]);
  v[0] = add_128(v[0], v[4]);
  v[1] = add_128(v[1], v[5]);
  v[2] = add_128(v[2], v[6]);
  v[3] = add_128(v[3], v[7]);
  v[12] = xar8_128(v[12], v[0]);
  v[13] = xar8_128(v[13], v[1]);
  v[14] = xar8_128(v[14], v[2]);
  v[15] = xar8_128(v[15], v[3]);
  v[8] = add_128(v[8], v[12]);
  v[9] = add_128(v[9], v[13]);
  v[10] = add_128(v[10], v[14]);
  v[11] = add_128(v[11], v[15]);
  v[4] = xar7_128(v[4], v[8]);
  v[5] = xar7_128(v[5], v[9]);
  v[6] = xar7_128(v[6], v[10]);
  v[7] = xar7_128(v[7], v[11]);

  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][8]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][10]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][12]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][14]]);
  v[0] = add_128(v[0], v[5]);
  v[1] = add_128(v[1], v[6]);
  v[2] = add_128(v[2], v[7]);
  v[3] = add_128(v[3], v[4]);
  v[15] = xar16_128(v[15], v[0]);
  v[12] = xar16_128(v[12], v[1]);
  v[13] = xar16_128(v[13], v[2]);
  v[14] = xar16_128(v[14], v[3]);
  v[10] = add_128(v[10], v[15]);
  v[11] = add_128(v[11], v[12]);
  v[8] = add_128(v[8], v[13]);
  v[9] = add_128(v[9], v[14]);
  v[5] = xar12_128(v[5], v[10]);
  v[6] = xar12_128(v[6], v[11]);
  v[7] = xar12_128(v[7], v[8]);
  v[4] = xar12_128(v[4], v[9]);
  v[0] = add_128(v[0], m[(size_t)MSG_SCHEDULE[r][9]]);
  v[1] = add_128(v[1], m[(size_t)MSG_SCHEDULE[r][11]]);
  v[2] = add_128(v[2], m[(size_t)MSG_SCHEDULE[r][13]]);
  v[3] = add_128(v[3], m[(size_t)MSG_SCHEDULE[r][15]]);
  v[0] = add_128(v[0], v[5]);
  v[1] = add_128(v[1], v[6]);
  v[2] = add_128(v[2], v[7]);
  v[3] = add_128(v[3], v[4]);
  v[15] = xar8_128(v[15], v[0]);
  v[12] = xar8_128(v[12], v[1]);
  v[13] = xar8_128(v[13], v[2]);
  v[14] = xar8_128(v[14], v[3]);
  v[10] = add_128(v[10], v[15]);
  v[11] = add_128(v[11], v[12]);
  v[8] = add_128(v[8], v[13]);
  v[9] = add_128(v[9], v[14]);
  v[5] = xar7_128(v[5], v[10]);
  v[6] = xar7_128(v[6], v[11]);
  v[7] = xar7_128(v[7], v[8]);
  v[4] = xar7_128(v[4], v[9]);
}

// 4x4 transpose of 32-bit lanes, matching transpose_vecs_128 in blake3_neon.c.
// TRN1/TRN2 do the 2x2 corner transposes; ZIP1/ZIP2 on 64-bit lanes then swap the
// top-right and bottom-left blocks, which is what vcombine does in the NEON version.
INLINE void transpose_vecs_128(svuint32_4_t vecs[4]) {
  svuint32_4_t t0 = svtrn1_u32(vecs[0], vecs[1]);
  svuint32_4_t t1 = svtrn2_u32(vecs[0], vecs[1]);
  svuint32_4_t t2 = svtrn1_u32(vecs[2], vecs[3]);
  svuint32_4_t t3 = svtrn2_u32(vecs[2], vecs[3]);

  svuint64_2_t a0 = svreinterpret_u64_u32(t0);
  svuint64_2_t a1 = svreinterpret_u64_u32(t1);
  svuint64_2_t a2 = svreinterpret_u64_u32(t2);
  svuint64_2_t a3 = svreinterpret_u64_u32(t3);

  vecs[0] = svreinterpret_u32_u64(svzip1_u64(a0, a2));
  vecs[1] = svreinterpret_u32_u64(svzip1_u64(a1, a3));
  vecs[2] = svreinterpret_u32_u64(svzip2_u64(a0, a2));
  vecs[3] = svreinterpret_u32_u64(svzip2_u64(a1, a3));
}

INLINE void transpose_msg_vecs4(const uint8_t *const *inputs,
                                size_t block_offset, svuint32_4_t out[16]) {
  out[0] = loadu_128(&inputs[0][block_offset + 0 * sizeof(svuint32_4_t)]);
  out[1] = loadu_128(&inputs[1][block_offset + 0 * sizeof(svuint32_4_t)]);
  out[2] = loadu_128(&inputs[2][block_offset + 0 * sizeof(svuint32_4_t)]);
  out[3] = loadu_128(&inputs[3][block_offset + 0 * sizeof(svuint32_4_t)]);
  out[4] = loadu_128(&inputs[0][block_offset + 1 * sizeof(svuint32_4_t)]);
  out[5] = loadu_128(&inputs[1][block_offset + 1 * sizeof(svuint32_4_t)]);
  out[6] = loadu_128(&inputs[2][block_offset + 1 * sizeof(svuint32_4_t)]);
  out[7] = loadu_128(&inputs[3][block_offset + 1 * sizeof(svuint32_4_t)]);
  out[8] = loadu_128(&inputs[0][block_offset + 2 * sizeof(svuint32_4_t)]);
  out[9] = loadu_128(&inputs[1][block_offset + 2 * sizeof(svuint32_4_t)]);
  out[10] = loadu_128(&inputs[2][block_offset + 2 * sizeof(svuint32_4_t)]);
  out[11] = loadu_128(&inputs[3][block_offset + 2 * sizeof(svuint32_4_t)]);
  out[12] = loadu_128(&inputs[0][block_offset + 3 * sizeof(svuint32_4_t)]);
  out[13] = loadu_128(&inputs[1][block_offset + 3 * sizeof(svuint32_4_t)]);
  out[14] = loadu_128(&inputs[2][block_offset + 3 * sizeof(svuint32_4_t)]);
  out[15] = loadu_128(&inputs[3][block_offset + 3 * sizeof(svuint32_4_t)]);
  transpose_vecs_128(&out[0]);
  transpose_vecs_128(&out[4]);
  transpose_vecs_128(&out[8]);
  transpose_vecs_128(&out[12]);
}

INLINE void load_counters4(uint64_t counter, bool increment_counter,
                           svuint32_4_t *out_low, svuint32_4_t *out_high) {
  uint64_t mask = (increment_counter ? ~0 : 0);
  *out_low = set4(
      counter_low(counter + (mask & 0)), counter_low(counter + (mask & 1)),
      counter_low(counter + (mask & 2)), counter_low(counter + (mask & 3)));
  *out_high = set4(
      counter_high(counter + (mask & 0)), counter_high(counter + (mask & 1)),
      counter_high(counter + (mask & 2)), counter_high(counter + (mask & 3)));
}

static void blake3_hash4_sve2(const uint8_t *const *inputs, size_t blocks,
                              const uint32_t key[8], uint64_t counter,
                              bool increment_counter, uint8_t flags,
                              uint8_t flags_start, uint8_t flags_end,
                              uint8_t *out) {
  svuint32_4_t h_vecs[8] = {
      set1_128(key[0]), set1_128(key[1]), set1_128(key[2]), set1_128(key[3]),
      set1_128(key[4]), set1_128(key[5]), set1_128(key[6]), set1_128(key[7]),
  };
  svuint32_4_t counter_low_vec, counter_high_vec;
  load_counters4(counter, increment_counter, &counter_low_vec,
                 &counter_high_vec);
  uint8_t block_flags = flags | flags_start;

  for (size_t block = 0; block < blocks; block++) {
    if (block + 1 == blocks) {
      block_flags |= flags_end;
    }
    svuint32_4_t block_len_vec = set1_128(BLAKE3_BLOCK_LEN);
    svuint32_4_t block_flags_vec = set1_128(block_flags);
    svuint32_4_t msg_vecs[16];
    transpose_msg_vecs4(inputs, block * BLAKE3_BLOCK_LEN, msg_vecs);

    svuint32_4_t v[16] = {
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
  storeu_128(h_vecs[0], &out[0 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[4], &out[1 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[1], &out[2 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[5], &out[3 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[2], &out[4 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[6], &out[5 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[3], &out[6 * sizeof(svuint32_4_t)]);
  storeu_128(h_vecs[7], &out[7 * sizeof(svuint32_4_t)]);
}

/*
 * ----------------------------------------------------------------------------
 * hash_many_sve2
 * ----------------------------------------------------------------------------
 */

void blake3_compress_in_place_portable(uint32_t cv[8],
                                       const uint8_t block[BLAKE3_BLOCK_LEN],
                                       uint8_t block_len, uint64_t counter,
                                       uint8_t flags);

// Same as hash_one_neon: there is no SIMD single-block compression on aarch64, so
// the tail falls back to portable.
INLINE void hash_one_sve2(const uint8_t *input, size_t blocks,
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
    blake3_compress_in_place_portable(cv, input, BLAKE3_BLOCK_LEN, counter,
                                      block_flags);
    input = &input[BLAKE3_BLOCK_LEN];
    blocks -= 1;
    block_flags = flags;
  }
  memcpy(out, cv, BLAKE3_OUT_LEN);
}

void blake3_hash_many_sve2(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out) {
  while (num_inputs >= 4) {
    blake3_hash4_sve2(inputs, blocks, key, counter, increment_counter, flags,
                      flags_start, flags_end, out);
    if (increment_counter) {
      counter += 4;
    }
    inputs += 4;
    num_inputs -= 4;
    out = &out[4 * BLAKE3_OUT_LEN];
  }
  while (num_inputs > 0) {
    hash_one_sve2(inputs[0], blocks, key, counter, flags, flags_start,
                  flags_end, out);
    if (increment_counter) {
      counter += 1;
    }
    inputs += 1;
    num_inputs -= 1;
    out = &out[BLAKE3_OUT_LEN];
  }
}
