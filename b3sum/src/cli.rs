use clap::{Parser, ValueHint};
use std::path::PathBuf;

const DERIVE_KEY_ARG: &str = "derive_key";
const KEYED_ARG: &str = "keyed";
const LENGTH_ARG: &str = "length";
const NO_NAMES_ARG: &str = "no_names";
const RAW_ARG: &str = "raw";
const CHECK_ARG: &str = "check";

/// Print or check BLAKE3 checksums.
///
/// With no FILE, or when FILE is -, read standard input.
#[derive(Parser)]
#[command(version, max_term_width(100))]
pub struct Inner {
    /// Files to hash, or checkfiles to check
    ///
    /// When no file is given, or when - is given, read standard input.
    #[arg(value_hint(ValueHint::FilePath))]
    pub file: Vec<PathBuf>,

    /// Use the keyed mode, reading the 32-byte key from stdin
    #[arg(long, requires("file"))]
    pub keyed: bool,

    /// Use the key derivation mode, with the given context string
    ///
    /// Cannot be used with --keyed.
    #[arg(long, value_name("CONTEXT"), conflicts_with(KEYED_ARG))]
    pub derive_key: Option<String>,

    /// The number of output bytes, before hex encoding
    #[arg(
        short,
        long,
        default_value_t = blake3::OUT_LEN as u64,
        value_name("LEN")
    )]
    pub length: u64,

    /// The starting output byte offset, before hex encoding
    #[arg(long, default_value_t = 0, value_name("SEEK"))]
    pub seek: u64,

    /// The maximum number of threads to use
    ///
    /// By default, this is the number of logical cores. If this flag is
    /// omitted, or if its value is 0, RAYON_NUM_THREADS is also respected.
    #[arg(long, value_name("NUM"))]
    pub num_threads: Option<usize>,

    /// Disable memory mapping
    ///
    /// Currently this also disables multithreading.
    #[arg(long)]
    pub no_mmap: bool,

    /// Omit filenames in the output
    #[arg(long)]
    pub no_names: bool,

    /// Write raw output bytes to stdout, rather than hex
    ///
    /// --no-names is implied. In this case, only a single input is allowed.
    #[arg(long)]
    pub raw: bool,

    /// Read BLAKE3 sums from the [FILE]s and check them
    #[arg(
        short,
        long,
        conflicts_with(DERIVE_KEY_ARG),
        conflicts_with(KEYED_ARG),
        conflicts_with(LENGTH_ARG),
        conflicts_with(RAW_ARG),
        conflicts_with(NO_NAMES_ARG)
    )]
    pub check: bool,

    /// Skip printing OK for each checked file
    ///
    /// Must be used with --check.
    #[arg(long, requires(CHECK_ARG))]
    pub quiet: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_args() {
        Inner::command().debug_assert();
    }
}
