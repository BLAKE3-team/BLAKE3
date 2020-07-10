The official C implementation of BLAKE3.

# Example

An example program that hashes bytes from standard input and prints the
result:

```c
#include "blake3.h"
#include <stdio.h>
#include <unistd.h>

int main() {
  // Initialize the hasher.
  blake3_hasher hasher;
  blake3_hasher_init(&hasher);

  // Read input bytes from stdin.
  unsigned char buf[65536];
  ssize_t n;
  while ((n = read(STDIN_FILENO, buf, sizeof(buf))) > 0) {
    blake3_hasher_update(&hasher, buf, n);
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
```

If you save the example code above as `example.c`, and you're on x86\_64
with a Unix-like OS, you can compile a working binary like this:

```bash
gcc -O3 -o example example.c blake3.c blake3_dispatch.c blake3_portable.c \
    blake3_sse41_x86-64_unix.S blake3_avx2_x86-64_unix.S blake3_avx512_x86-64_unix.S
```

# API

## The Struct

```c
typedef struct {
  // private fields
} blake3_hasher;
```

An incremental BLAKE3 hashing state, which can accept any number of
updates. This implementation doesn't allocate any heap memory, but
`sizeof(blake3_hasher)` itself is relatively large, currently 1912 bytes
on x86-64. This size can be reduced by restricting the maximum input
length, as described in Section 5.4 of [the BLAKE3
spec](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf),
but this implementation doesn't currently support that strategy.

## Common API Functions

```c
void blake3_hasher_init(
  blake3_hasher *self);
```

Initialize a `blake3_hasher` in the default hashing mode.

```c
void blake3_hasher_update(
  blake3_hasher *self,
  const void *input,
  size_t input_len);
```

Add input to the hasher. This can be called any number of times.

```c
void blake3_hasher_finalize(
  const blake3_hasher *self,
  uint8_t *out,
  size_t out_len);
```

Finalize the hasher and emit an output of any length. This doesn't
modify the hasher itself, and it's possible to finalize again after
adding more input. The constant `BLAKE3_OUT_LEN` provides the default
output length, 32 bytes.

## Less Common API Functions

```c
void blake3_hasher_init_keyed(
  blake3_hasher *self,
  const uint8_t key[BLAKE3_KEY_LEN]);
```

Initialize a `blake3_hasher` in the keyed hashing mode. The key must be
exactly 32 bytes.

```c
void blake3_hasher_init_derive_key(
  blake3_hasher *self,
  const char *context);
```

Initialize a `blake3_hasher` in the key derivation mode. Key material
should be given as input after initialization, using
`blake3_hasher_update`. `context` is a standard C string of any length,
and the terminating null byte is not included. The context string should
be hardcoded, globally unique, and application-specific. A good default
format for the context string is `"[application] [commit timestamp]
[purpose]"`, e.g., `"example.com 2019-12-25 16:18:03 session tokens
v1"`.

```c
void blake3_hasher_finalize_seek(
  const blake3_hasher *self,
  uint64_t seek,
  uint8_t *out,
  size_t out_len);
```

The same as `blake3_hasher_finalize`, but with an additional `seek`
parameter for the starting byte position in the output stream. To
efficiently stream a large output without allocating memory, call this
function in a loop, incrementing `seek` by the output length each time.

# Building

This implementation is just C and assembly files. It doesn't include a
public-facing build system. (The `Makefile` in this directory is only
for testing.) Instead, the intention is that you can include these files
in whatever build system you're already using. This section describes
the commands your build system should execute, or which you can execute
by hand. Note that these steps may change in future versions.

## x86

Dynamic dispatch is enabled by default on x86. The implementation will
query the CPU at runtime to detect SIMD support, and it will use the
widest instruction set available. By default, `blake3_dispatch.c`
expects to be linked with code for four different instruction sets:
portable C, SSE4.1, AVX2, and AVX-512.

For each of the x86 SIMD instruction sets, two versions are available,
one in assembly (with three flavors: Unix, Windows MSVC, and Windows
GNU) and one using C intrinsics. The assembly versions are generally
preferred: they perform better, they perform more consistently across
different compilers, and they build more quickly. On the other hand, the
assembly versions are x86\_64-only, and you need to select the right
flavor for your target platform.

Here's an example of building a shared library on x86\_64 Linux using
the assembly implementations:

```bash
gcc -shared -O3 -o libblake3.so blake3.c blake3_dispatch.c blake3_portable.c \
    blake3_sse41_x86-64_unix.S blake3_avx2_x86-64_unix.S blake3_avx512_x86-64_unix.S
```

Here's the same shared library using the intrinsics-based implementations:

```bash
gcc -shared -O3 -o libblake3.so blake3.c blake3_dispatch.c blake3_portable.c \
    blake3_avx2.c blake3_avx512.c blake3_sse41.c
```

When building the intrinsics-based implementations under MSVC, you need to
build `blake3_avx2.c` and `blake3_avx512.c` separately first, specifying the
`/arch:AVX2` and `/arch:AVX512` compiler flags respectively.

If you want to omit SIMD code on x86, you need to explicitly disable
each instruction set. Here's an example of building a shared library on
x86 with only portable code:

```bash
gcc -shared -O3 -o libblake3.so -DBLAKE3_NO_SSE41 -DBLAKE3_NO_AVX2 -DBLAKE3_NO_AVX512 \
    blake3.c blake3_dispatch.c blake3_portable.c
```

## ARM NEON

The NEON implementation is not enabled by default on ARM, since not all
ARM targets support it. To enable it, set `BLAKE3_USE_NEON=1`. Here's an
example of building a shared library on ARM Linux with NEON support:

```bash
gcc -shared -O3 -o libblake3.so -DBLAKE3_USE_NEON blake3.c blake3_dispatch.c \
    blake3_portable.c blake3_neon.c
```

Note that on some targets (ARMv7 in particular), extra flags may be
required to activate NEON support in the compiler. If you see an error
like...

```
/usr/lib/gcc/armv7l-unknown-linux-gnueabihf/9.2.0/include/arm_neon.h:635:1: error: inlining failed
in call to always_inline ‘vaddq_u32’: target specific option mismatch
```

...then you may need to add something like `-mfpu=neon-vfpv4
-mfloat-abi=hard`.

## Other Platforms

The portable implementation should work on most other architectures. For
example:

```bash
gcc -shared -O3 -o libblake3.so blake3.c blake3_dispatch.c blake3_portable.c
```

# Differences from the Rust Implementation

The single-threaded Rust and C implementations use the same algorithms,
and their performance is the same if you use the assembly
implementations or if you compile the intrinsics-based implementations
with Clang. (Both Clang and rustc are LLVM-based.)

The C implementation doesn't currently support multi-threading. OpenMP
support or similar might be added in the future.
