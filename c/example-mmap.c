// vim: ts=2 sw=2 et
#include "blake3.h"
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>
#include <fcntl.h>
#include <alloca.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mman.h>

/* 256K threshold for using alloca */
#define BLAKE3_ALLOCA_BUFFER_SIZE  (256U << 10)

int blake3_hash_file(const char *filename, uint8_t output[BLAKE3_OUT_LEN])
{
  blake3_hasher hasher;
  FILE *fp = NULL;
  void *mem = NULL;
  void *buf = NULL;
  int fd = -1, ret = -1;
  size_t rdn, filesize, bufsz = BLAKE3_ALLOCA_BUFFER_SIZE;
  struct stat sb;
  int rc;

  memset(output, 0, BLAKE3_OUT_LEN);

  blake3_hasher_init(&hasher);

  if (!strcmp(filename, "-")) {
    fp = stdin;
  } else {
    fp = NULL;

    fd = open(filename, O_RDONLY);
    if (fd < 0)
      goto err_out;

    rc = fstat(fd, &sb);
    if (rc < 0)
      goto err_out;

    filesize = (size_t)-1;

    /* try to mmap */
    if (sb.st_size > 0) {
      filesize = sb.st_size;
      mem = mmap(NULL, sb.st_size, PROT_READ, MAP_PRIVATE, fd, 0);
      if (mem != MAP_FAILED) {
        close(fd);
        fd = -1;
      } else
        mem = NULL;
    }

    /* unable to map? fallback to stream mode */
    if (!mem) {
      close(fd);
      fd = -1;
      fp = fopen(filename, "r");
      if (!fp)
        goto err_out;
    }
  }

  /* mmap case, very simple */
  if (mem) {
    blake3_hasher_update(&hasher, mem, filesize);
  } else {
    /* slow path using file reads */

    assert(fp);
    if (bufsz <= BLAKE3_ALLOCA_BUFFER_SIZE) {
      buf = alloca(bufsz);
    } else {
      buf = malloc(bufsz);
      if (!buf)
        goto err_out;
    }

    do {
      rdn = fread(buf, 1, bufsz, fp);
      if (rdn == 0)
        break;
      blake3_hasher_update(&hasher, buf, rdn);
    } while (rdn >= bufsz);
  }

  // Finalize the hash. BLAKE3_OUT_LEN is the default output length, 32 bytes.
  blake3_hasher_finalize(&hasher, output, BLAKE3_OUT_LEN);

  ret = 0;

out:
  if (mem)
    munmap(mem, filesize);

  if (fp && fp != stdin)
    fclose(fp);

  if (fd >= 0)
    close(fd);

  if (buf && (bufsz > BLAKE3_ALLOCA_BUFFER_SIZE))
    free(buf);

  return ret;

err_out:
  ret = -1;
  goto out;
}

int main(int argc, char *argv[])
{
  uint8_t output[BLAKE3_OUT_LEN];
  int i, ok, rc;

  ok = 1;
  for (i = 1; i < argc; i++) {
    rc = blake3_hash_file(argv[i], output);
    if (rc) {
      fprintf(stderr, "Error hashing file \"%s\": %s\n", argv[i], strerror(errno));
      continue;
    }

    // Print the hash as hexadecimal.
    for (size_t j = 0; j < BLAKE3_OUT_LEN; j++)
      printf("%02x", output[j]);
    printf("  %s\n", argv[i]);
    ok++;
  }
  return ok == argc ? EXIT_SUCCESS : EXIT_FAILURE;
}
