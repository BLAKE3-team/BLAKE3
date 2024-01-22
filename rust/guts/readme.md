# The BLAKE3 Guts API

## Introduction

This crate contains low-level, high-performance, platform-specific
implementations of the BLAKE3 compression function. This API is complicated and
unsafe, and this crate will never have a stable release. For the standard
BLAKE3 hash function, see the [`blake3`](https://crates.io/crates/blake3)
crate, which depends on this one.

The most important ingredient in a high-performance implementation of BLAKE3 is
parallelism. The BLAKE3 tree structure lets us hash different parts of the tree
in parallel, and modern computers have a _lot_ of parallelism to offer.
Sometimes that means using multiple threads running on multiple cores, but
multithreading isn't appropriate for all applications, and it's not the usual
default for library APIs. More commonly, BLAKE3 implementations use SIMD
instructions ("Single Instruction Multiple Data") to improve the performance of
a single thread. When we do use multithreading, the performance benefits
multiply.

The tricky thing about SIMD is that each instruction set works differently.
Instead of writing portable code once and letting the compiler do most of the
optimization work, we need to write platform-specific implementations, and
sometimes more than one per platform. We maintain *four* different
implementations on x86 alone (targeting SSE2, SSE4.1, AVX2, and AVX-512), in
addition to ARM NEON and the RISC-V vector extensions. In the future we might
add ARM SVE2.

All of that means a lot of duplicated logic and maintenance. So while the main
goal of this API is high performance, it's also important to keep the API as
small and simple as possible. Higher level details like the "CV stack", input
buffering, and multithreading are handled by portable code in the main `blake3`
crate. These are just building blocks.

## The private API

This is the API that each platform reimplements. It's completely `unsafe`,
inputs and outputs are allowed to alias, and bounds checking is the caller's
responsibility.

- `degree`
- `compress`
- `hash_chunks`
- `hash_parents`
- `xof`
- `xof_xor`
- `universal_hash`

## The public API

This is the API that this crate exposes to callers, i.e. to the main `blake3`
crate. It's a thin, portable layer on top of the private API above. The Rust
version of this API is memory-safe.

- `degree`
- `compress`
- `hash_chunks`
- `hash_parents`
- `reduce_parents`
- `xof`
- `xof_xor`
- `universal_hash`
