# BLAKE3

BLAKE3 is a cryptographic hash function that is:

1. Faster than MD5, SHA1, SHA2, SHA3, and even BLAKE2.
1. Highly parallelizable: The more data and the more cores, the faster it
goes. (For you specialists reading this: this is because it is actually a
Merkle tree under the hood.)
1. Capable of verified streaming and incremental updates. (Again: the magic of Merkle trees.)
1. Carefully engineered to be simple and safe to use, with no "flavors" or variants.

<p align="center">
<img src="media/speed.svg" alt="performance graph">
</p>

The complete specifications and design rationale are available as a
[PDF](https://github.com/BLAKE3-team/BLAKE3-specs/raw/master/blake3.pdf) and its
[LaTeX source](https://github.com/BLAKE3-team/BLAKE3-specs/).

This repository provides the official Rust implementation of BLAKE3, the
[`blake3`](https://crates.io/crates/blake3) crate. It includes optimized
SIMD implementations, using dynamic CPU feature detection on x86. SSE4.1
and AVX2 support are implemented in Rust, while AVX-512 and ARM NEON
support are implemented in C and controlled by the `c_avx512` and
`c_neon` features. Multi-threading is implemented with
[Rayon](https://github.com/rayon-rs/rayon) and controlled by the `rayon`
feature. This repository also hosts the simplified [reference
implementation](reference_impl/reference_impl.rs), which is portable and
`no_std`-compatible.

The [`b3sum` sub-crate](./b3sum) provides a command line interface. You
can install it with `cargo install b3sum`. It includes multi-threading
and AVX-512 support by default.

BLAKE3 was designed by:

* [@oconnor63 ](https://github.com/oconnor63) (Jack O'Connor)
* [@sneves](https://github.com/sneves) (Samuel Neves)
* [@veorq](https://github.com/veorq) (Jean-Philippe Aumasson)
* [@zookozcash](https://github.com/zookozcash) (Zooko)

*WARNING*: BLAKE3 is not a password hash, because it's designed to be
fast, whereas password hashing should not be fast. If you hash passwords
to store the hashes or if you derive keys from passwords, we recommend
[Argon2](https://github.com/P-H-C/phc-winner-argon2).

## Usage

TODO

## History

BLAKE3 is essentially an adapted version of [BLAKE2](https://blake2.net)
using the [Bao](https://github.com/oconnor663/baokeshed) tree mode.

BLAKE2 is an established cryptographic hash function, for example
supported by OpenSSL, and used in countless applications.
Bao is a tree hashing mode satisfying the requirements for provably
secure tree hashing.

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md)

## Intellectual property

The source code in the present repository is dual-licensed under CC0 1.0
and Apache 2.0 licences.

The Rust code is copyright Jack O'Connor, 2019. 
The C code is copyright Samuel Neves and Jack O'Connor, 2019.

