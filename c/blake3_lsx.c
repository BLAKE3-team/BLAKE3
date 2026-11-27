#include "blake3_impl.h"

#include <lsxintrin.h>
#define blake3_hash_many_fn     blake3_hash_many_lsx
#define blake3_hash_fn          blake3_hash_lsx
#define DEGREE                  4
#define vec                     __m128i
#define vld                     __lsx_vld
#define vst                     __lsx_vst
#define vaddw                   __lsx_vadd_w
#define vsubw                   __lsx_vsub_w
#define vand                    __lsx_vand_v
#define vxor                    __lsx_vxor_v
#define vsetw                   __lsx_vreplgr2vr_w
#define vrotriw                 __lsx_vrotri_w
#define vrotrw                  __lsx_vrotr_w
#define vreplgr2vr              __lsx_vreplgr2vr_w
#define vsltwu                  __lsx_vslt_wu
#define vilvqlw                 __lsx_vilvl_w
#define vilvqhw                 __lsx_vilvh_w
#define vilvqld                 __lsx_vilvl_d
#define vilvqhd                 __lsx_vilvh_d
#define counter_base            ((__m128i)((v4u32){0,1,2,3}))

INLINE vec loadu(const void *src) { return vld(src, 0); }
INLINE void storeu(vec src, void *dest) { vst(src, dest, 0); }
INLINE vec setu(uint32_t x) { return vsetw(x); }
INLINE vec addu(vec a, vec b) { return vaddw(a, b); }
INLINE vec subu(vec a, vec b) { return vsubw(a, b); }
INLINE vec and(vec a, vec b) { return vand(a, b); }
INLINE vec xor(vec a, vec b) { return vxor(a, b); }
INLINE vec sltu(vec a, vec b) { return vsltwu(a, b); }

INLINE void round_fn(vec v[16], vec m[16], size_t r) {
  v[0] = addu(addu(v[0], v[4]), m[(size_t)MSG_SCHEDULE[r][0]]);
  v[1] = addu(addu(v[1], v[5]), m[(size_t)MSG_SCHEDULE[r][2]]);
  v[2] = addu(addu(v[2], v[6]), m[(size_t)MSG_SCHEDULE[r][4]]);
  v[3] = addu(addu(v[3], v[7]), m[(size_t)MSG_SCHEDULE[r][6]]);
  v[12] = vrotriw(xor(v[12], v[0]), 16);
  v[13] = vrotriw(xor(v[13], v[1]), 16);
  v[14] = vrotriw(xor(v[14], v[2]), 16);
  v[15] = vrotriw(xor(v[15], v[3]), 16);
  v[8] = addu(v[8], v[12]);
  v[9] = addu(v[9], v[13]);
  v[10] = addu(v[10], v[14]);
  v[11] = addu(v[11], v[15]);
  v[4] = vrotriw(xor(v[4], v[8]), 12);
  v[5] = vrotriw(xor(v[5], v[9]), 12);
  v[6] = vrotriw(xor(v[6], v[10]), 12);
  v[7] = vrotriw(xor(v[7], v[11]), 12);
  v[0] = addu(addu(v[0], v[4]), m[(size_t)MSG_SCHEDULE[r][1]]);
  v[1] = addu(addu(v[1], v[5]), m[(size_t)MSG_SCHEDULE[r][3]]);
  v[2] = addu(addu(v[2], v[6]), m[(size_t)MSG_SCHEDULE[r][5]]);
  v[3] = addu(addu(v[3], v[7]), m[(size_t)MSG_SCHEDULE[r][7]]);
  v[12] = vrotriw(xor(v[12], v[0]), 8);
  v[13] = vrotriw(xor(v[13], v[1]), 8);
  v[14] = vrotriw(xor(v[14], v[2]), 8);
  v[15] = vrotriw(xor(v[15], v[3]), 8);
  v[8] = addu(v[8], v[12]);
  v[9] = addu(v[9], v[13]);
  v[10] = addu(v[10], v[14]);
  v[11] = addu(v[11], v[15]);
  v[4] = vrotriw(xor(v[4], v[8]), 7);
  v[5] = vrotriw(xor(v[5], v[9]), 7);
  v[6] = vrotriw(xor(v[6], v[10]), 7);
  v[7] = vrotriw(xor(v[7], v[11]), 7);
  v[0] = addu(addu(v[0], v[5]), m[(size_t)MSG_SCHEDULE[r][8]]);
  v[1] = addu(addu(v[1], v[6]), m[(size_t)MSG_SCHEDULE[r][10]]);
  v[2] = addu(addu(v[2], v[7]), m[(size_t)MSG_SCHEDULE[r][12]]);
  v[3] = addu(addu(v[3], v[4]), m[(size_t)MSG_SCHEDULE[r][14]]);
  v[15] = vrotriw(xor(v[15], v[0]), 16);
  v[12] = vrotriw(xor(v[12], v[1]), 16);
  v[13] = vrotriw(xor(v[13], v[2]), 16);
  v[14] = vrotriw(xor(v[14], v[3]), 16);
  v[10] = addu(v[10], v[15]);
  v[11] = addu(v[11], v[12]);
  v[8] = addu(v[8], v[13]);
  v[9] = addu(v[9], v[14]);
  v[5] = vrotriw(xor(v[5], v[10]), 12);
  v[6] = vrotriw(xor(v[6], v[11]), 12);
  v[7] = vrotriw(xor(v[7], v[8]), 12);
  v[4] = vrotriw(xor(v[4], v[9]), 12);
  v[0] = addu(addu(v[0], v[5]), m[(size_t)MSG_SCHEDULE[r][9]]);
  v[1] = addu(addu(v[1], v[6]), m[(size_t)MSG_SCHEDULE[r][11]]);
  v[2] = addu(addu(v[2], v[7]), m[(size_t)MSG_SCHEDULE[r][13]]);
  v[3] = addu(addu(v[3], v[4]), m[(size_t)MSG_SCHEDULE[r][15]]);
  v[15] = vrotriw(xor(v[15], v[0]), 8);
  v[12] = vrotriw(xor(v[12], v[1]), 8);
  v[13] = vrotriw(xor(v[13], v[2]), 8);
  v[14] = vrotriw(xor(v[14], v[3]), 8);
  v[10] = addu(v[10], v[15]);
  v[11] = addu(v[11], v[12]);
  v[8] = addu(v[8], v[13]);
  v[9] = addu(v[9], v[14]);
  v[5] = vrotriw(xor(v[5], v[10]), 7);
  v[6] = vrotriw(xor(v[6], v[11]), 7);
  v[7] = vrotriw(xor(v[7], v[8]), 7);
  v[4] = vrotriw(xor(v[4], v[9]), 7);
}

INLINE void load_counters(uint64_t counter, bool increment_counter,
                          vec *lo, vec *hi) {
  vec mask = setu(increment_counter ? -1 : 0);
  vec addval = and(counter_base, mask);
  *lo = addu(setu(counter_low(counter)), addval);
  *hi = subu(setu(counter_high(counter)), sltu(*lo, addval));
}

INLINE void transpose_vecs(vec vecs[DEGREE]) {
  vec t0[DEGREE];
  // Interleave 32-bits lanes.
  t0[0] = vilvqlw(vecs[1], vecs[0]);
  t0[1] = vilvqhw(vecs[1], vecs[0]);
  t0[2] = vilvqlw(vecs[3], vecs[2]);
  t0[3] = vilvqhw(vecs[3], vecs[2]);
  // Interleave 64-bits lanes.
  vecs[0] = vilvqld(t0[2], t0[0]);
  vecs[1] = vilvqhd(t0[2], t0[0]);
  vecs[2] = vilvqld(t0[3], t0[1]);
  vecs[3] = vilvqhd(t0[3], t0[1]);
}

INLINE void transpose_msg_vecs(const uint8_t *const *inputs,
                               size_t block_offset, vec out[16]) {
  out[0] = loadu(&inputs[0][block_offset + 0 * sizeof(vec)]);
  out[1] = loadu(&inputs[1][block_offset + 0 * sizeof(vec)]);
  out[2] = loadu(&inputs[2][block_offset + 0 * sizeof(vec)]);
  out[3] = loadu(&inputs[3][block_offset + 0 * sizeof(vec)]);
  out[4] = loadu(&inputs[0][block_offset + 1 * sizeof(vec)]);
  out[5] = loadu(&inputs[1][block_offset + 1 * sizeof(vec)]);
  out[6] = loadu(&inputs[2][block_offset + 1 * sizeof(vec)]);
  out[7] = loadu(&inputs[3][block_offset + 1 * sizeof(vec)]);
  out[8] = loadu(&inputs[0][block_offset + 2 * sizeof(vec)]);
  out[9] = loadu(&inputs[1][block_offset + 2 * sizeof(vec)]);
  out[10] = loadu(&inputs[2][block_offset + 2 * sizeof(vec)]);
  out[11] = loadu(&inputs[3][block_offset + 2 * sizeof(vec)]);
  out[12] = loadu(&inputs[0][block_offset + 3 * sizeof(vec)]);
  out[13] = loadu(&inputs[1][block_offset + 3 * sizeof(vec)]);
  out[14] = loadu(&inputs[2][block_offset + 3 * sizeof(vec)]);
  out[15] = loadu(&inputs[3][block_offset + 3 * sizeof(vec)]);

  transpose_vecs(&out[0]);
  transpose_vecs(&out[4]);
  transpose_vecs(&out[8]);
  transpose_vecs(&out[12]);
}

static
void blake3_hash_fn(const uint8_t *const *inputs, size_t blocks,
                    const uint32_t key[8], uint64_t counter,
                    bool increment_counter, uint8_t flags,
                    uint8_t flags_start, uint8_t flags_end, uint8_t *out) {
  vec h_vecs[8] = {
      setu(key[0]), setu(key[1]), setu(key[2]), setu(key[3]),
      setu(key[4]), setu(key[5]), setu(key[6]), setu(key[7]),
  };
  vec counter_low_vec, counter_high_vec;
  load_counters(counter, increment_counter, &counter_low_vec,
                &counter_high_vec);
  uint8_t block_flags = flags | flags_start;

  for (size_t block = 0; block < blocks; block++) {
    block_flags |= (block + 1 == blocks) ? flags_end : 0;
    vec block_len_vec = setu(BLAKE3_BLOCK_LEN);
    vec block_flags_vec = setu(block_flags);
    vec msg_vecs[16];
    transpose_msg_vecs(inputs, block * BLAKE3_BLOCK_LEN, msg_vecs);

    vec v[16] = {
        h_vecs[0],       h_vecs[1],        h_vecs[2],     h_vecs[3],
        h_vecs[4],       h_vecs[5],        h_vecs[6],     h_vecs[7],
        setu(IV[0]),     setu(IV[1]),      setu(IV[2]),   setu(IV[3]),
        counter_low_vec, counter_high_vec, block_len_vec, block_flags_vec,
    };
    round_fn(v, msg_vecs, 0);
    round_fn(v, msg_vecs, 1);
    round_fn(v, msg_vecs, 2);
    round_fn(v, msg_vecs, 3);
    round_fn(v, msg_vecs, 4);
    round_fn(v, msg_vecs, 5);
    round_fn(v, msg_vecs, 6);
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

  transpose_vecs(&h_vecs[0]);
  transpose_vecs(&h_vecs[DEGREE]);

  storeu(h_vecs[0], &out[0 * sizeof(vec)]);
  storeu(h_vecs[1], &out[2 * sizeof(vec)]);
  storeu(h_vecs[2], &out[4 * sizeof(vec)]);
  storeu(h_vecs[3], &out[6 * sizeof(vec)]);
  storeu(h_vecs[4], &out[1 * sizeof(vec)]);
  storeu(h_vecs[5], &out[3 * sizeof(vec)]);
  storeu(h_vecs[6], &out[5 * sizeof(vec)]);
  storeu(h_vecs[7], &out[7 * sizeof(vec)]);
}

void blake3_hash_many_fn(const uint8_t *const *inputs, size_t num_inputs,
                         size_t blocks, const uint32_t key[8],
                         uint64_t counter, bool increment_counter,
                         uint8_t flags, uint8_t flags_start,
                         uint8_t flags_end, uint8_t *out) {
  while (num_inputs >= DEGREE) {
    blake3_hash_fn(inputs, blocks, key, counter, increment_counter, flags,
                   flags_start, flags_end, out);
    counter += increment_counter ? DEGREE : 0;
    inputs += DEGREE;
    num_inputs -= DEGREE;
    out = &out[DEGREE * BLAKE3_OUT_LEN];
  }
  blake3_hash_many_portable(inputs, num_inputs, blocks, key, counter,
                            increment_counter, flags, flags_start, flags_end,
                            out);
}
