# BLAKE3 [![Actions Status](https://github.com/veorq/BLAKE3/workflows/tests/badge.svg)](https://github.com/veorq/BLAKE3/actions)

The official Rust implementation of BLAKE3. The `b3sum` sub-crate
provides a command line implementation. SSE4.1 and AVX2 implementations
are provided in Rust, enabled by default, with dynamic CPU feature
detection. AVX-512 and NEON implementation are available via C FFI,
controlled by the `c_avx512` and `c_neon` features. Rayon-based
multi-threading is controlled by the `rayon` feature.

Eventually docs will be published on docs.rs. For now, you can build and
view the docs locally with `cargo doc --open`.
