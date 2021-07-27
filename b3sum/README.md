# b3sum

A command line utility for calculating
[BLAKE3](https://github.com/BLAKE3-team/BLAKE3) hashes, similar to
Coreutils tools like `b2sum` or `md5sum`.

```
b3sum 1.0.0

USAGE:
    b3sum [FLAGS] [OPTIONS] [FILE]...

FLAGS:
    -c, --check       Reads BLAKE3 sums from the [file]s and checks them
    -h, --help        Prints help information
        --keyed       Uses the keyed mode. The secret key is read from standard
                      input, and it must be exactly 32 raw bytes.
        --no-mmap     Disables memory mapping. Currently this also disables
                      multithreading.
        --no-names    Omits filenames in the output
        --quiet       Skips printing OK for each successfully verified file.
                      Must be used with --check.
        --raw         Writes raw output bytes to stdout, rather than hex.
                      --no-names is implied. In this case, only a single
                      input is allowed.
    -V, --version     Prints version information

OPTIONS:
        --derive-key <CONTEXT>    Uses the key derivation mode, with the given
                                  context string. Cannot be used with --keyed.
    -l, --length <LEN>            The number of output bytes, prior to hex
                                  encoding (default 32)
        --num-threads <NUM>       The maximum number of threads to use. By
                                  default, this is the number of logical cores.
                                  If this flag is omitted, or if its value is 0,
                                  RAYON_NUM_THREADS is also respected.

ARGS:
    <FILE>...    Files to hash, or checkfiles to check. When no file is given,
                 or when - is given, read standard input.
```

See also [this document about how the `--check` flag
works](https://github.com/BLAKE3-team/BLAKE3/blob/master/b3sum/what_does_check_do.md).

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

Prebuilt binaries are available for Linux, Windows, and macOS (requiring
the [unidentified developer
workaround](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac))
on the [releases page](https://github.com/BLAKE3-team/BLAKE3/releases).
If you've [installed Rust and
Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html),
you can also build `b3sum` yourself with:

```
cargo install b3sum
```

On Linux for example, Cargo will put the compiled binary in
`~/.cargo/bin`. You might want to add that directory to your `$PATH`, or
`rustup` might have done it for you when you installed Cargo.

If you want to install directly from this directory, you can run `cargo
install --path .`. Or you can just build with `cargo build --release`,
which puts the binary at `./target/release/b3sum`.
