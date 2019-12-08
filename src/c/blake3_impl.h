#pragma once

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#if __POPCNT__
#include <nmmintrin.h>
#endif

#include "blake3.h"

// internal flags
#define CHUNK_START 1
#define CHUNK_END 2
#define PARENT 4
#define ROOT 8
#define KEYED_HASH 16
#define DERIVE_KEY 32

// This C implementation tries to support recent versions of GCC, Clang, and
// MSVC.
#if defined(_MSC_VER)
#define INLINE __forceinline static
#else
#define INLINE __attribute__((always_inline)) static inline
#endif

static const uint32_t IV[8] = {0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL,
                               0xA54FF53AUL, 0x510E527FUL, 0x9B05688CUL,
                               0x1F83D9ABUL, 0x5BE0CD19UL};

static const uint8_t IV_BYTES[32] = {
    0x67, 0xe6, 0x09, 0x6a, 0x85, 0xae, 0x67, 0xbb, 0x72, 0xf3, 0x6e,
    0x3c, 0x3a, 0xf5, 0x4f, 0xa5, 0x7f, 0x52, 0x0e, 0x51, 0x8c, 0x68,
    0x05, 0x9b, 0xab, 0xd9, 0x83, 0x1f, 0x19, 0xcd, 0xe0, 0x5b,
};

static const uint8_t MSG_SCHEDULE[7][16] = {
    {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15},
    {14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3},
    {11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4},
    {7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8},
    {9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13},
    {2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9},
    {12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11},
};

// 17 is 1 + the largest supported SIMD degree. Each hash_many() implementation
// can thus do `offset += offset_deltas[DEGREE]` at the end of each batch.
typedef const uint64_t offset_deltas_t[17];

static offset_deltas_t CHUNK_OFFSET_DELTAS = {
    BLAKE3_CHUNK_LEN * 0,  BLAKE3_CHUNK_LEN * 1,  BLAKE3_CHUNK_LEN * 2,
    BLAKE3_CHUNK_LEN * 3,  BLAKE3_CHUNK_LEN * 4,  BLAKE3_CHUNK_LEN * 5,
    BLAKE3_CHUNK_LEN * 6,  BLAKE3_CHUNK_LEN * 7,  BLAKE3_CHUNK_LEN * 8,
    BLAKE3_CHUNK_LEN * 9,  BLAKE3_CHUNK_LEN * 10, BLAKE3_CHUNK_LEN * 11,
    BLAKE3_CHUNK_LEN * 12, BLAKE3_CHUNK_LEN * 13, BLAKE3_CHUNK_LEN * 14,
    BLAKE3_CHUNK_LEN * 15, BLAKE3_CHUNK_LEN * 16,
};

// Count the number of 1 bits.
INLINE uint8_t popcnt(uint64_t x) {
#if __POPCNT__
  return (uint8_t)_mm_popcnt_u64(x);
#else
  uint8_t count = 0;
  while (x > 0) {
    count += ((uint8_t)x) & 1;
    x >>= 1;
  }
  return count;
#endif
}

INLINE uint32_t offset_low(uint64_t offset) { return (uint32_t)offset; }

INLINE uint32_t offset_high(uint64_t offset) {
  return (uint32_t)(offset >> 32);
}

// Declarations for implementation-specific functions.
void blake3_compress_portable(const uint8_t cv[BLAKE3_OUT_LEN],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t offset, uint8_t flags,
                              uint8_t out[64]);
void blake3_compress_sse41(const uint8_t cv[BLAKE3_OUT_LEN],
                           const uint8_t block[BLAKE3_BLOCK_LEN],
                           uint8_t block_len, uint64_t offset, uint8_t flags,
                           uint8_t out[64]);
void blake3_compress_avx512(const uint8_t cv[BLAKE3_OUT_LEN],
                            const uint8_t block[BLAKE3_BLOCK_LEN],
                            uint8_t block_len, uint64_t offset, uint8_t flags,
                            uint8_t out[64]);
void blake3_hash_many_portable(const uint8_t *const *inputs, size_t num_inputs,
                               size_t blocks, const uint8_t key[BLAKE3_KEY_LEN],
                               uint64_t offset, offset_deltas_t od,
                               uint8_t flags, uint8_t flags_start,
                               uint8_t flags_end, uint8_t *out);
void blake3_hash_many_sse41(const uint8_t *const *inputs, size_t num_inputs,
                            size_t blocks, const uint8_t key[BLAKE3_KEY_LEN],
                            uint64_t offset, offset_deltas_t od, uint8_t flags,
                            uint8_t flags_start, uint8_t flags_end,
                            uint8_t *out);
void blake3_hash_many_avx2(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint8_t key[BLAKE3_KEY_LEN],
                           uint64_t offset, offset_deltas_t od, uint8_t flags,
                           uint8_t flags_start, uint8_t flags_end,
                           uint8_t *out);
void blake3_hash_many_avx512(const uint8_t *const *inputs, size_t num_inputs,
                             size_t blocks, const uint8_t key[BLAKE3_KEY_LEN],
                             uint64_t offset, offset_deltas_t od, uint8_t flags,
                             uint8_t flags_start, uint8_t flags_end,
                             uint8_t *out);
void blake3_hash_many_neon(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint8_t key[BLAKE3_KEY_LEN],
                           uint64_t offset, offset_deltas_t od, uint8_t flags,
                           uint8_t flags_start, uint8_t flags_end,
                           uint8_t *out);
