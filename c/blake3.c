// NB: This is only for benchmarking. The guy who wrote this file hasn't
// touched C since college. Please don't use this code in production.

#include <assert.h>
#include <stdbool.h>
#include <string.h>

#include "blake3.h"
#include "blake3_impl.h"

INLINE void chunk_state_init(blake3_chunk_state *self, const uint32_t key[8],
                             uint8_t flags) {
  memcpy(self->cv, key, BLAKE3_KEY_LEN);
  self->chunk_counter = 0;
  memset(self->buf, 0, BLAKE3_BLOCK_LEN);
  self->buf_len = 0;
  self->blocks_compressed = 0;
  self->flags = flags;
}

INLINE void chunk_state_reset(blake3_chunk_state *self, const uint32_t key[8],
                              uint64_t chunk_counter) {
  memcpy(self->cv, key, BLAKE3_KEY_LEN);
  self->chunk_counter = chunk_counter;
  self->blocks_compressed = 0;
  memset(self->buf, 0, BLAKE3_BLOCK_LEN);
  self->buf_len = 0;
}

INLINE size_t chunk_state_len(const blake3_chunk_state *self) {
  return (BLAKE3_BLOCK_LEN * (size_t)self->blocks_compressed) +
         ((size_t)self->buf_len);
}

INLINE size_t chunk_state_fill_buf(blake3_chunk_state *self,
                                   const uint8_t *input, size_t input_len) {
  size_t take = BLAKE3_BLOCK_LEN - ((size_t)self->buf_len);
  if (take > input_len) {
    take = input_len;
  }
  uint8_t *dest = self->buf + ((size_t)self->buf_len);
  memcpy(dest, input, take);
  self->buf_len += (uint8_t)take;
  return take;
}

INLINE uint8_t chunk_state_maybe_start_flag(const blake3_chunk_state *self) {
  if (self->blocks_compressed == 0) {
    return CHUNK_START;
  } else {
    return 0;
  }
}

typedef struct {
  uint32_t input_cv[8];
  uint64_t counter;
  uint8_t block[BLAKE3_BLOCK_LEN];
  uint8_t block_len;
  uint8_t flags;
} output_t;

INLINE output_t make_output(const uint32_t input_cv[8],
                            const uint8_t block[BLAKE3_BLOCK_LEN],
                            uint8_t block_len, uint64_t counter,
                            uint8_t flags) {
  output_t ret;
  memcpy(ret.input_cv, input_cv, 32);
  memcpy(ret.block, block, BLAKE3_BLOCK_LEN);
  ret.block_len = block_len;
  ret.counter = counter;
  ret.flags = flags;
  return ret;
}

// Chaining values within a given chunk (specifically the compress_in_place
// interface) are represented as words. This avoids unnecessary bytes<->words
// conversion overhead in the portable implementation. However, the hash_many
// interface handles both user input and parent node blocks, so it accepts
// bytes. For that reason, chaining values in the CV stack are represented as
// bytes.
INLINE void output_chaining_value(const output_t *self, uint8_t cv[32]) {
  uint32_t cv_words[8];
  memcpy(cv_words, self->input_cv, 32);
  blake3_compress_in_place(cv_words, self->block, self->block_len, self->counter,
                    self->flags);
  memcpy(cv, cv_words, 32);
}

INLINE void output_root_bytes(const output_t *self, uint8_t *out,
                              size_t out_len) {
  uint64_t output_block_counter = 0;
  uint8_t wide_buf[64];
  while (out_len > 0) {
    blake3_compress_xof(self->input_cv, self->block, self->block_len,
                 output_block_counter, self->flags | ROOT, wide_buf);
    size_t memcpy_len;
    if (out_len > 64) {
      memcpy_len = 64;
    } else {
      memcpy_len = out_len;
    }
    memcpy(out, wide_buf, memcpy_len);
    out += memcpy_len;
    out_len -= memcpy_len;
    output_block_counter += 1;
  }
}

INLINE void chunk_state_update(blake3_chunk_state *self, const uint8_t *input,
                               size_t input_len) {
  if (self->buf_len > 0) {
    size_t take = chunk_state_fill_buf(self, input, input_len);
    input += take;
    input_len -= take;
    if (input_len > 0) {
      blake3_compress_in_place(self->cv, self->buf, BLAKE3_BLOCK_LEN,
                        self->chunk_counter,
                        self->flags | chunk_state_maybe_start_flag(self));
      self->blocks_compressed += 1;
      self->buf_len = 0;
      memset(self->buf, 0, BLAKE3_BLOCK_LEN);
    }
  }

  while (input_len > BLAKE3_BLOCK_LEN) {
    blake3_compress_in_place(self->cv, input, BLAKE3_BLOCK_LEN, self->chunk_counter,
                      self->flags | chunk_state_maybe_start_flag(self));
    self->blocks_compressed += 1;
    input += BLAKE3_BLOCK_LEN;
    input_len -= BLAKE3_BLOCK_LEN;
  }

  size_t take = chunk_state_fill_buf(self, input, input_len);
  input += take;
  input_len -= take;
}

INLINE output_t chunk_state_output(const blake3_chunk_state *self) {
  uint8_t block_flags =
      self->flags | chunk_state_maybe_start_flag(self) | CHUNK_END;
  return make_output(self->cv, self->buf, self->buf_len, self->chunk_counter,
                     block_flags);
}

INLINE output_t parent_output(const uint8_t block[BLAKE3_BLOCK_LEN],
                              const uint32_t key[8], uint8_t flags) {
  return make_output(key, block, BLAKE3_BLOCK_LEN, 0, flags | PARENT);
}

INLINE void hasher_init_base(blake3_hasher *self, const uint32_t key[8],
                             uint8_t flags) {
  memcpy(self->key, key, BLAKE3_KEY_LEN);
  chunk_state_init(&self->chunk, key, flags);
  self->cv_stack_len = 0;
}

void blake3_hasher_init(blake3_hasher *self) { hasher_init_base(self, IV, 0); }

void blake3_hasher_init_keyed(blake3_hasher *self,
                              const uint8_t key[BLAKE3_KEY_LEN]) {
  uint32_t key_words[8];
  load_key_words(key, key_words);
  hasher_init_base(self, key_words, KEYED_HASH);
}

void blake3_hasher_init_derive_key(blake3_hasher *self, const char *context) {
  blake3_hasher context_hasher;
  hasher_init_base(&context_hasher, IV, DERIVE_KEY_CONTEXT);
  blake3_hasher_update(&context_hasher, context, strlen(context));
  uint8_t context_key[BLAKE3_KEY_LEN];
  blake3_hasher_finalize(&context_hasher, context_key, BLAKE3_KEY_LEN);
  uint32_t context_key_words[8];
  load_key_words(context_key, context_key_words);
  hasher_init_base(self, context_key_words, DERIVE_KEY_MATERIAL);
}

INLINE bool hasher_needs_merge(const blake3_hasher *self,
                               uint64_t total_chunks) {
  return self->cv_stack_len > popcnt(total_chunks);
}

INLINE void hasher_merge_parent(blake3_hasher *self) {
  size_t parent_block_start =
      (((size_t)self->cv_stack_len) - 2) * BLAKE3_OUT_LEN;
  output_t output = parent_output(&self->cv_stack[parent_block_start],
                                  self->key, self->chunk.flags);
  output_chaining_value(&output, &self->cv_stack[parent_block_start]);
  self->cv_stack_len -= 1;
}

INLINE void hasher_push_chunk_cv(blake3_hasher *self,
                                 uint8_t cv[BLAKE3_OUT_LEN],
                                 uint64_t chunk_counter) {
  assert(self->cv_stack_len < BLAKE3_MAX_DEPTH);
  while (hasher_needs_merge(self, chunk_counter)) {
    hasher_merge_parent(self);
  }
  memcpy(&self->cv_stack[self->cv_stack_len * BLAKE3_OUT_LEN], cv,
         BLAKE3_OUT_LEN);
  self->cv_stack_len += 1;
}

void blake3_hasher_update(blake3_hasher *self, const void *input,
                          size_t input_len) {
  const uint8_t *input_bytes = (const uint8_t *)input;

  // If we already have a partial chunk, or if this is the very first chunk
  // (and it could be the root), we need to add bytes to the chunk state.
  bool is_first_chunk = self->chunk.chunk_counter == 0;
  bool maybe_root = is_first_chunk && input_len == BLAKE3_CHUNK_LEN;
  if (maybe_root || chunk_state_len(&self->chunk) > 0) {
    size_t take = BLAKE3_CHUNK_LEN - chunk_state_len(&self->chunk);
    if (take > input_len) {
      take = input_len;
    }
    chunk_state_update(&self->chunk, input_bytes, take);
    input_bytes += take;
    input_len -= take;
    // If we've filled the current chunk and there's more coming, finalize this
    // chunk and proceed. In this case we know it's not the root.
    if (input_len > 0) {
      output_t output = chunk_state_output(&self->chunk);
      uint8_t chunk_cv[32];
      output_chaining_value(&output, chunk_cv);
      hasher_push_chunk_cv(self, chunk_cv, self->chunk.chunk_counter);
      chunk_state_reset(&self->chunk, self->key, self->chunk.chunk_counter + 1);
    } else {
      return;
    }
  }

  // Hash as many whole chunks as we can, without buffering anything. At this
  // point we know none of them can be the root.
  uint8_t out[BLAKE3_OUT_LEN * BLAKE3_MAX_SIMD_DEGREE];
  const uint8_t *chunks[BLAKE3_MAX_SIMD_DEGREE];
  size_t num_chunks = 0;
  while (input_len >= BLAKE3_CHUNK_LEN) {
    while (input_len >= BLAKE3_CHUNK_LEN &&
           num_chunks < BLAKE3_MAX_SIMD_DEGREE) {
      chunks[num_chunks] = input_bytes;
      input_bytes += BLAKE3_CHUNK_LEN;
      input_len -= BLAKE3_CHUNK_LEN;
      num_chunks += 1;
    }
    blake3_hash_many(chunks, num_chunks, BLAKE3_CHUNK_LEN / BLAKE3_BLOCK_LEN,
              self->key, self->chunk.chunk_counter, true, self->chunk.flags,
              CHUNK_START, CHUNK_END, out);
    for (size_t chunk_index = 0; chunk_index < num_chunks; chunk_index++) {
      // The chunk state is empty here, but it stores the counter of the next
      // chunk hash we need to push. Use that counter, and then move it forward.
      hasher_push_chunk_cv(self, &out[chunk_index * BLAKE3_OUT_LEN],
                           self->chunk.chunk_counter);
      self->chunk.chunk_counter += 1;
    }
    num_chunks = 0;
  }

  // If there's any remaining input less than a full chunk, add it to the chunk
  // state. In that case, also do a final merge loop to make sure the subtree
  // stack doesn't contain any unmerged pairs. The remaining input means we
  // know these merges are non-root. This merge loop isn't strictly necessary
  // here, because hasher_push_chunk_cv already does its own merge loop, but it
  // simplifies blake3_hasher_finalize below.
  if (input_len > 0) {
    while (hasher_needs_merge(self, self->chunk.chunk_counter)) {
      hasher_merge_parent(self);
    }
    chunk_state_update(&self->chunk, input_bytes, input_len);
  }
}

void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out,
                            size_t out_len) {
  // If the subtree stack is empty, then the current chunk is the root.
  if (self->cv_stack_len == 0) {
    output_t output = chunk_state_output(&self->chunk);
    output_root_bytes(&output, out, out_len);
    return;
  }
  // If there are any bytes in the chunk state, finalize that chunk and do a
  // roll-up merge between that chunk hash and every subtree in the stack. In
  // this case, the extra merge loop at the end of blake3_hasher_update
  // guarantees that none of the subtrees in the stack need to be merged with
  // each other first. Otherwise, if there are no bytes in the chunk state,
  // then the top of the stack is a chunk hash, and we start the merge from
  // that.
  output_t output;
  size_t cvs_remaining;
  if (chunk_state_len(&self->chunk) > 0) {
    cvs_remaining = self->cv_stack_len;
    output = chunk_state_output(&self->chunk);
  } else {
    // There are always at least 2 CVs in the stack in this case.
    cvs_remaining = self->cv_stack_len - 2;
    output = parent_output(&self->cv_stack[cvs_remaining * 32], self->key,
                           self->chunk.flags);
  }
  while (cvs_remaining > 0) {
    cvs_remaining -= 1;
    uint8_t parent_block[BLAKE3_BLOCK_LEN];
    memcpy(parent_block, &self->cv_stack[cvs_remaining * 32], 32);
    output_chaining_value(&output, &parent_block[32]);
    output = parent_output(parent_block, self->key, self->chunk.flags);
  }
  output_root_bytes(&output, out, out_len);
}
