This is the C implementation of BLAKE3. It's tested, and parts of it are
linked into the Rust implementation for AVX-512 and NEON support.
However, it doesn't yet have a friendly public interface.

This implementation is simpler than the Rust implementation. It doesn't
support multithreading, and it doesn't parallelize parent hashes, so
throughput is lower.
