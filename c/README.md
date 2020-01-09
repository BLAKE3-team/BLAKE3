# BLAKE3-c [![Actions Status](https://github.com/veorq/BLAKE3-c/workflows/tests/badge.svg)](https://github.com/veorq/BLAKE3-c/actions)

A very rough initial implementation of BLAKE3 in C. SSE4.1, AVX2,
AVX-512, and NEON are supported, using compile-time feature selection in
the Makefile.

This implementation is simpler than the [Rust
implementation](https://github.com/veorq/BLAKE3). It doesn't support
multithreading, and it doesn't parallelize parent hashes, so throughput
is lower.

TODO:
- CI testing for AVX-512 and NEON.
- Cross-platform build, e.g. Windows.
- Dynamic CPU feature detection, at least for x86.

Example usage:

```bash
$ make avx2
$ head -c 1000000 /dev/urandom | ./blake3
43f2cae3cfd7678bc3a3ebdbf170608d19d5ebaad23e9d06291dba3269853608
$ head -c 1000000 /dev/urandom | ./blake3 --length 50
4fc0ee74a60aa77fb699821997498fd93f1a98bd03eaf2a7969c4b35fb742c233a7a161fd2a431605f6e92dcf4cd7d052102
$ head -c 1000000 /dev/urandom | ./blake3 --keyed 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
8aee87b232fe90b042bf9119591e24409763a268139ff157d20021003e314064

```
