#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "blake3_impl.h"

#if defined(IS_ARMHF) && defined(__linux__)
#include <sys/auxv.h>
#include <asm/hwcap.h>
#endif

#if defined(IS_ARMHF)
#define IFUNC_RESOLVER_ARGS uint64_t hwcap
#else
#define IFUNC_RESOLVER_ARGS void
#endif

void blake3_compress_in_place(uint32_t cv[8],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t counter,
                              uint8_t flags) __attribute__((ifunc ("resolve_compress_in_place")));

typedef void (*f_blake3_compress_in_place)(uint32_t *, const uint8_t *, uint8_t, uint64_t, uint8_t);
static f_blake3_compress_in_place resolve_compress_in_place(IFUNC_RESOLVER_ARGS) {
#if defined(IS_X86)
  __builtin_cpu_init();
  if (__builtin_cpu_supports("avx512vl"))
    return blake3_compress_in_place_avx512;
  else if (__builtin_cpu_supports("sse4.1"))
    return blake3_compress_in_place_sse41;
  else
#endif
    return blake3_compress_in_place_portable;
}

void blake3_compress_xof(const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags,
                         uint8_t out[64]) __attribute__((ifunc ("resolve_compress_xof")));

typedef void (*f_blake3_compress_xof)(const uint32_t *, const uint8_t *, uint8_t, uint64_t, uint8_t, uint8_t *);
static f_blake3_compress_xof resolve_compress_xof(IFUNC_RESOLVER_ARGS) {
#if defined(IS_X86)
  __builtin_cpu_init();
  if (__builtin_cpu_supports("avx512vl"))
    return blake3_compress_xof_avx512;
  else if (__builtin_cpu_supports("sse4.1"))
    return blake3_compress_xof_sse41;
  else
#endif
    return blake3_compress_xof_portable;
}

void blake3_hash_many(const uint8_t *const *inputs, size_t num_inputs,
                      size_t blocks, const uint32_t key[8], uint64_t counter,
                      bool increment_counter, uint8_t flags,
                      uint8_t flags_start, uint8_t flags_end, uint8_t *out) __attribute__((ifunc ("resolve_hash_many")));

typedef void(*f_blake3_hash_many)(const uint8_t * const *, size_t, size_t, const uint32_t *, uint64_t, bool, uint8_t, uint8_t, uint8_t, uint8_t *);
static f_blake3_hash_many resolve_hash_many(IFUNC_RESOLVER_ARGS) {
#if defined(IS_X86)
  __builtin_cpu_init();
  if (__builtin_cpu_supports("avx512f") && __builtin_cpu_supports("avx512vl"))
    return blake3_hash_many_avx512;
  else if (__builtin_cpu_supports("avx2"))
    return blake3_hash_many_avx2;
  else if (__builtin_cpu_supports("sse4.1"))
    return blake3_hash_many_sse41;
  else
    return blake3_hash_many_portable;
#elif defined(IS_ARM64)
  return blake3_hash_many_neon;
#elif defined(IS_ARMHF)
  if (hwcap & HWCAP_ARM_NEON)
    return blake3_hash_many_neon;
  else
    return blake3_hash_many_portable;
#elif
  return blake3_hash_many_portable;
#endif
}

// The dynamically detected SIMD degree of the current platform.
size_t blake3_simd_degree(void) {
#if defined(IS_X86)
  __builtin_cpu_init();
  if (__builtin_cpu_supports("avx512f") && __builtin_cpu_supports("avx512vl"))
    return 16;
  if (__builtin_cpu_supports("avx2"))
    return 8;
  if (__builtin_cpu_supports("sse4.1"))
    return 4;
#endif
#if defined(IS_ARM64)
  return 4;
#endif
#if defined(IS_ARMHF) && defined(__linux__)
  if (getauxval(AT_HWCAP) & HWCAP_ARM_NEON)
    return 4;
#endif
  return 1;
}

enum cpu_feature {
  SSE2 = 1 << 0,
  SSSE3 = 1 << 1,
  SSE41 = 1 << 2,
  AVX = 1 << 3,
  AVX2 = 1 << 4,
  AVX512F = 1 << 5,
  AVX512VL = 1 << 6,
  /* ... */
  UNDEFINED = 1 << 30
};
// ifunc resolves functions at elf startup thus it's not possible to change features at runtime
enum cpu_feature g_cpu_features = 0;
enum cpu_feature get_cpu_features() {
  return 0;
}
