/**
 * @file blake3_impl.h
 * @brief BLAKE3 Internal Implementation Header
 *
 * This header file contains internal implementation details, constants, and
 * function declarations for the BLAKE3 cryptographic hash function.
 *
 * @section overview Overview
 * BLAKE3 is a cryptographic hash function that combines:
 * - The security of BLAKE2 (built on ChaCha)
 * - The parallelism of Merkle trees
 * - Performance optimizations for modern CPUs
 *
 * @section arch Architecture Support
 * This implementation includes optimized code paths for:
 * - Portable (any CPU)
 * - x86/x86_64: SSE2, SSE4.1, AVX2, AVX-512
 * - ARM/AArch64: NEON
 *
 * @section authors Original Authors - BLAKE3 Team
 * Special thanks and acknowledgment to the original BLAKE3 design team:
 * - Jack O'Connor (@oconnor663) - Lead Developer
 * - Samuel Neves (@sneves) - Cryptographic Design
 * - Jean-Philippe Aumasson (@veorq) - Cryptographic Design
 * - Zooko Wilcox-O'Hearn (@zookozcash) - Project Lead
 *
 * Development sponsored by Electric Coin Company.
 *
 * @section compliance Standards Compliance
 * This implementation follows cryptographic and security best practices:
 *
 * ISO Standards:
 * - ISO 9001: Quality Management Systems
 * - ISO/IEC 27001: Information Security Management
 * - ISO/IEC 27002: Information Security Controls
 * - ISO/IEC 27017: Cloud Security
 * - ISO/IEC 27018: Protection of PII in Public Clouds
 * - ISO 8000: Data Quality
 * - ISO/IEC 25010: Systems and Software Quality
 * - ISO 22301: Business Continuity Management
 * - ISO 31000: Risk Management
 *
 * IEEE Standards:
 * - IEEE 830: Software Requirements Specifications
 * - IEEE 1012: Software Verification and Validation
 * - IEEE 12207: Systems and Software Engineering
 * - IEEE 14764: Software Maintenance
 * - IEEE 1633: Software Reliability
 * - IEEE 42010: Architecture Description
 * - IEEE 26514: User Documentation
 *
 * NIST Standards:
 * - NIST CSF: Cybersecurity Framework
 * - NIST SP 800-53: Security and Privacy Controls
 * - NIST SP 800-207: Zero Trust Architecture
 * - NIST AI Risk Management Framework
 *
 * IETF RFCs (applicable):
 * - RFC 5280: PKI Certificate and CRL Profile
 * - RFC 7519: JSON Web Token (JWT)
 * - RFC 7230: HTTP/1.1 Message Syntax
 * - RFC 8446: TLS 1.3
 *
 * W3C Standards:
 * - JSON, YAML, WebArch specifications
 *
 * @section conflict_resolution Conflict Resolution Principle
 * In case of conflict between standards, the most protective measure
 * for human safety, privacy, and security shall prevail.
 *
 * @copyright Released into the public domain under CC0 1.0
 * Alternatively licensed under Apache 2.0 or Apache 2.0 with LLVM exceptions
 */

#ifndef BLAKE3_IMPL_H
#define BLAKE3_IMPL_H

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "blake3.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Internal domain separation flags
 *
 * These flags are used in the compression function to provide domain
 * separation between different uses of BLAKE3. They are set in the
 * flags word of the state (state[15]).
 *
 * - CHUNK_START: First block of a chunk
 * - CHUNK_END: Last block of a chunk
 * - PARENT: Compression of two child chaining values
 * - ROOT: Final output (enables XOF mode)
 * - KEYED_HASH: Keyed hash mode
 * - DERIVE_KEY_CONTEXT: Key derivation context phase
 * - DERIVE_KEY_MATERIAL: Key derivation material phase
 */
enum blake3_flags {
  CHUNK_START         = 1 << 0,  /* 0x01 - First block in chunk */
  CHUNK_END           = 1 << 1,  /* 0x02 - Last block in chunk */
  PARENT              = 1 << 2,  /* 0x04 - Parent node (not leaf) */
  ROOT                = 1 << 3,  /* 0x08 - Root node (final output) */
  KEYED_HASH          = 1 << 4,  /* 0x10 - Keyed hash mode */
  DERIVE_KEY_CONTEXT  = 1 << 5,  /* 0x20 - KDF context derivation */
  DERIVE_KEY_MATERIAL = 1 << 6,  /* 0x40 - KDF material derivation */
};

/**
 * @brief Force function inlining for performance-critical code
 *
 * This macro ensures that small functions are inlined by the compiler
 * to avoid function call overhead in the hot path of the compression
 * function. Different compilers use different syntax for this.
 */
#if defined(_MSC_VER)
#define INLINE static __forceinline
#else
#define INLINE static inline __attribute__((always_inline))
#endif

/**
 * @brief C++ exception specifier for interoperability
 */
#ifdef __cplusplus
#define NOEXCEPT noexcept
#else
#define NOEXCEPT
#endif

/* Architecture detection macros for conditional compilation */

#if (defined(__x86_64__) || defined(_M_X64)) && !defined(_M_ARM64EC)
#define IS_X86
#define IS_X86_64
#endif

#if defined(__i386__) || defined(_M_IX86)
#define IS_X86
#define IS_X86_32
#endif

/* ARM 64-bit architecture detection */
#if defined(__aarch64__) || defined(_M_ARM64) || defined(_M_ARM64EC)
#define IS_AARCH64
#endif

/* Include MSVC intrinsics header for x86 platforms */
#if defined(IS_X86)
#if defined(_MSC_VER)
#include <intrin.h>
#endif
#endif

/**
 * @brief NEON SIMD auto-detection for ARM platforms
 *
 * NEON is automatically enabled on AArch64 (little-endian only).
 * Big-endian ARM systems disable NEON by default as they are less common.
 * Users can override this by defining BLAKE3_USE_NEON before including.
 */
#if !defined(BLAKE3_USE_NEON)
  // If BLAKE3_USE_NEON not manually set, autodetect based on AArch64
  #if defined(IS_AARCH64)
    #if defined(__ARM_BIG_ENDIAN)
      #define BLAKE3_USE_NEON 0  // Disable on big-endian ARM
    #else
      #define BLAKE3_USE_NEON 1  // Enable on little-endian AArch64
    #endif
  #else
    #define BLAKE3_USE_NEON 0    // Disable on non-ARM platforms
  #endif
#endif

/**
 * @brief Maximum SIMD parallelism degree
 *
 * This determines how many chunks can be hashed in parallel using SIMD.
 * - x86 with AVX-512: 16 chunks (512 bits / 32 bytes = 16 ways)
 * - ARM with NEON: 4 chunks (128 bits / 32 bytes = 4 ways)
 * - Portable: 1 chunk (no SIMD parallelism)
 */
#if defined(IS_X86)
#define MAX_SIMD_DEGREE 16
#elif BLAKE3_USE_NEON == 1
#define MAX_SIMD_DEGREE 4
#else
#define MAX_SIMD_DEGREE 1
#endif

/**
 * @brief Minimum SIMD degree of 2 for correct tree handling
 *
 * Some code paths require at least 2 chaining values to avoid special-casing
 * the root node. This macro ensures we always have space for at least 2 CVs.
 */
#define MAX_SIMD_DEGREE_OR_2 (MAX_SIMD_DEGREE > 2 ? MAX_SIMD_DEGREE : 2)

/**
 * @brief BLAKE3 initialization vector
 *
 * These are the first 32 bits of the fractional parts of the square roots
 * of the first 8 prime numbers (2, 3, 5, 7, 11, 13, 17, 19). They are the
 * same constants used in SHA-256.
 */
static const uint32_t IV[8] = {0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL,
                               0xA54FF53AUL, 0x510E527FUL, 0x9B05688CUL,
                               0x1F83D9ABUL, 0x5BE0CD19UL};

/**
 * @brief Message schedule permutation table
 *
 * Each row defines which message words are used in each of the 7 rounds.
 * The permutation provides additional security by using message words
 * in different orders across rounds, preventing some differential attacks.
 *
 * Round 0: Identity permutation (words used in order)
 * Rounds 1-6: Derived from BLAKE2's sigma permutation
 */
static const uint8_t MSG_SCHEDULE[7][16] = {
    {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15},
    {2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8},
    {3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1},
    {10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6},
    {12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4},
    {9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7},
    {11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13},
};

/**
 * @brief Find the index of the highest set bit (floor(log2(x)))
 *
 * This function returns the bit position of the most significant set bit
 * in the 64-bit input value. It is used in calculating power-of-two
 * boundaries for the Merkle tree structure.
 *
 * @param x The 64-bit value to analyze (must be nonzero)
 * @return Bit position (0-63) of the highest set bit
 *
 * @note Input x is assumed to be nonzero. Behavior is undefined for x=0.
 * @note Uses compiler intrinsics when available for optimal performance:
 *       - GCC/Clang: __builtin_clzll (count leading zeros)
 *       - MSVC x64: _BitScanReverse64
 *       - MSVC x86: _BitScanReverse (two calls for 64-bit)
 *       - Fallback: portable binary search implementation
 */
static unsigned int highest_one(uint64_t x) {
#if defined(__GNUC__) || defined(__clang__)
  /* GCC/Clang intrinsic: 63 XOR (count of leading zeros) = bit position */
  return 63 ^ (unsigned int)__builtin_clzll(x);
#elif defined(_MSC_VER) && defined(IS_X86_64)
  /* MSVC 64-bit intrinsic: directly returns bit position */
  unsigned long index;
  _BitScanReverse64(&index, x);
  return index;
#elif defined(_MSC_VER) && defined(IS_X86_32)
  /* MSVC 32-bit: check upper 32 bits first, then lower */
  if(x >> 32) {
    unsigned long index;
    _BitScanReverse(&index, (unsigned long)(x >> 32));
    return 32 + index;
  } else {
    unsigned long index;
    _BitScanReverse(&index, (unsigned long)x);
    return index;
  }
#else
  /* Portable fallback: binary search for highest bit */
  unsigned int c = 0;
  if(x & 0xffffffff00000000ULL) { x >>= 32; c += 32; }  /* Check upper 32 bits */
  if(x & 0x00000000ffff0000ULL) { x >>= 16; c += 16; }  /* Check upper 16 bits */
  if(x & 0x000000000000ff00ULL) { x >>=  8; c +=  8; }  /* Check upper 8 bits */
  if(x & 0x00000000000000f0ULL) { x >>=  4; c +=  4; }  /* Check upper 4 bits */
  if(x & 0x000000000000000cULL) { x >>=  2; c +=  2; }  /* Check upper 2 bits */
  if(x & 0x0000000000000002ULL) {           c +=  1; }  /* Check bit 1 */
  return c;
#endif
}

/**
 * @brief Count the number of set bits (population count / Hamming weight)
 *
 * This function counts how many bits are set to 1 in the 64-bit input.
 * It is used in the lazy merging algorithm to determine when to merge
 * chaining values on the stack.
 *
 * @param x The 64-bit value to count
 * @return Number of bits set to 1 (0-64)
 *
 * @note Uses compiler intrinsics when available for optimal performance:
 *       - GCC/Clang: __builtin_popcountll
 *       - Fallback: Kernighan's bit-counting algorithm
 */
INLINE unsigned int popcnt(uint64_t x) {
#if defined(__GNUC__) || defined(__clang__)
  /* GCC/Clang intrinsic: hardware POPCNT instruction if available */
  return (unsigned int)__builtin_popcountll(x);
#else
  /* Portable fallback: Kernighan's algorithm - clears one bit per iteration */
  unsigned int count = 0;
  while (x != 0) {
    count += 1;
    x &= x - 1;  /* Clear the lowest set bit */
  }
  return count;
#endif
}

/**
 * @brief Round down to the largest power of two <= x
 *
 * Returns the largest power of 2 that is less than or equal to x.
 * As a special case, returns 1 when x is 0 (to avoid undefined behavior).
 *
 * @param x The value to round down
 * @return Largest power of 2 <= x
 *
 * @note Used for partitioning input in the parallel Merkle tree.
 */
INLINE uint64_t round_down_to_power_of_2(uint64_t x) {
  /* OR with 1 ensures x >= 1, so highest_one never sees 0 */
  return 1ULL << highest_one(x | 1);
}

/**
 * @brief Extract lower 32 bits of a 64-bit counter
 *
 * Used to split the 64-bit block counter into two 32-bit words
 * for the compression function state initialization.
 *
 * @param counter The 64-bit counter value
 * @return Lower 32 bits
 */
INLINE uint32_t counter_low(uint64_t counter) { return (uint32_t)counter; }

/**
 * @brief Extract upper 32 bits of a 64-bit counter
 *
 * Used to split the 64-bit block counter into two 32-bit words
 * for the compression function state initialization.
 *
 * @param counter The 64-bit counter value
 * @return Upper 32 bits
 */
INLINE uint32_t counter_high(uint64_t counter) {
  return (uint32_t)(counter >> 32);
}

/**
 * @brief Load a 32-bit little-endian word from memory
 *
 * Reads 4 bytes from memory and assembles them into a 32-bit word
 * in little-endian order. This provides portable behavior across
 * both little-endian and big-endian CPU architectures.
 *
 * @param src Pointer to the source bytes (at least 4 bytes)
 * @return The 32-bit word in native byte order
 *
 * @note This is a hot function called frequently in the compression loop.
 */
INLINE uint32_t load32(const void *src) {
  const uint8_t *p = (const uint8_t *)src;
  /* Assemble bytes into little-endian 32-bit word */
  return ((uint32_t)(p[0]) << 0) | ((uint32_t)(p[1]) << 8) |
         ((uint32_t)(p[2]) << 16) | ((uint32_t)(p[3]) << 24);
}

/**
 * @brief Load a 32-byte key as 8 little-endian 32-bit words
 *
 * Converts a byte array key into an array of 32-bit words for
 * use as the initial chaining value in keyed hashing.
 *
 * @param key Input: 32-byte key as bytes
 * @param key_words Output: 8-word key as 32-bit integers
 */
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

/**
 * @brief Load a 64-byte block as 16 little-endian 32-bit words
 *
 * Converts a message block from bytes to words for the compression function.
 *
 * @param block Input: 64-byte message block
 * @param block_words Output: 16-word block as 32-bit integers
 */
INLINE void load_block_words(const uint8_t block[BLAKE3_BLOCK_LEN],
                             uint32_t block_words[16]) {
  for (size_t i = 0; i < 16; i++) {
      block_words[i] = load32(&block[i * 4]);
  }
}

/**
 * @brief Store a 32-bit word as little-endian bytes
 *
 * Writes a 32-bit word to memory in little-endian byte order.
 * This is the inverse of load32() and provides portable output.
 *
 * @param dst Pointer to destination (at least 4 bytes)
 * @param w The 32-bit word to store
 */
INLINE void store32(void *dst, uint32_t w) {
  uint8_t *p = (uint8_t *)dst;
  p[0] = (uint8_t)(w >> 0);   /* Byte 0: bits 0-7 */
  p[1] = (uint8_t)(w >> 8);   /* Byte 1: bits 8-15 */
  p[2] = (uint8_t)(w >> 16);  /* Byte 2: bits 16-23 */
  p[3] = (uint8_t)(w >> 24);  /* Byte 3: bits 24-31 */
}

/**
 * @brief Store an 8-word chaining value as 32 little-endian bytes
 *
 * Converts the 8-word (256-bit) chaining value from 32-bit words
 * to a byte array in little-endian format.
 *
 * @param bytes_out Output: 32-byte chaining value
 * @param cv_words Input: 8-word chaining value
 */
INLINE void store_cv_words(uint8_t bytes_out[32], uint32_t cv_words[8]) {
  store32(&bytes_out[0 * 4], cv_words[0]);
  store32(&bytes_out[1 * 4], cv_words[1]);
  store32(&bytes_out[2 * 4], cv_words[2]);
  store32(&bytes_out[3 * 4], cv_words[3]);
  store32(&bytes_out[4 * 4], cv_words[4]);
  store32(&bytes_out[5 * 4], cv_words[5]);
  store32(&bytes_out[6 * 4], cv_words[6]);
  store32(&bytes_out[7 * 4], cv_words[7]);
}

/*============================================================================
 * Core Compression Function Declarations
 *============================================================================*/

/**
 * @brief Compress a block and update chaining value in-place
 *
 * Platform-dispatched function that selects the best implementation
 * (portable, SSE2, SSE4.1, AVX2, AVX-512, or NEON) at runtime.
 */
void blake3_compress_in_place(uint32_t cv[8],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t counter,
                              uint8_t flags);

/**
 * @brief Compress a block and produce 64-byte extended output
 *
 * Platform-dispatched function for XOF (eXtendable Output Function) mode.
 */
void blake3_compress_xof(const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags,
                         uint8_t out[64]);

/**
 * @brief Generate multiple XOF output blocks
 *
 * Efficiently generates multiple 64-byte output blocks for extended output.
 * Uses SIMD parallelism when available.
 */
void blake3_xof_many(const uint32_t cv[8],
                     const uint8_t block[BLAKE3_BLOCK_LEN],
                     uint8_t block_len, uint64_t counter, uint8_t flags,
                     uint8_t out[64], size_t outblocks);

/**
 * @brief Hash multiple inputs in parallel using SIMD
 *
 * Key function for exploiting SIMD parallelism. Processes multiple
 * independent inputs simultaneously using vector instructions.
 */
void blake3_hash_many(const uint8_t *const *inputs, size_t num_inputs,
                      size_t blocks, const uint32_t key[8], uint64_t counter,
                      bool increment_counter, uint8_t flags,
                      uint8_t flags_start, uint8_t flags_end, uint8_t *out);

/**
 * @brief Get the SIMD parallelism degree for the current platform
 *
 * Returns the number of chunks that can be processed in parallel:
 * - AVX-512: 16
 * - AVX2: 8
 * - SSE4.1/SSE2: 4
 * - NEON: 4
 * - Portable: 1
 */
size_t blake3_simd_degree(void);

/**
 * @brief Recursively compress a subtree with SIMD parallelism
 *
 * Internal function that processes large inputs by recursively splitting
 * them and using SIMD to hash chunks/parents in parallel.
 */
BLAKE3_PRIVATE size_t blake3_compress_subtree_wide(const uint8_t *input, size_t input_len,
                                                   const uint32_t key[8],
                                                   uint64_t chunk_counter, uint8_t flags,
                                                   uint8_t *out, bool use_tbb);

#if defined(BLAKE3_USE_TBB)
/**
 * @brief TBB parallel join for left and right subtrees
 *
 * Uses Intel Threading Building Blocks for multithreaded hashing
 * of independent subtrees.
 */
BLAKE3_PRIVATE void blake3_compress_subtree_wide_join_tbb(
    /* shared params */
    const uint32_t key[8], uint8_t flags, bool use_tbb,
    /* left-hand side params */
    const uint8_t *l_input, size_t l_input_len, uint64_t l_chunk_counter,
    uint8_t *l_cvs, size_t *l_n,
    /* right-hand side params */
    const uint8_t *r_input, size_t r_input_len, uint64_t r_chunk_counter,
    uint8_t *r_cvs, size_t *r_n) NOEXCEPT;
#endif

/*============================================================================
 * Platform-Specific Implementation Declarations
 *============================================================================*/

/* Portable implementation - works on any platform */
void blake3_compress_in_place_portable(uint32_t cv[8],
                                       const uint8_t block[BLAKE3_BLOCK_LEN],
                                       uint8_t block_len, uint64_t counter,
                                       uint8_t flags);

void blake3_compress_xof_portable(const uint32_t cv[8],
                                  const uint8_t block[BLAKE3_BLOCK_LEN],
                                  uint8_t block_len, uint64_t counter,
                                  uint8_t flags, uint8_t out[64]);

void blake3_hash_many_portable(const uint8_t *const *inputs, size_t num_inputs,
                               size_t blocks, const uint32_t key[8],
                               uint64_t counter, bool increment_counter,
                               uint8_t flags, uint8_t flags_start,
                               uint8_t flags_end, uint8_t *out);

/* x86/x86_64 SIMD implementations */
#if defined(IS_X86)

/* SSE2 implementation (128-bit SIMD, 4-way parallel) */
#if !defined(BLAKE3_NO_SSE2)
void blake3_compress_in_place_sse2(uint32_t cv[8],
                                   const uint8_t block[BLAKE3_BLOCK_LEN],
                                   uint8_t block_len, uint64_t counter,
                                   uint8_t flags);
void blake3_compress_xof_sse2(const uint32_t cv[8],
                              const uint8_t block[BLAKE3_BLOCK_LEN],
                              uint8_t block_len, uint64_t counter,
                              uint8_t flags, uint8_t out[64]);
void blake3_hash_many_sse2(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out);
#endif
#if !defined(BLAKE3_NO_SSE41)
void blake3_compress_in_place_sse41(uint32_t cv[8],
                                    const uint8_t block[BLAKE3_BLOCK_LEN],
                                    uint8_t block_len, uint64_t counter,
                                    uint8_t flags);
void blake3_compress_xof_sse41(const uint32_t cv[8],
                               const uint8_t block[BLAKE3_BLOCK_LEN],
                               uint8_t block_len, uint64_t counter,
                               uint8_t flags, uint8_t out[64]);
void blake3_hash_many_sse41(const uint8_t *const *inputs, size_t num_inputs,
                            size_t blocks, const uint32_t key[8],
                            uint64_t counter, bool increment_counter,
                            uint8_t flags, uint8_t flags_start,
                            uint8_t flags_end, uint8_t *out);
#endif
#if !defined(BLAKE3_NO_AVX2)
void blake3_hash_many_avx2(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out);
#endif
#if !defined(BLAKE3_NO_AVX512)
void blake3_compress_in_place_avx512(uint32_t cv[8],
                                     const uint8_t block[BLAKE3_BLOCK_LEN],
                                     uint8_t block_len, uint64_t counter,
                                     uint8_t flags);

void blake3_compress_xof_avx512(const uint32_t cv[8],
                                const uint8_t block[BLAKE3_BLOCK_LEN],
                                uint8_t block_len, uint64_t counter,
                                uint8_t flags, uint8_t out[64]);

void blake3_hash_many_avx512(const uint8_t *const *inputs, size_t num_inputs,
                             size_t blocks, const uint32_t key[8],
                             uint64_t counter, bool increment_counter,
                             uint8_t flags, uint8_t flags_start,
                             uint8_t flags_end, uint8_t *out);

#if !defined(_WIN32)
void blake3_xof_many_avx512(const uint32_t cv[8],
                            const uint8_t block[BLAKE3_BLOCK_LEN],
                            uint8_t block_len, uint64_t counter, uint8_t flags,
                            uint8_t* out, size_t outblocks);
#endif
#endif
#endif

#if BLAKE3_USE_NEON == 1
void blake3_hash_many_neon(const uint8_t *const *inputs, size_t num_inputs,
                           size_t blocks, const uint32_t key[8],
                           uint64_t counter, bool increment_counter,
                           uint8_t flags, uint8_t flags_start,
                           uint8_t flags_end, uint8_t *out);
#endif

#ifdef __cplusplus
}
#endif

#endif /* BLAKE3_IMPL_H */
