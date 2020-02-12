# b3sum

```
b3sum 0.2.0

USAGE:
    b3sum [FLAGS] [OPTIONS] [file]...

FLAGS:
    -h, --help        Prints help information
        --keyed       Uses the keyed mode, with the raw 32-byte key read from stdin
        --no-mmap     Disables memory mapping
        --no-names    Omits filenames in the output
        --raw         Writes raw output bytes to stdout, rather than hex. --no-names is implied.
                      In this case, only a single input is allowed
    -V, --version     Prints version information

OPTIONS:
        --derive-key <CONTEXT>    Uses the key derivation mode, with the input as key material
    -l, --length <LEN>            The number of output bytes, prior to hex encoding (default 32)

ARGS:
    <file>...
```

# Building

The standard way to install `b3sum` is:

```
cargo install b3sum
```

On Linux for example, Cargo will put the compiled binary in
`~/.cargo/bin`. You might want to add that directory to your `$PATH`, or
`rustup` might have done it for you when you installed Cargo.

If you want to install directly from this directory, you can run `cargo
install --path .`. Or you can just build with `cargo build --release`,
which puts the binary at `./target/release/b3sum`.

AVX-512 support (via C FFI, with dynamic CPU feature detection) and
multi-threading (via Rayon) are enabled by default. Note that the
underlying `blake3` crate does not enable those by default.
