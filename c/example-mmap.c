#include <errno.h>
#include <stdio.h>
#include <stdlib.h>

#include "blake3.h"

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "USAGE: blake3-example-mmap <path>\n");
    exit(1);
  }

  // Initialize the hasher.
  blake3_hasher hasher;
  blake3_hasher_init(&hasher);

  // Update hash from file at path via mmap.
  errno = 0;
#ifdef BLAKE3_USE_TBB
  blake3_hasher_update_mmap_tbb(&hasher, argv[1]);
#else
  blake3_hasher_update_mmap(&hasher, argv[1]);
#endif
  // Check if any errors occurred because mmap may fail.
  if (errno != 0) {
    perror("ERROR");
    exit(1);
  }

  // Finalize the hash. BLAKE3_OUT_LEN is the default output length, 32 bytes.
  uint8_t output[BLAKE3_OUT_LEN];
  blake3_hasher_finalize(&hasher, output, BLAKE3_OUT_LEN);

  // Print the hash as hexadecimal.
  for (size_t i = 0; i < BLAKE3_OUT_LEN; i++) {
    printf("%02x", output[i]);
  }
  printf("\n");
  return 0;
}
