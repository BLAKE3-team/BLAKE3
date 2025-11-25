/**
 * @file blake3.h
 * @brief BLAKE3 Cryptographic Hash Function - Public API
 *
 * BLAKE3 is a cryptographic hash function that is:
 * - Much faster than MD5, SHA-1, SHA-2, SHA-3, and BLAKE2
 * - Secure against length extension attacks
 * - Highly parallelizable using a Merkle tree structure
 * - Capable of verified streaming and incremental updates
 * - A PRF, MAC, KDF, and XOF as well as a regular hash
 *
 * @section usage Basic Usage
 * @code{.c}
 * // Hash some data
 * blake3_hasher hasher;
 * blake3_hasher_init(&hasher);
 * blake3_hasher_update(&hasher, data, data_len);
 * uint8_t hash[BLAKE3_OUT_LEN];
 * blake3_hasher_finalize(&hasher, hash, BLAKE3_OUT_LEN);
 * @endcode
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
 * - NIST SP 800-53: Security and Privacy Controls
 * - NIST CSF: Cybersecurity Framework
 * - ISO/IEC 27001: Information Security Management
 * - IEEE 12207: Systems and Software Engineering
 *
 * @copyright Released into the public domain under CC0 1.0
 * Alternatively licensed under Apache 2.0 or Apache 2.0 with LLVM exceptions
 */

#ifndef BLAKE3_H
#define BLAKE3_H

#include <stddef.h>
#include <stdint.h>

/**
 * @name API Visibility Macros
 * @brief Control symbol visibility for shared library builds
 * @{
 */
#if !defined(BLAKE3_API)
# if defined(_WIN32) || defined(__CYGWIN__)
#   if defined(BLAKE3_DLL)
#     if defined(BLAKE3_DLL_EXPORTS)
#       define BLAKE3_API __declspec(dllexport)
#     else
#       define BLAKE3_API __declspec(dllimport)
#     endif
#     define BLAKE3_PRIVATE
#   else
#     define BLAKE3_API
#     define BLAKE3_PRIVATE
#   endif
# elif __GNUC__ >= 4
#   define BLAKE3_API __attribute__((visibility("default")))
#   define BLAKE3_PRIVATE __attribute__((visibility("hidden")))
# else
#   define BLAKE3_API
#   define BLAKE3_PRIVATE
# endif
#endif
/** @} */

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @name BLAKE3 Constants
 * @brief Public constants for the BLAKE3 hash function
 * @{
 */

/** @brief Version string for this BLAKE3 implementation */
#define BLAKE3_VERSION_STRING "1.8.2"

/** @brief Length of a BLAKE3 key in bytes (256 bits) */
#define BLAKE3_KEY_LEN 32

/** @brief Default output length in bytes (256 bits) */
#define BLAKE3_OUT_LEN 32

/** @brief Internal block length in bytes */
#define BLAKE3_BLOCK_LEN 64

/** @brief Chunk length in bytes (1 KiB) */
#define BLAKE3_CHUNK_LEN 1024

/** @brief Maximum tree depth (supports up to 2^64 bytes of input) */
#define BLAKE3_MAX_DEPTH 54

/** @} */

/**
 * @brief Internal chunk state structure
 *
 * This struct tracks the state while processing a single chunk (1024 bytes).
 * It is a private implementation detail exposed here only because it is
 * needed for the blake3_hasher structure.
 *
 * @note Do not access these fields directly.
 */
typedef struct {
  uint32_t cv[8];              /**< Current chaining value (8 words) */
  uint64_t chunk_counter;      /**< Index of this chunk in the stream */
  uint8_t buf[BLAKE3_BLOCK_LEN]; /**< Buffer for partial block data */
  uint8_t buf_len;             /**< Number of bytes in buf */
  uint8_t blocks_compressed;   /**< Number of complete blocks processed */
  uint8_t flags;               /**< Domain separation flags */
} blake3_chunk_state;

/**
 * @brief BLAKE3 incremental hasher state
 *
 * This structure holds the state for incremental hashing. Initialize it
 * with blake3_hasher_init(), blake3_hasher_init_keyed(), or
 * blake3_hasher_init_derive_key() before use.
 *
 * @note The hasher can be reused after calling blake3_hasher_reset().
 * @note For thread safety, each thread should have its own hasher instance.
 */
typedef struct {
  uint32_t key[8];             /**< Key or IV for this hasher instance */
  blake3_chunk_state chunk;    /**< Current chunk being processed */
  uint8_t cv_stack_len;        /**< Number of CVs on the stack */
  /**
   * Stack of chaining values for completed subtrees.
   * Size is MAX_DEPTH + 1 because of lazy merging - we delay merging
   * until we know whether more input is coming.
   */
  uint8_t cv_stack[(BLAKE3_MAX_DEPTH + 1) * BLAKE3_OUT_LEN];
} blake3_hasher;

/*============================================================================
 * Public API Functions
 *============================================================================*/

/**
 * @brief Get the BLAKE3 library version string
 * @return Pointer to static version string (e.g., "1.8.2")
 */
BLAKE3_API const char *blake3_version(void);

/**
 * @brief Initialize a hasher for regular (unkeyed) hashing
 *
 * Sets up the hasher to compute a standard BLAKE3 hash.
 *
 * @param self Pointer to the hasher to initialize
 */
BLAKE3_API void blake3_hasher_init(blake3_hasher *self);

/**
 * @brief Initialize a hasher for keyed hashing (MAC)
 *
 * Sets up the hasher to compute a keyed hash, suitable for use
 * as a message authentication code (MAC).
 *
 * @param self Pointer to the hasher to initialize
 * @param key 32-byte secret key
 *
 * @note For MAC usage, verify hashes using constant-time comparison.
 */
BLAKE3_API void blake3_hasher_init_keyed(blake3_hasher *self,
                                         const uint8_t key[BLAKE3_KEY_LEN]);

/**
 * @brief Initialize a hasher for key derivation (KDF)
 *
 * Sets up the hasher to derive keys from input key material.
 * The context string provides domain separation.
 *
 * @param self Pointer to the hasher to initialize
 * @param context Application-specific context string (null-terminated)
 *
 * @note Context should be hardcoded, globally unique, and application-specific.
 * @note Do NOT use BLAKE3 for password hashing - use Argon2 instead.
 */
BLAKE3_API void blake3_hasher_init_derive_key(blake3_hasher *self, const char *context);

/**
 * @brief Initialize a hasher for key derivation with raw context
 *
 * Like blake3_hasher_init_derive_key(), but accepts binary context data.
 *
 * @param self Pointer to the hasher to initialize
 * @param context Pointer to context data
 * @param context_len Length of context data in bytes
 */
BLAKE3_API void blake3_hasher_init_derive_key_raw(blake3_hasher *self, const void *context,
                                                  size_t context_len);

/**
 * @brief Add input data to the hash state
 *
 * Process additional input bytes. Can be called multiple times.
 *
 * @param self Pointer to the hasher
 * @param input Pointer to input data (may be NULL if input_len is 0)
 * @param input_len Number of bytes to process
 *
 * @note Thread-safe if each thread uses its own hasher instance.
 */
BLAKE3_API void blake3_hasher_update(blake3_hasher *self, const void *input,
                                     size_t input_len);

#if defined(BLAKE3_USE_TBB)
/**
 * @brief Add input data using multithreaded processing
 *
 * Like blake3_hasher_update(), but uses Intel TBB for parallelism.
 * Best for large inputs (> 128 KiB).
 *
 * @param self Pointer to the hasher
 * @param input Pointer to input data
 * @param input_len Number of bytes to process
 */
BLAKE3_API void blake3_hasher_update_tbb(blake3_hasher *self, const void *input,
                                         size_t input_len);
#endif // BLAKE3_USE_TBB

/**
 * @brief Finalize the hash and produce output
 *
 * Computes the final hash value. Can be called multiple times without
 * affecting the hasher state (idempotent). Supports extended output
 * (XOF mode) by specifying out_len > 32.
 *
 * @param self Pointer to the hasher
 * @param out Pointer to output buffer
 * @param out_len Number of output bytes to produce (any length)
 *
 * @note Shorter outputs are prefixes of longer outputs.
 * @note For security, 32 bytes (256 bits) is recommended.
 */
BLAKE3_API void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out,
                                       size_t out_len);

/**
 * @brief Finalize with output seeking (XOF mode)
 *
 * Like blake3_hasher_finalize(), but allows starting output at an offset.
 * Useful for extracting different portions of extended output.
 *
 * @param self Pointer to the hasher
 * @param seek Starting byte offset in the output stream
 * @param out Pointer to output buffer
 * @param out_len Number of output bytes to produce
 */
BLAKE3_API void blake3_hasher_finalize_seek(const blake3_hasher *self, uint64_t seek,
                                            uint8_t *out, size_t out_len);

/**
 * @brief Reset hasher to initial state
 *
 * Resets the hasher to its initial state, preserving the key/context
 * if using keyed or derive-key mode. Allows reuse without reallocation.
 *
 * @param self Pointer to the hasher to reset
 */
BLAKE3_API void blake3_hasher_reset(blake3_hasher *self);

#ifdef __cplusplus
}
#endif

#endif /* BLAKE3_H */
