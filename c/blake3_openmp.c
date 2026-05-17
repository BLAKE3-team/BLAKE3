#include "blake3_impl.h"

void blake3_compress_subtree_wide_join_openmp(
    // shared params
    const uint32_t key[8], uint8_t flags, uint8_t parallel,
    // left-hand side params
    const uint8_t *l_input, size_t l_input_len, uint64_t l_chunk_counter,
    uint8_t *l_cvs, size_t *l_n,
    // right-hand side params
    const uint8_t *r_input, size_t r_input_len, uint64_t r_chunk_counter,
    uint8_t *r_cvs, size_t *r_n) {
  if (parallel != USE_OPENMP) {
    *l_n = blake3_compress_subtree_wide(
        l_input, l_input_len, key, l_chunk_counter, flags, l_cvs, parallel);
    *r_n = blake3_compress_subtree_wide(
        r_input, r_input_len, key, r_chunk_counter, flags, r_cvs, parallel);
    return;
  }

#pragma omp parallel sections num_threads(2) firstprivate(                     \
        key, flags, parallel, l_input, l_input_len, l_chunk_counter, l_cvs,    \
            l_n, r_input, r_input_len, r_chunk_counter, r_cvs, r_n)
  {
#pragma omp section
    {
      *l_n = blake3_compress_subtree_wide(
          l_input, l_input_len, key, l_chunk_counter, flags, l_cvs, parallel);
    }
#pragma omp section
    {
      *r_n = blake3_compress_subtree_wide(
          r_input, r_input_len, key, r_chunk_counter, flags, r_cvs, parallel);
    }
  }
}
