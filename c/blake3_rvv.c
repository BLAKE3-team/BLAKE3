#include "blake3_impl.h"
#include <riscv_vector.h>

INLINE vuint32m1_t rvv_vror_v(vuint32m1_t src, size_t shr, size_t vl) {
  vuint32m1_t dst = __riscv_vundefined_u32m1();
  size_t delta = vl - shr;
  dst = __riscv_vslideup(dst, src, delta, vl);       // VL = vl
  dst = __riscv_vslidedown_tu(dst, src, shr, delta); // VL = dela
  return dst;
}

INLINE vuint32m1_t rvv_vror_s(vuint32m1_t src, size_t shr, size_t vl) {
  size_t shl = sizeof(uint32_t) * 8 - shr;
  vuint32m1_t op0 = __riscv_vsrl(src, shr, vl);
  vuint32m1_t op1 = __riscv_vsll(src, shl, vl);
  return __riscv_vor(op0, op1, vl);
}

// NOTE: See the following for several approaches to transposing matrices with RVV:
// https://fprox.substack.com/p/transposing-a-matrix-using-risc-v
//
// This version of transpose_nxn uses the strided store approach, which easily scales to NxN
// matrices. For LMUL=1, the data at the above link suggests that this approach may be less
// efficient than a scalar implementation for large N > 16. However, for LMUL=8, the data suggests
// this vectorized approach may be more efficient than a scalar implementation at least up to N=512.
//
// If we assume a typical vector size of 128b (the minimum; no idea how representative this will
// be), then LMUL=8 gives us vector groups of 1024b (x 4, since RVV has 32 registers). This should
// let us process 32 state vectors at a time (c.f., 8 for AVX2, 16 for AVX512). Wider base vector
// registers would increase this further, of course.
//
// One of the more efficient alternative approaches would be to use segmented loads/stores. However,
// this would limit us to 8-row matrices, based on the currently supported vector tuple-sizes.
//
// With larger vector registers (or larger LMUL), the in-register masked slide approach may also be
// worth exploring.

INLINE
void rvv_transpose_nxn(uint32_t *dst, uint32_t *src, size_t n) {
  for (size_t row_idx = 0; row_idx < n; row_idx += 1) {
    size_t avl = n;
    uint32_t *row_src = src + row_idx * n;
    uint32_t *row_dst = dst + row_idx;
    for (
        /* clang-format off */
      size_t vl  = __riscv_vsetvl_e32m8(avl);
              0  < avl;
            avl -=  vl,
        row_src +=  vl,
        row_dst +=  vl * n
        /* clang-format on */
    ) {
      vuint32m8_t row = __riscv_vle32_v_u32m8(row_src, vl);
      __riscv_vsse32(row_dst, sizeof(uint32_t) * n, row, vl);
    }
  }
}

INLINE
vuint32m1_t rvv_zip_lo_u32(vuint32m1_t op0, vuint32m1_t op1) {
  vuint32m1_t odd_mask_u32 = __riscv_vmv_v_x_u32m1(0xAAAA, 1);
  vbool32_t odd_mask = __riscv_vreinterpret_v_u32m1_b32(odd_mask_u32);
  op0 = __riscv_vslideup_tu(op0, op0, 1, 4);              // VL = 4
  op1 = __riscv_vslideup_tu(op1, op1, 2, 4);              // VL = 4
  op1 = __riscv_vslideup_tu(op1, op1, 1, 2);              // VL = 2
  return __riscv_vmerge_vvm_u32m1(op0, op1, odd_mask, 4); // VL = 4
}

INLINE
vuint32m1_t rvv_zip_hi_u32(vuint32m1_t op0, vuint32m1_t op1) {
  vuint32m1_t odd_mask_u32 = __riscv_vmv_v_x_u32m1(0xAAAA, 1);
  vbool32_t odd_mask = __riscv_vreinterpret_v_u32m1_b32(odd_mask_u32);
  op0 = __riscv_vslidedown_tu(op0, op0, 1, 4);            // VL = 4
  op0 = __riscv_vslidedown_tu(op0, op0, 1, 2);            // VL = 2
  op1 = __riscv_vslidedown_tu(op1, op1, 1, 2);            // VL = 2
  return __riscv_vmerge_vvm_u32m1(op0, op1, odd_mask, 4); // VL = 4
}

INLINE
vuint32m1_t rvv_zip_lo_u64(vuint32m1_t op0, vuint32m1_t op1) {
  vuint64m1_t op0_u64 = __riscv_vreinterpret_v_u32m1_u64m1(op0);
  vuint64m1_t op1_u64 = __riscv_vreinterpret_v_u32m1_u64m1(op1);
  vuint64m1_t dst_u64 = __riscv_vslideup_tu(op0_u64, op1_u64, 1, 2); // VL = 2
  return __riscv_vreinterpret_v_u64m1_u32m1(dst_u64);
}

INLINE
vuint32m1_t rvv_zip_hi_u64(vuint32m1_t op0, vuint32m1_t op1) {
  vuint64m1_t op0_u64 = __riscv_vreinterpret_v_u32m1_u64m1(op0);
  vuint64m1_t op1_u64 = __riscv_vreinterpret_v_u32m1_u64m1(op1);
  vuint64m1_t dst_u64 = __riscv_vslidedown_tu(op1_u64, op0_u64, 1, 1); // VL = 1
  return __riscv_vreinterpret_v_u64m1_u32m1(dst_u64);
}

INLINE
vuint32m1_t rvv_shuffle_zip_lo_hi_u256(vuint32m1_t op0, vuint32m1_t op1,
                                       vuint32m1_t tab) {
  op1 = __riscv_vrgather_vv_u32m1(op1, tab, 4);         // VL = 4
  op0 = __riscv_vrgather_vv_u32m1_tu(op1, op0, tab, 2); // VL = 2
  return op0;
}

INLINE
vuint32m1_t rvv_shuffle_u128(vuint32m1_t src, vuint32m1_t tab) {
  return __riscv_vrgather_vv_u32m1(src, tab, 4); // VL = 4
}

INLINE
vuint32m1_t rvv_blend_u16(vuint32m1_t op0, vuint32m1_t op1, uint16_t mask) {
  vuint16m1_t op0_u16 = __riscv_vreinterpret_v_u32m1_u16m1(op0);
  vuint16m1_t op1_u16 = __riscv_vreinterpret_v_u32m1_u16m1(op1);
  vbool16_t mask_u16 = __riscv_vreinterpret_v_u16m1_b16(__riscv_vmv_v_x_u16m1(mask, 1)); 
  vuint16m1_t dst = __riscv_vmerge_vvm_u16m1(op0_u16, op1_u16, mask_u16, 4); // VL = 4
  return __riscv_vreinterpret_v_u16m1_u32m1(dst);
}

/*
 * ----------------------------------------------------------------------------
 * compress_rvv
 * ----------------------------------------------------------------------------
 */

INLINE void g1(vuint32m1_t *row0, vuint32m1_t *row1, vuint32m1_t *row2,
               vuint32m1_t *row3, vuint32m1_t m, size_t vl) {
  *row0 = __riscv_vadd(*row0, m, vl);
  *row0 = __riscv_vadd(*row0, *row1, vl);
  *row3 = __riscv_vxor(*row3, *row0, vl);
  *row3 = rvv_vror_s(*row3, 16, vl);
  *row2 = __riscv_vadd(*row2, *row3, vl);
  *row1 = __riscv_vxor(*row1, *row2, vl);
  *row1 = rvv_vror_s(*row1, 12, vl);
}

INLINE void g2(vuint32m1_t *row0, vuint32m1_t *row1, vuint32m1_t *row2,
               vuint32m1_t *row3, vuint32m1_t m, size_t vl) {
  *row0 = __riscv_vadd(*row0, m, vl);
  *row0 = __riscv_vadd(*row0, *row1, vl);
  *row3 = __riscv_vxor(*row3, *row0, vl);
  *row3 = rvv_vror_s(*row3, 8, vl);
  *row2 = __riscv_vadd(*row2, *row3, vl);
  *row1 = __riscv_vxor(*row1, *row2, vl);
  *row1 = rvv_vror_s(*row1, 7, vl);
}

INLINE void diagonalize(vuint32m1_t *row0, vuint32m1_t *row2, vuint32m1_t *row3,
                        size_t vl) {
  *row0 = rvv_vror_v(*row0, 3, vl);
  *row3 = rvv_vror_v(*row3, 2, vl);
  *row2 = rvv_vror_v(*row2, 1, vl);
}

INLINE void undiagonalize(vuint32m1_t *row0, vuint32m1_t *row2,
                          vuint32m1_t *row3, size_t vl) {
  *row0 = rvv_vror_v(*row0, 1, vl);
  *row3 = rvv_vror_v(*row3, 2, vl);
  *row2 = rvv_vror_v(*row2, 3, vl);
}

INLINE void compress_pre(vuint32m1x4_t *rows, const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags,
                         size_t vl) {
  (void)rows;
  (void)cv;
  (void)block;
  (void)block_len;
  (void)counter;
  (void)flags;
  (void)vl;

  // 0, 0, 3, 3
  // 0, 1, 3, 2
  // 1, 3, 2, 0
  // 2, 0, 2, 0
  // 3, 1, 1, 2
  // 3, 1, 3, 1
  // 3, 3, 2, 2

  // 2, 1, 0, 3 (rotate)
  // 1, 3, 2, 0 (rotate)
  // 0, 3, 2, 1 (rotate)
}

void blake3_compress_xof_rvv(const uint32_t cv[8],
                             const uint8_t block[BLAKE3_BLOCK_LEN],
                             uint8_t block_len, uint64_t counter, uint8_t flags,
                             uint8_t out[64]) {
  assert((uintptr_t)&block[0] % sizeof(uint32_t) == 0); // FIXME: alignment
  assert((uintptr_t)&out[0] % sizeof(uint32_t) == 0);   // FIXME: alignment
  (void)cv;
  (void)block;
  (void)block_len;
  (void)counter;
  (void)flags;
  (void)out;
}

void blake3_compress_in_place_rvv(uint32_t cv[8],
                                  const uint8_t block[BLAKE3_BLOCK_LEN],
                                  uint8_t block_len, uint64_t counter,
                                  uint8_t flags) {
  assert((uintptr_t)&block[0] % sizeof(uint32_t) == 0); // FIXME: alignment
  (void)cv;
  (void)block;
  (void)block_len;
  (void)counter;
  (void)flags;
}

/*
 * ----------------------------------------------------------------------------
 * hash_vl_rvv
 * ----------------------------------------------------------------------------
 */

void round_fn_vl() {
  //
}

void transpose_vecs_vl() {
  //
}

void transpose_msg_vecs_vl() {
  //
}

void load_counters_vl() {
  //
}

void blake3_hash_vl_rvv(const uint8_t *const *inputs, size_t blocks,
                        const uint32_t key[8], uint64_t counter,
                        bool increment_counter, uint8_t flags,
                        uint8_t flags_start, uint8_t flags_end, uint8_t *out,
                        size_t vl) {
  assert((uintptr_t)&inputs[0] % sizeof(uint32_t) == 0); // FIXME: alignment
  assert((uintptr_t)&out[0] % sizeof(uint32_t) == 0);    // FIXME: alignment
  (void)inputs;
  (void)blocks;
  (void)key;
  (void)counter;
  (void)increment_counter;
  (void)flags;
  (void)flags_start;
  (void)flags_end;
  (void)out;
  (void)vl;
}

/*
 * ----------------------------------------------------------------------------
 * hash_many_rvv
 * ----------------------------------------------------------------------------
 */

INLINE void hash_one_rvv(const uint8_t *input, size_t blocks,
                         const uint32_t key[8], uint64_t counter, uint8_t flags,
                         uint8_t flags_start, uint8_t flags_end,
                         uint8_t out[BLAKE3_OUT_LEN]) {
  assert((uintptr_t)&input[0] % sizeof(uint32_t) == 0); // FIXME: alignment
  assert((uintptr_t)&out[0] % sizeof(uint32_t) == 0);   // FIXME: alignment
  uint32_t cv[8];
  memcpy(cv, key, BLAKE3_KEY_LEN);
  uint8_t block_flags = flags | flags_start;
  while (blocks > 0) {
    block_flags |= blocks == 1 ? flags_end : 0;
    if (blocks == 1) {
      block_flags |= flags_end;
    }
    blake3_compress_in_place_rvv(cv, input, BLAKE3_BLOCK_LEN, counter,
                                 block_flags);
    input = &input[BLAKE3_BLOCK_LEN];
    blocks -= 1;
    block_flags = flags;
  }
  memcpy(out, cv, BLAKE3_OUT_LEN);
}

void blake3_hash_many_rvv(const uint8_t *const *inputs, size_t num_inputs,
                          size_t blocks, const uint32_t key[8],
                          uint64_t counter, bool increment_counter,
                          uint8_t flags, uint8_t flags_start, uint8_t flags_end,
                          uint8_t *out) {
  assert((uintptr_t)&inputs[0] % sizeof(uint32_t) == 0); // FIXME: alignment
  assert((uintptr_t)&out[0] % sizeof(uint32_t) == 0);    // FIXME: alignment
  for (
      /* clang-format off */
    size_t vl   = __riscv_vsetvl_e32m1(num_inputs);
    num_inputs  > 0;
    num_inputs -= vl,
        inputs += vl,
       counter += increment_counter * vl,
           out  = &out[vl * BLAKE3_OUT_LEN]
      /* clang-format on */
  ) {
    blake3_hash_vl_rvv(inputs, blocks, key, counter, increment_counter, flags,
                       flags_start, flags_end, out, vl);
  }
}
