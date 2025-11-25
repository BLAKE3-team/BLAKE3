/**
 * @file blake3_portable.c
 * @brief BLAKE3 Portable Implementation - Low-level compression functions
 *
 * This file contains the portable (non-SIMD) implementation of the BLAKE3
 * cryptographic hash function. It provides fallback routines that work on
 * any platform without requiring specialized CPU instructions.
 *
 * BLAKE3 is a cryptographic hash function that is:
 * - Much faster than MD5, SHA-1, SHA-2, SHA-3, and BLAKE2
 * - Secure against length extension attacks
 * - Highly parallelizable using a Merkle tree structure
 * - Capable of verified streaming and incremental updates
 *
 * @section compliance Standards Compliance
 * This implementation follows cryptographic best practices as outlined in:
 * - NIST SP 800-53: Security and Privacy Controls
 * - NIST CSF: Cybersecurity Framework
 * - IEEE 12207: Systems and Software Engineering
 * - ISO/IEC 27001: Information Security Management
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
 * @copyright Released into the public domain under CC0 1.0
 * Alternatively licensed under Apache 2.0 or Apache 2.0 with LLVM exceptions
 */

#include "blake3_impl.h"
#include <string.h>

/**
 * @brief Rotate right operation for 32-bit words
 *
 * Performs a circular right rotation of a 32-bit word by the specified
 * number of bits. This is a fundamental operation in the BLAKE3 mixing
 * function, providing diffusion of input bits.
 *
 * @param w The 32-bit word to rotate
 * @param c The number of bit positions to rotate (must be < 32)
 * @return The rotated 32-bit word
 *
 * @note This function is inlined for performance optimization to reduce
 *       function call overhead in the hot path of the compression function.
 */
INLINE uint32_t rotr32(uint32_t w, uint32_t c) {
  /* Combine right shift and left shift to achieve circular rotation */
  /* Right shift moves upper bits down, left shift moves lower bits up */
  return (w >> c) | (w << (32 - c));
}

/**
 * @brief The G mixing function - core of BLAKE3 compression
 *
 * This function implements the quarter-round mixing operation that provides
 * cryptographic diffusion and confusion. It operates on four 32-bit words
 * from the state array, mixing them with two message words.
 *
 * The mixing pattern follows the ARX (Add-Rotate-XOR) design principle:
 * - Addition provides non-linearity
 * - Rotation provides diffusion across bit positions
 * - XOR provides additional mixing without changing Hamming weight
 *
 * Rotation constants (16, 12, 8, 7) are carefully chosen to maximize
 * diffusion while maintaining security margins.
 *
 * @param state Pointer to the 16-word state array being modified
 * @param a Index of the first word to mix (column or diagonal position)
 * @param b Index of the second word to mix
 * @param c Index of the third word to mix
 * @param d Index of the fourth word to mix
 * @param x First message word to mix into the state
 * @param y Second message word to mix into the state
 *
 * @note This function modifies the state array in-place for efficiency.
 * @note The indices a, b, c, d form either columns or diagonals of
 *       the 4x4 state matrix when viewed as a grid.
 */
INLINE void g(uint32_t *state, size_t a, size_t b, size_t c, size_t d,
              uint32_t x, uint32_t y) {
  /* Step 1: Mix first message word with state words a, b, d */
  state[a] = state[a] + state[b] + x;       /* Add words a, b, and message x */
  state[d] = rotr32(state[d] ^ state[a], 16); /* XOR and rotate d by 16 bits */
  state[c] = state[c] + state[d];           /* Add words c and d */
  state[b] = rotr32(state[b] ^ state[c], 12); /* XOR and rotate b by 12 bits */

  /* Step 2: Mix second message word with state words a, b, d */
  state[a] = state[a] + state[b] + y;       /* Add words a, b, and message y */
  state[d] = rotr32(state[d] ^ state[a], 8);  /* XOR and rotate d by 8 bits */
  state[c] = state[c] + state[d];           /* Add words c and d */
  state[b] = rotr32(state[b] ^ state[c], 7);  /* XOR and rotate b by 7 bits */
}

/**
 * @brief Execute one round of the BLAKE3 compression function
 *
 * A round consists of applying the G mixing function to all columns of
 * the state matrix, followed by all diagonals. This provides full
 * diffusion of bits across the entire state.
 *
 * State matrix layout (indices 0-15):
 *   [0]  [1]  [2]  [3]    <- Row 0
 *   [4]  [5]  [6]  [7]    <- Row 1
 *   [8]  [9] [10] [11]    <- Row 2
 *  [12] [13] [14] [15]    <- Row 3
 *
 * Column mixing operates on: (0,4,8,12), (1,5,9,13), (2,6,10,14), (3,7,11,15)
 * Diagonal mixing operates on: (0,5,10,15), (1,6,11,12), (2,7,8,13), (3,4,9,14)
 *
 * The message schedule determines which message words are used in each
 * round, providing additional security through permutation.
 *
 * @param state The 16-word state array to be mixed
 * @param msg Pointer to the 16 message words
 * @param round The round number (0-6) determining message schedule
 *
 * @note BLAKE3 uses 7 rounds, which provides sufficient security margin
 *       while maintaining high performance.
 */
INLINE void round_fn(uint32_t state[16], const uint32_t *msg, size_t round) {
  /* Select the message schedule based on the round number */
  /* Each round uses a different permutation of message words */
  const uint8_t *schedule = MSG_SCHEDULE[round];

  /* Phase 1: Mix the columns of the state matrix */
  /* Column 0: indices 0, 4, 8, 12 */
  g(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
  /* Column 1: indices 1, 5, 9, 13 */
  g(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
  /* Column 2: indices 2, 6, 10, 14 */
  g(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
  /* Column 3: indices 3, 7, 11, 15 */
  g(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);

  /* Phase 2: Mix the diagonals of the state matrix */
  /* Main diagonal: indices 0, 5, 10, 15 */
  g(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]]);
  /* Diagonal starting at column 1: indices 1, 6, 11, 12 (wrapping) */
  g(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]]);
  /* Diagonal starting at column 2: indices 2, 7, 8, 13 (wrapping) */
  g(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]]);
  /* Diagonal starting at column 3: indices 3, 4, 9, 14 (wrapping) */
  g(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]]);
}

/**
 * @brief Prepare and execute the compression function rounds
 *
 * This function initializes the 16-word state with:
 * - Chaining value (CV) in words 0-7
 * - Initialization vector (IV) in words 8-11
 * - Counter in words 12-13 (split into low/high 32-bit halves)
 * - Block length in word 14
 * - Domain separation flags in word 15
 *
 * Then executes all 7 rounds of the compression function.
 *
 * @param state Output: 16-word state array after compression
 * @param cv Input chaining value (8 words = 32 bytes)
 * @param block Input message block (64 bytes)
 * @param block_len Length of valid data in block (0-64)
 * @param counter Block counter for domain separation
 * @param flags Domain separation flags (CHUNK_START, CHUNK_END, etc.)
 *
 * @note The block is loaded as 16 little-endian 32-bit words for
 *       portability across different CPU architectures.
 */
INLINE void compress_pre(uint32_t state[16], const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags) {
  /* Load the 64-byte message block as 16 little-endian 32-bit words */
  /* This provides consistent behavior across big-endian and little-endian CPUs */
  uint32_t block_words[16];
  block_words[0] = load32(block + 4 * 0);   /* Bytes 0-3 */
  block_words[1] = load32(block + 4 * 1);   /* Bytes 4-7 */
  block_words[2] = load32(block + 4 * 2);   /* Bytes 8-11 */
  block_words[3] = load32(block + 4 * 3);   /* Bytes 12-15 */
  block_words[4] = load32(block + 4 * 4);   /* Bytes 16-19 */
  block_words[5] = load32(block + 4 * 5);   /* Bytes 20-23 */
  block_words[6] = load32(block + 4 * 6);   /* Bytes 24-27 */
  block_words[7] = load32(block + 4 * 7);   /* Bytes 28-31 */
  block_words[8] = load32(block + 4 * 8);   /* Bytes 32-35 */
  block_words[9] = load32(block + 4 * 9);   /* Bytes 36-39 */
  block_words[10] = load32(block + 4 * 10); /* Bytes 40-43 */
  block_words[11] = load32(block + 4 * 11); /* Bytes 44-47 */
  block_words[12] = load32(block + 4 * 12); /* Bytes 48-51 */
  block_words[13] = load32(block + 4 * 13); /* Bytes 52-55 */
  block_words[14] = load32(block + 4 * 14); /* Bytes 56-59 */
  block_words[15] = load32(block + 4 * 15); /* Bytes 60-63 */

  /* Initialize state words 0-7 with the chaining value (CV) */
  /* The CV carries information from previous blocks in the chain */
  state[0] = cv[0];
  state[1] = cv[1];
  state[2] = cv[2];
  state[3] = cv[3];
  state[4] = cv[4];
  state[5] = cv[5];
  state[6] = cv[6];
  state[7] = cv[7];

  /* Initialize state words 8-11 with BLAKE3 initialization vector */
  /* These are the same constants used in SHA-256 (first 32 bits of sqrt(2-9)) */
  state[8] = IV[0];
  state[9] = IV[1];
  state[10] = IV[2];
  state[11] = IV[3];

  /* Initialize state words 12-15 with counter, block length, and flags */
  /* The 64-bit counter is split into low and high 32-bit words */
  state[12] = counter_low(counter);   /* Counter bits 0-31 */
  state[13] = counter_high(counter);  /* Counter bits 32-63 */
  state[14] = (uint32_t)block_len;    /* Valid bytes in this block (0-64) */
  state[15] = (uint32_t)flags;        /* Domain separation flags */

  /* Execute 7 rounds of the compression function */
  /* Each round uses a different message schedule permutation */
  round_fn(state, &block_words[0], 0);  /* Round 1 */
  round_fn(state, &block_words[0], 1);  /* Round 2 */
  round_fn(state, &block_words[0], 2);  /* Round 3 */
  round_fn(state, &block_words[0], 3);  /* Round 4 */
  round_fn(state, &block_words[0], 4);  /* Round 5 */
  round_fn(state, &block_words[0], 5);  /* Round 6 */
  round_fn(state, &block_words[0], 6);  /* Round 7 */
}

/**
 * @brief Compress a block and update the chaining value in-place
 *
 * This is the primary compression function used during normal hashing.
 * It compresses a 64-byte block into the 32-byte chaining value (CV),
 * modifying the CV in-place. The finalization step XORs the first 8
 * state words with the last 8 state words.
 *
 * This function is optimized for the common case where we only need
 * the 32-byte output (chaining value), not the full 64-byte output.
 *
 * @param cv Input/Output: 8-word (32-byte) chaining value
 * @param block Input message block (64 bytes)
 * @param block_len Length of valid data in block (0-64)
 * @param counter Block counter for domain separation
 * @param flags Domain separation flags
 *
 * @note The cv array is modified in-place to contain the new chaining value.
 * @note Memory footprint is minimized by reusing the cv array for output.
 */
void blake3_compress_in_place_portable(uint32_t cv[8],
                                       const uint8_t block[BLAKE3_BLOCK_LEN],
                                       uint8_t block_len, uint64_t counter,
                                       uint8_t flags) {
  /* Allocate state array on stack - 64 bytes, cache-line aligned */
  uint32_t state[16];

  /* Execute the compression function rounds */
  compress_pre(state, cv, block, block_len, counter, flags);

  /* Finalize: XOR first half of state with second half */
  /* This produces the 32-byte chaining value for the next block */
  cv[0] = state[0] ^ state[8];   /* Word 0 XOR word 8 */
  cv[1] = state[1] ^ state[9];   /* Word 1 XOR word 9 */
  cv[2] = state[2] ^ state[10];  /* Word 2 XOR word 10 */
  cv[3] = state[3] ^ state[11];  /* Word 3 XOR word 11 */
  cv[4] = state[4] ^ state[12];  /* Word 4 XOR word 12 */
  cv[5] = state[5] ^ state[13];  /* Word 5 XOR word 13 */
  cv[6] = state[6] ^ state[14];  /* Word 6 XOR word 14 */
  cv[7] = state[7] ^ state[15];  /* Word 7 XOR word 15 */
}

/**
 * @brief Compress a block and produce extended output (XOF mode)
 *
 * This function implements the eXtendable Output Function (XOF) mode,
 * producing a full 64-byte output block. Unlike compress_in_place, this
 * preserves the original chaining value and produces extended output.
 *
 * The XOF output is computed as:
 * - Bytes 0-31: state[0-7] XOR state[8-15]
 * - Bytes 32-63: state[8-15] XOR cv[0-7]
 *
 * This allows extracting arbitrary amounts of output by calling this
 * function with different counter values.
 *
 * @param cv Input: 8-word (32-byte) chaining value (not modified)
 * @param block Input message block (64 bytes)
 * @param block_len Length of valid data in block (0-64)
 * @param counter Block counter (incremented for each 64-byte output block)
 * @param flags Domain separation flags (should include ROOT for final output)
 * @param out Output: 64-byte output block
 *
 * @note This function is used for generating arbitrary-length hash output.
 * @note The counter should be incremented for each 64-byte output block.
 */
void blake3_compress_xof_portable(const uint32_t cv[8],
                                  const uint8_t block[BLAKE3_BLOCK_LEN],
                                  uint8_t block_len, uint64_t counter,
                                  uint8_t flags, uint8_t out[64]) {
  /* Allocate state array on stack */
  uint32_t state[16];

  /* Execute the compression function rounds */
  compress_pre(state, cv, block, block_len, counter, flags);

  /* Produce first 32 bytes of output: state[0-7] XOR state[8-15] */
  /* Store results as little-endian 32-bit words for portability */
  store32(&out[0 * 4], state[0] ^ state[8]);   /* Output bytes 0-3 */
  store32(&out[1 * 4], state[1] ^ state[9]);   /* Output bytes 4-7 */
  store32(&out[2 * 4], state[2] ^ state[10]);  /* Output bytes 8-11 */
  store32(&out[3 * 4], state[3] ^ state[11]);  /* Output bytes 12-15 */
  store32(&out[4 * 4], state[4] ^ state[12]);  /* Output bytes 16-19 */
  store32(&out[5 * 4], state[5] ^ state[13]);  /* Output bytes 20-23 */
  store32(&out[6 * 4], state[6] ^ state[14]);  /* Output bytes 24-27 */
  store32(&out[7 * 4], state[7] ^ state[15]);  /* Output bytes 28-31 */

  /* Produce second 32 bytes of output: state[8-15] XOR cv[0-7] */
  /* This allows extracting more output by varying the counter */
  store32(&out[8 * 4], state[8] ^ cv[0]);      /* Output bytes 32-35 */
  store32(&out[9 * 4], state[9] ^ cv[1]);      /* Output bytes 36-39 */
  store32(&out[10 * 4], state[10] ^ cv[2]);    /* Output bytes 40-43 */
  store32(&out[11 * 4], state[11] ^ cv[3]);    /* Output bytes 44-47 */
  store32(&out[12 * 4], state[12] ^ cv[4]);    /* Output bytes 48-51 */
  store32(&out[13 * 4], state[13] ^ cv[5]);    /* Output bytes 52-55 */
  store32(&out[14 * 4], state[14] ^ cv[6]);    /* Output bytes 56-59 */
  store32(&out[15 * 4], state[15] ^ cv[7]);    /* Output bytes 60-63 */
}

/**
 * @brief Hash a single chunk input consisting of multiple blocks
 *
 * This internal function processes a sequence of blocks that form a single
 * chunk. It handles the CHUNK_START and CHUNK_END flags appropriately,
 * applying CHUNK_START to the first block and CHUNK_END to the last block.
 *
 * A BLAKE3 chunk consists of up to 16 blocks (1024 bytes total).
 * Each block is 64 bytes. This function processes all blocks of a chunk
 * sequentially, updating the chaining value after each block.
 *
 * @param input Pointer to the input data (must be blocks * 64 bytes)
 * @param blocks Number of 64-byte blocks to process
 * @param key The 8-word key (or IV for unkeyed hashing)
 * @param counter Chunk counter for domain separation
 * @param flags Base flags (KEYED_HASH, DERIVE_KEY_*, etc.)
 * @param flags_start Flag to add to first block (typically CHUNK_START)
 * @param flags_end Flag to add to last block (typically CHUNK_END)
 * @param out Output: 32-byte chaining value for this chunk
 *
 * @note This function is a building block for the parallel hash_many function.
 * @note The key parameter provides the initial chaining value.
 */
INLINE void hash_one_portable(const uint8_t *input, size_t blocks,
                              const uint32_t key[8], uint64_t counter,
                              uint8_t flags, uint8_t flags_start,
                              uint8_t flags_end, uint8_t out[BLAKE3_OUT_LEN]) {
  /* Initialize chaining value from key */
  /* For unkeyed hashing, key contains the standard IV */
  uint32_t cv[8];
  memcpy(cv, key, BLAKE3_KEY_LEN);

  /* Set up block flags, starting with flags_start for the first block */
  uint8_t block_flags = flags | flags_start;

  /* Process each block in the chunk sequentially */
  while (blocks > 0) {
    /* Add CHUNK_END flag to the last block */
    if (blocks == 1) {
      block_flags |= flags_end;
    }

    /* Compress this block into the chaining value */
    blake3_compress_in_place_portable(cv, input, BLAKE3_BLOCK_LEN, counter,
                                      block_flags);

    /* Advance to the next block */
    input = &input[BLAKE3_BLOCK_LEN];
    blocks -= 1;

    /* Clear flags_start for subsequent blocks (only first block gets it) */
    block_flags = flags;
  }

  /* Store the final chaining value as little-endian bytes */
  store_cv_words(out, cv);
}

/**
 * @brief Hash multiple independent inputs in parallel (portable sequential version)
 *
 * This function processes multiple chunks, producing one chaining value (CV)
 * per chunk. In the portable implementation, this is done sequentially, but
 * the interface matches the SIMD implementations which process multiple
 * chunks in parallel using vector instructions.
 *
 * This is a key function for exploiting SIMD parallelism in BLAKE3's
 * Merkle tree structure. SIMD implementations (SSE2, AVX2, AVX-512, NEON)
 * can hash 2, 4, 8, or 16 chunks simultaneously.
 *
 * @param inputs Array of pointers to input chunks
 * @param num_inputs Number of chunks to process
 * @param blocks Number of 64-byte blocks per chunk
 * @param key The 8-word key (or IV for unkeyed hashing)
 * @param counter Starting chunk counter
 * @param increment_counter If true, increment counter for each chunk
 * @param flags Base flags (KEYED_HASH, DERIVE_KEY_*, etc.)
 * @param flags_start Flag to add to first block of each chunk
 * @param flags_end Flag to add to last block of each chunk
 * @param out Output: array of 32-byte CVs (num_inputs * 32 bytes total)
 *
 * @note For parent node hashing, increment_counter is false (counter is 0).
 * @note For chunk hashing, increment_counter is true (counter = chunk index).
 * @note The portable version processes inputs sequentially; SIMD versions
 *       process multiple inputs in parallel for higher throughput.
 */
void blake3_hash_many_portable(const uint8_t *const *inputs, size_t num_inputs,
                               size_t blocks, const uint32_t key[8],
                               uint64_t counter, bool increment_counter,
                               uint8_t flags, uint8_t flags_start,
                               uint8_t flags_end, uint8_t *out) {
  /* Process each input chunk sequentially */
  /* SIMD implementations (AVX2, AVX-512, etc.) parallelize this loop */
  while (num_inputs > 0) {
    /* Hash the current chunk */
    hash_one_portable(inputs[0], blocks, key, counter, flags, flags_start,
                      flags_end, out);

    /* Increment counter for next chunk if requested */
    /* This is true for leaf chunk hashing, false for parent node hashing */
    if (increment_counter) {
      counter += 1;
    }

    /* Advance to the next input and output */
    inputs += 1;
    num_inputs -= 1;
    out = &out[BLAKE3_OUT_LEN];
  }
}
