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

static const uint8_t MSG_SCHEDULE[7][16] = {
    {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15},
    {14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3},
    {11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4},
    {7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8},
    {9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13},
    {2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9},
    {12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11},
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

INLINE uint32_t counter_low(uint64_t counter) { return (uint32_t)counter; }

INLINE uint32_t counter_high(uint64_t counter) {
  return (uint32_t)(counter >> 32);
}

INLINE uint32_t load32(const void *src) {
  const uint8_t *p = (const uint8_t *)src;
  return ((uint32_t)(p[0]) << 0) | ((uint32_t)(p[1]) << 8) |
         ((uint32_t)(p[2]) << 16) | ((uint32_t)(p[3]) << 24);
}

INLINE void load_key_words(const uint8_t key[BLAKE3_KEY_LEN],
                           uint32_t key_words[8]) {
  key_words[0] = load32(&key[0 * 4]);
  key_words[1] = load32(&key[1 * 4]);
  key_words[2] = load32(&key[2 * 4]);
  key_words[3] = load32(&key[3 * 4]);
  key_words[4] = load32(&key[4 * 4]);
  key_words[5] = load32(&key[5 * 4]);
  key_words[6] = load32(&key[6 * 4]);
  key_words[7] = load32(&key[7 * 4]);
}

// Declarations for implementation-specific functions.
void blake3_compress_in_place_portable(uint32_t cv[8],
                                       const uint8_t block[BLAKE3_BLOCK_LEN],
                                       uint8_t block_len, uint64_t counter,
                                       uint8_t flags);
void blake3_compress_in_place_sse41(uint32_t cv[8],
                                    const uint8_t block[BLAKE3_BLOCK_LEN],
                                    uint8_t block_len, uint64_t counter,
                                    uint8_t flags);
void blake3_compress_in_place_avx512(uint32_t cv[8],
                                     const uint8_t block[BLAKE3_BLOCK_LEN],
                                     uint8_t block_len, uint64_t counter,
                                     uint8_t flags);
void blake3_compress_xof_portable(const uint32_t cv[8],
                                  const uint8_t block[BLAKE3_BLOCK_LEN],
                                  uint8_t block_len, uint64_t counter,
                                  uint8_t flags, uint8_t out[64]);
void blake3_compress_xof_sse41(const uint32_t cv[8],
                               const uint8_t block[BLAKE3_BLOCK_LEN],
                               uint8_t block_len, uint64_t counter,
                               uint8_t flags, uint8_t out[64]);
void blake3_compress_xof_avx512(const uint32_t cv[8],
                                const uint8_t block[BLAKE3_BLOCK_LEN],
                                uint8_t block_len, uint64_t counter,
                                uint8_t flags, uint8_t out[64]);
void blake3_hash_many_portable(const uint8_t *const *inputs, size_t num_inputs,
                               size_t blocks, const uint32_t key[8],
                               uint64_t counter, bool increment_counter,
                               uint8_t flags, uint8_t flags_start,
                               uint8_t flags_end, uint8_t *out);
void blake3_hash_many_sse41(const uint8_t *const *inputs, size_t num_inputs,
                            size_t blocks, const uint32_t key[8],
                            uint64_t counter, bool increment_counter,
                            uint8_t flags, uint8_t flags_start,
                            uint8_t flags_end, uint8_t *out);
void blake3_hash_many_avx2(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out);
void blake3_hash_many_avx512(const uint8_t *const *inputs, size_t num_inputs,
                             size_t blocks, const uint32_t key[8],
                             uint64_t counter, bool increment_counter,
                             uint8_t flags, uint8_t flags_start,
                             uint8_t flags_end, uint8_t *out);
void blake3_hash_many_neon(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out);
