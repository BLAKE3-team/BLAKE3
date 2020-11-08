use anyhow::{bail, Context, Result};
use clap::{App, Arg};
use std::convert::TryInto;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use b3sum::*;

const FILE_ARG: &str = "FILE";
const DERIVE_KEY_ARG: &str = "derive-key";
const KEYED_ARG: &str = "keyed";
const LENGTH_ARG: &str = "length";
const NO_MMAP_ARG: &str = "no-mmap";
const NO_NAMES_ARG: &str = "no-names";
const NUM_THREADS_ARG: &str = "num-threads";
const RAW_ARG: &str = "raw";
const CHECK_ARG: &str = "check";
const QUIET_ARG: &str = "quiet";

struct Args {
    inner: clap::ArgMatches<'static>,
    file_args: Vec<PathBuf>,
    generator: HasherGenerator,
}

impl Args {
    fn parse() -> Result<Self> {
        let inner = App::new(NAME)
            .version(env!("CARGO_PKG_VERSION"))
            .arg(Arg::with_name(FILE_ARG).multiple(true).help(
                "Files to hash, or checkfiles to check. When no file is given,\n\
                 or when - is given, read standard input.",
            ))
            .arg(
                Arg::with_name(LENGTH_ARG)
                    .long(LENGTH_ARG)
                    .short("l")
                    .takes_value(true)
                    .value_name("LEN")
                    .help(
                        "The number of output bytes, prior to hex\n\
                         encoding (default 32)",
                    ),
            )
            .arg(
                Arg::with_name(NUM_THREADS_ARG)
                    .long(NUM_THREADS_ARG)
                    .takes_value(true)
                    .value_name("NUM")
                    .help(
                        "The maximum number of threads to use. By\n\
                         default, this is the number of logical cores.\n\
                         If this flag is omitted, or if its value is 0,\n\
                         RAYON_NUM_THREADS is also respected.",
                    ),
            )
            .arg(
                Arg::with_name(KEYED_ARG)
                    .long(KEYED_ARG)
                    .requires(FILE_ARG)
                    .help(
                        "Uses the keyed mode. The secret key is read from standard\n\
                         input, and it must be exactly 32 raw bytes.",
                    ),
            )
            .arg(
                Arg::with_name(DERIVE_KEY_ARG)
                    .long(DERIVE_KEY_ARG)
                    .conflicts_with(KEYED_ARG)
                    .takes_value(true)
                    .value_name("CONTEXT")
                    .help(
                        "Uses the key derivation mode, with the given\n\
                         context string. Cannot be used with --keyed.",
                    ),
            )
            .arg(Arg::with_name(NO_MMAP_ARG).long(NO_MMAP_ARG).help(
                "Disables memory mapping. Currently this also disables\n\
                 multithreading.",
            ))
            .arg(
                Arg::with_name(NO_NAMES_ARG)
                    .long(NO_NAMES_ARG)
                    .help("Omits filenames in the output"),
            )
            .arg(Arg::with_name(RAW_ARG).long(RAW_ARG).help(
                "Writes raw output bytes to stdout, rather than hex.\n\
                 --no-names is implied. In this case, only a single\n\
                 input is allowed.",
            ))
            .arg(
                Arg::with_name(CHECK_ARG)
                    .long(CHECK_ARG)
                    .short("c")
                    .conflicts_with(DERIVE_KEY_ARG)
                    .conflicts_with(KEYED_ARG)
                    .conflicts_with(LENGTH_ARG)
                    .conflicts_with(RAW_ARG)
                    .conflicts_with(NO_NAMES_ARG)
                    .help("Reads BLAKE3 sums from the [file]s and checks them"),
            )
            .arg(
                Arg::with_name(QUIET_ARG)
                    .long(QUIET_ARG)
                    .requires(CHECK_ARG)
                    .help(
                        "Skips printing OK for each successfully verified file.\n\
                         Must be used with --check.",
                    ),
            )
            // wild::args_os() is equivalent to std::env::args_os() on Unix,
            // but on Windows it adds support for globbing.
            .get_matches_from(wild::args_os());
        let file_args = if let Some(iter) = inner.values_of_os(FILE_ARG) {
            iter.map(|s| s.into()).collect()
        } else {
            vec!["-".into()]
        };
        if inner.is_present(RAW_ARG) && file_args.len() > 1 {
            bail!("Only one filename can be provided when using --raw");
        }

        let generator = if inner.is_present(KEYED_ARG) {
            // In keyed mode, since stdin is used for the key, we can't handle
            // `-` arguments. Input::open handles that case below.
            HasherGenerator::new_keyed(&read_key_from_stdin()?)
        } else if let Some(context) = inner.value_of(DERIVE_KEY_ARG) {
            HasherGenerator::new_derive_key(context)
        } else {
            HasherGenerator::new()
        };

        Ok(Self {
            inner,
            file_args,
            generator,
        })
    }

    fn num_threads(&self) -> Result<Option<usize>> {
        if let Some(num_threads_str) = self.inner.value_of(NUM_THREADS_ARG) {
            Ok(Some(
                num_threads_str
                    .parse()
                    .context("Failed to parse num threads.")?,
            ))
        } else {
            Ok(None)
        }
    }

    fn check(&self) -> bool {
        self.inner.is_present(CHECK_ARG)
    }

    fn raw(&self) -> bool {
        self.inner.is_present(RAW_ARG)
    }

    fn no_mmap(&self) -> bool {
        self.inner.is_present(NO_MMAP_ARG)
    }

    fn no_names(&self) -> bool {
        self.inner.is_present(NO_NAMES_ARG)
    }

    fn len(&self) -> Result<u64> {
        if let Some(length) = self.inner.value_of(LENGTH_ARG) {
            length.parse::<u64>().context("Failed to parse length.")
        } else {
            Ok(blake3::OUT_LEN as u64)
        }
    }

    fn quiet(&self) -> bool {
        self.inner.is_present(QUIET_ARG)
    }
}

fn read_key_from_stdin() -> Result<[u8; blake3::KEY_LEN]> {
    let mut bytes = Vec::with_capacity(blake3::KEY_LEN + 1);
    let n = std::io::stdin()
        .lock()
        .take(blake3::KEY_LEN as u64 + 1)
        .read_to_end(&mut bytes)?;
    if n < 32 {
        bail!(
            "expected {} key bytes from stdin, found {}",
            blake3::KEY_LEN,
            n,
        )
    } else if n > 32 {
        bail!("read more than {} key bytes from stdin", blake3::KEY_LEN)
    } else {
        Ok(bytes[..blake3::KEY_LEN].try_into().unwrap())
    }
}

struct FilepathString {
    filepath_string: String,
    is_escaped: bool,
}

// returns (string, did_escape)
fn filepath_to_string(filepath: &Path) -> FilepathString {
    let unicode_cow = filepath.to_string_lossy();
    let mut filepath_string = unicode_cow.to_string();
    // If we're on Windows, normalize backslashes to forward slashes. This
    // avoids a lot of ugly escaping in the common case, and it makes
    // checkfiles created on Windows more likely to be portable to Unix. It
    // also allows us to set a blanket "no backslashes allowed in checkfiles on
    // Windows" rule, rather than allowing a Unix backslash to potentially get
    // interpreted as a directory separator on Windows.
    if cfg!(windows) {
        filepath_string = filepath_string.replace('\\', "/");
    }
    let mut is_escaped = false;
    if filepath_string.contains('\\') || filepath_string.contains('\n') {
        filepath_string = filepath_string.replace('\\', "\\\\").replace('\n', "\\n");
        is_escaped = true;
    }
    FilepathString {
        filepath_string,
        is_escaped,
    }
}

fn hash_one_input<P>(
    path: P,
    gen: &HasherGenerator,
    no_mmap: bool,
    raw: bool,
    no_names: bool,
    len: u64,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let output = hash(&path, &gen, no_mmap)?;
    if raw {
        print_hash_raw_output(output, len)?;
        return Ok(());
    }
    if no_names {
        print_hash_hex_output(output, len)?;
        println!();
        return Ok(());
    }
    let FilepathString {
        filepath_string,
        is_escaped,
    } = filepath_to_string(path.as_ref());
    if is_escaped {
        print!("\\");
    }
    print_hash_hex_output(output, len)?;
    println!("  {}", filepath_string);
    Ok(())
}

fn hash_with_args(args: &Args) -> Result<i32> {
    let mut some_file_failed = false;
    for path in &args.file_args {
        let result = hash_one_input(
            path,
            &args.generator,
            args.no_mmap(),
            args.raw(),
            args.no_names(),
            args.len()?,
        );
        if let Err(e) = result {
            some_file_failed = true;
            eprintln!("{}: {}: {}", NAME, path.to_string_lossy(), e);
        }
    }
    Ok(if some_file_failed { 1 } else { 0 })
}

fn check_with_args(args: &Args) -> Result<i32> {
    let mut some_file_failed = false;
    for path in &args.file_args {
        let failed = print_check_checkfile(path, &args.generator, args.no_mmap(), args.quiet())?;
        if failed {
            some_file_failed = true;
        }
    }
    Ok(if some_file_failed { 1 } else { 0 })
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(num_threads) = args.num_threads()? {
        thread_pool_builder = thread_pool_builder.num_threads(num_threads);
    }
    let thread_pool = thread_pool_builder.build()?;
    if args.check() {
        thread_pool.install(|| std::process::exit(check_with_args(&args)?))
    } else {
        thread_pool.install(|| std::process::exit(hash_with_args(&args)?))
    }
}
