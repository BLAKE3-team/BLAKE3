# b3sum

```
b3sum 0.1.0

USAGE:
    b3sum [FLAGS] [OPTIONS] [file]...

FLAGS:
        --derive-key    Uses the KDF mode, with the 32-byte key read from stdin
    -h, --help          Prints help information
        --keyed         Uses the keyed mode, with the 32-byte key read from stdin
        --no-names      Omits filenames in the output
    -V, --version       Prints version information

OPTIONS:
    -l, --length <LEN>    The number of output bytes, prior to hex encoding [default: 32]

ARGS:
    <file>...
```

# Building

You can build and install with `cargo install --path .`, which installs
binaries in `~/.cargo/bin` on Linux. Or you can just build with `cargo
build --release`, which puts the binary at `./target/release/b3sum`.

AVX-512 support (via C FFI, with dynamic CPU feature detection) and
multi-threading (via Rayon) are enabled by default. Note that the
underlying `blake3` crate does not enable those by default.
