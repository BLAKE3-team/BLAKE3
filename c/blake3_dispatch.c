#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "blake3_impl.h"

#if defined(_MSC_VER)
#include <Windows.h>
#endif

#if defined(IS_X86)
#if defined(_MSC_VER)
#include <intrin.h>
#elif defined(__GNUC__)
#include <immintrin.h>
#else
#undef IS_X86 /* Unimplemented! */
#endif
#endif

#if !defined(BLAKE3_ATOMICS)
#if defined(__has_include)
#if __has_include(<stdatomic.h>) && !defined(_MSC_VER)
#define BLAKE3_ATOMICS 1
#else
#define BLAKE3_ATOMICS 0
#endif /* __has_include(<stdatomic.h>) && !defined(_MSC_VER) */
#else
#define BLAKE3_ATOMICS 0
#endif /* defined(__has_include) */
#endif /* BLAKE3_ATOMICS */

#if BLAKE3_ATOMICS
#define ATOMIC_INT _Atomic int
#define ATOMIC_LOAD(x) x
#define ATOMIC_STORE(x, y) x = y
#elif defined(_MSC_VER)
#define ATOMIC_INT LONG
#define ATOMIC_LOAD(x) InterlockedOr(&x, 0)
#define ATOMIC_STORE(x, y) InterlockedExchange(&x, y)
#else
#define ATOMIC_INT int
#define ATOMIC_LOAD(x) x
#define ATOMIC_STORE(x, y) x = y
#endif

#define MAYBE_UNUSED(x) (void)((x))

#if defined(IS_X86) || defined(IS_LOONGARCH)
#if defined(IS_X86)
static uint64_t xgetbv(void) {
#if defined(_MSC_VER)
  return _xgetbv(0);
#else
  uint32_t eax = 0, edx = 0;
  __asm__ __volatile__("xgetbv\n" : "=a"(eax), "=d"(edx) : "c"(0));
  return ((uint64_t)edx << 32) | eax;
#endif
}

static void cpuid(uint32_t out[4], uint32_t id) {
#if defined(_MSC_VER)
  __cpuid((int *)out, id);
#elif defined(__i386__) || defined(_M_IX86)
  __asm__ __volatile__("movl %%ebx, %1\n"
                       "cpuid\n"
                       "xchgl %1, %%ebx\n"
                       : "=a"(out[0]), "=r"(out[1]), "=c"(out[2]), "=d"(out[3])
                       : "a"(id));
#else
  __asm__ __volatile__("cpuid\n"
                       : "=a"(out[0]), "=b"(out[1]), "=c"(out[2]), "=d"(out[3])
                       : "a"(id));
#endif
}

static void cpuidex(uint32_t out[4], uint32_t id, uint32_t sid) {
#if defined(_MSC_VER)
  __cpuidex((int *)out, id, sid);
#elif defined(__i386__) || defined(_M_IX86)
  __asm__ __volatile__("movl %%ebx, %1\n"
                       "cpuid\n"
                       "xchgl %1, %%ebx\n"
                       : "=a"(out[0]), "=r"(out[1]), "=c"(out[2]), "=d"(out[3])
                       : "a"(id), "c"(sid));
#else
  __asm__ __volatile__("cpuid\n"
                       : "=a"(out[0]), "=b"(out[1]), "=c"(out[2]), "=d"(out[3])
                       : "a"(id), "c"(sid));
#endif
}
#endif

#if defined(IS_LOONGARCH)
#ifdef __linux__
static inline int prctl_auxv(void *buf, size_t bufsz) {
  register uint64_t a0 __asm__("a0") = 0x41555856UL; /* PR_GET_AUXV */
  register uint64_t a1 __asm__("a1") = (uint64_t)buf;
  register uint64_t a2 __asm__("a2") = (uint64_t)bufsz;
  register uint64_t a3 __asm__("a3") = 0;
  register uint64_t a4 __asm__("a4") = 0;
  register uint64_t a7 __asm__("a7") = 167; /* __NR_prctl */
  __asm__ __volatile__("syscall 0\n\t"
    : "+r"(a0)
    : "r"(a7), "r"(a1), "r"(a2), "r"(a3), "r"(a4)
    : "memory", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8");
  return a0;
}
#endif /* __linux__ */

static uint32_t get_hwcap(void) {
#ifdef __linux__
  int bufsz = prctl_auxv(NULL, 0);
  if (bufsz < 0)
    return 0;
  unsigned long *buf = __builtin_alloca(bufsz);
  if (prctl_auxv(buf, bufsz) < 0)
    return 0;
  for (size_t i = 0; i < bufsz / (2 * sizeof(unsigned long)); ++i)
    if (buf[2 * i] == 16 /* AT_HWCAP */)
      return (uint32_t)buf[2 * i + 1] >> 4;
  return 0;
#else
  return __builtin_loongarch_cpucfg(2) >> 6;
#endif
}
#endif /* IS_LOONGARCH */

enum cpu_feature {
  SSE2 = 1 << 0,
  SSSE3 = 1 << 1,
  SSE41 = 1 << 2,
  AVX = 1 << 3,
  AVX2 = 1 << 4,
  AVX512F = 1 << 5,
  AVX512VL = 1 << 6,
  LSX = 1 << 0,
  LASX = 1 << 1,
  /* ... */
  UNDEFINED = 1 << 30
};

#if !defined(BLAKE3_TESTING)
static /* Allow the variable to be controlled manually for testing */
#endif
    ATOMIC_INT g_cpu_features = UNDEFINED;

#if !defined(BLAKE3_TESTING)
static
#endif
    enum cpu_feature
    get_cpu_features(void) {

  /* If TSAN detects a data race here, try compiling with -DBLAKE3_ATOMICS=1 */
  enum cpu_feature features = ATOMIC_LOAD(g_cpu_features);
  if (features != UNDEFINED) {
    return features;
  } else {
#if defined(IS_X86)
    uint32_t regs[4] = {0};
    uint32_t *eax = &regs[0], *ebx = &regs[1], *ecx = &regs[2], *edx = &regs[3];
    (void)edx;
    features = 0;
    cpuid(regs, 0);
    const int max_id = *eax;
    cpuid(regs, 1);
#if defined(__amd64__) || defined(_M_X64)
    features |= SSE2;
#else
    if (*edx & (1UL << 26))
      features |= SSE2;
#endif
    if (*ecx & (1UL << 9))
      features |= SSSE3;
    if (*ecx & (1UL << 19))
      features |= SSE41;

    if (*ecx & (1UL << 27)) { // OSXSAVE
      const uint64_t mask = xgetbv();
      if ((mask & 6) == 6) { // SSE and AVX states
        if (*ecx & (1UL << 28))
          features |= AVX;
        if (max_id >= 7) {
          cpuidex(regs, 7, 0);
          if (*ebx & (1UL << 5))
            features |= AVX2;
          if ((mask & 224) == 224) { // Opmask, ZMM_Hi256, Hi16_Zmm
            if (*ebx & (1UL << 31))
              features |= AVX512VL;
            if (*ebx & (1UL << 16))
              features |= AVX512F;
          }
        }
      }
    }
    ATOMIC_STORE(g_cpu_features, features);
    return features;
#elif defined(IS_LOONGARCH)
    uint32_t hwcap = get_hwcap();
    features = hwcap & (LSX | LASX);
    ATOMIC_STORE(g_cpu_features, features);
    return features;
#else
    /* How to detect NEON? */
    return 0;
#endif
  }
}
#endif

void blake3_compress_in_place(uint32_t cv[8],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t counter,
                              uint8_t flags) {
#if defined(IS_X86)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_AVX512)
  if (features & AVX512VL) {
    blake3_compress_in_place_avx512(cv, block, block_len, counter, flags);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE41)
  if (features & SSE41) {
    blake3_compress_in_place_sse41(cv, block, block_len, counter, flags);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE2)
  if (features & SSE2) {
    blake3_compress_in_place_sse2(cv, block, block_len, counter, flags);
    return;
  }
#endif
#endif
  blake3_compress_in_place_portable(cv, block, block_len, counter, flags);
}

void blake3_compress_xof(const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags,
                         uint8_t out[64]) {
#if defined(IS_X86)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_AVX512)
  if (features & AVX512VL) {
    blake3_compress_xof_avx512(cv, block, block_len, counter, flags, out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE41)
  if (features & SSE41) {
    blake3_compress_xof_sse41(cv, block, block_len, counter, flags, out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE2)
  if (features & SSE2) {
    blake3_compress_xof_sse2(cv, block, block_len, counter, flags, out);
    return;
  }
#endif
#endif
  blake3_compress_xof_portable(cv, block, block_len, counter, flags, out);
}


void blake3_xof_many(const uint32_t cv[8],
                     const uint8_t block[BLAKE3_BLOCK_LEN],
                     uint8_t block_len, uint64_t counter, uint8_t flags,
                     uint8_t out[64], size_t outblocks) {
  if (outblocks == 0) {
    // The current assembly implementation always outputs at least 1 block.
    return;
  }
#if defined(IS_X86)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(_WIN32) && !defined(BLAKE3_NO_AVX512)
  if (features & AVX512VL) {
    blake3_xof_many_avx512(cv, block, block_len, counter, flags, out, outblocks);
    return;
  }
#endif
#endif
  for(size_t i = 0; i < outblocks; ++i) {
    blake3_compress_xof(cv, block, block_len, counter + i, flags, out + 64*i);
  }
}

void blake3_hash_many(const uint8_t *const *inputs, size_t num_inputs,
                      size_t blocks, const uint32_t key[8], uint64_t counter,
                      bool increment_counter, uint8_t flags,
                      uint8_t flags_start, uint8_t flags_end, uint8_t *out) {
#if defined(IS_X86)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_AVX512)
  if ((features & (AVX512F|AVX512VL)) == (AVX512F|AVX512VL)) {
    blake3_hash_many_avx512(inputs, num_inputs, blocks, key, counter,
                            increment_counter, flags, flags_start, flags_end,
                            out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_AVX2)
  if (features & AVX2) {
    blake3_hash_many_avx2(inputs, num_inputs, blocks, key, counter,
                          increment_counter, flags, flags_start, flags_end,
                          out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE41)
  if (features & SSE41) {
    blake3_hash_many_sse41(inputs, num_inputs, blocks, key, counter,
                           increment_counter, flags, flags_start, flags_end,
                           out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_SSE2)
  if (features & SSE2) {
    blake3_hash_many_sse2(inputs, num_inputs, blocks, key, counter,
                          increment_counter, flags, flags_start, flags_end,
                          out);
    return;
  }
#endif
#endif

#if BLAKE3_USE_NEON == 1
  blake3_hash_many_neon(inputs, num_inputs, blocks, key, counter,
                        increment_counter, flags, flags_start, flags_end, out);
  return;
#endif

#if defined(IS_LOONGARCH)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_LASX)
  if (features & LASX) {
    blake3_hash_many_lasx(inputs, num_inputs, blocks, key, counter,
                          increment_counter, flags, flags_start, flags_end,
                          out);
    return;
  }
#endif
#if !defined(BLAKE3_NO_LSX)
  if (features & LSX) {
    blake3_hash_many_lsx(inputs, num_inputs, blocks, key, counter,
                         increment_counter, flags, flags_start, flags_end, out);
    return;
  }
#endif
#endif /* IS_LOONGARCH */

  blake3_hash_many_portable(inputs, num_inputs, blocks, key, counter,
                            increment_counter, flags, flags_start, flags_end,
                            out);
}

// The dynamically detected SIMD degree of the current platform.
size_t blake3_simd_degree(void) {
#if defined(IS_X86)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_AVX512)
  if ((features & (AVX512F|AVX512VL)) == (AVX512F|AVX512VL)) {
    return 16;
  }
#endif
#if !defined(BLAKE3_NO_AVX2)
  if (features & AVX2) {
    return 8;
  }
#endif
#if !defined(BLAKE3_NO_SSE41)
  if (features & SSE41) {
    return 4;
  }
#endif
#if !defined(BLAKE3_NO_SSE2)
  if (features & SSE2) {
    return 4;
  }
#endif
#endif
#if BLAKE3_USE_NEON == 1
  return 4;
#endif
#if defined(IS_LOONGARCH)
  const enum cpu_feature features = get_cpu_features();
  MAYBE_UNUSED(features);
#if !defined(BLAKE3_NO_LASX)
  if (features & LASX) {
    return 8;
  }
#endif
#if !defined(BLAKE3_NO_LSX)
  if (features & LSX) {
    return 4;
  }
#endif
#endif /* IS_LOONGARCH */
  return 1;
}
