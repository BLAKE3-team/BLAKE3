# b3sum

A command line utility for calculating
[BLAKE3](https://github.com/BLAKE3-team/BLAKE3) hashes, similar to
Coreutils tools like `b2sum` or `md5sum`.

```
b3sum 0.2.2

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

# Example

Hash the file `foo.txt`:

```bash
b3sum foo.txt
```

Time hashing a gigabyte of data, to see how fast it is:

```bash
# Create a 1 GB file.
head -c 1000000000 /dev/zero > /tmp/bigfile
# Hash it with SHA-256.
time openssl sha256 /tmp/bigfile
# Hash it with BLAKE3.
time b3sum /tmp/bigfile
```


# Installation

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

By default, `b3sum` enables the assembly implementations, AVX-512
support, and multi-threading features of the underlying
[`blake3`](https://crates.io/crates/blake3) crate. To avoid this (for
example, if your C compiler does not support AVX-512), you can use
Cargo's `--no-default-features` flag.
