use anyhow::{bail, Context, Result};
use clap::{App, Arg};
use std::cmp;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;

const FILE_ARG: &str = "file";
const LENGTH_ARG: &str = "length";
const KEYED_ARG: &str = "keyed";
const DERIVE_KEY_ARG: &str = "derive-key";
const NO_NAMES_ARG: &str = "no-names";

fn clap_parse_argv() -> clap::ArgMatches<'static> {
    App::new("b3sum")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name(FILE_ARG).multiple(true))
        .arg(
            Arg::with_name(LENGTH_ARG)
                .long(LENGTH_ARG)
                .short("l")
                .takes_value(true)
                .value_name("LEN")
                .default_value("32")
                .help("The number of output bytes, prior to hex encoding"),
        )
        .arg(
            Arg::with_name(KEYED_ARG)
                .long(KEYED_ARG)
                .requires(FILE_ARG)
                .help("Uses the keyed mode, with the 32-byte key read from stdin"),
        )
        .arg(
            Arg::with_name(DERIVE_KEY_ARG)
                .long(DERIVE_KEY_ARG)
                .conflicts_with(KEYED_ARG)
                .requires(FILE_ARG)
                .help("Uses the KDF mode, with the 32-byte key read from stdin"),
        )
        .arg(
            Arg::with_name(NO_NAMES_ARG)
                .long(NO_NAMES_ARG)
                .help("Omits filenames in the output"),
        )
        .get_matches()
}

// The slow path, for inputs that we can't memmap.
fn hash_reader(
    base_hasher: &blake3::Hasher,
    mut reader: impl Read,
) -> Result<blake3::OutputReader> {
    let mut hasher = base_hasher.clone();
    // TODO: This is a narrow copy, so it might not take advantage of SIMD or
    // threads. With a larger buffer size, most of that performance can be
    // recovered. However, this requires some platform-specific tuning, based
    // on both the SIMD degree and the number of cores. A double-buffering
    // strategy is also helpful, where a dedicated background thread reads
    // input into one buffer while another thread is calling update() on a
    // second buffer. Since this is the slow path anyway, do the simple thing
    // for now.
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize_xof())
}

#[cfg(feature = "memmap")]
fn maybe_memmap_file(file: &File) -> Result<Option<memmap::Mmap>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    Ok(if !metadata.is_file() {
        // Not a real file.
        None
    } else if file_size > isize::max_value() as u64 {
        // Too long to safely map.
        // https://github.com/danburkert/memmap-rs/issues/69
        None
    } else if file_size == 0 {
        // Mapping an empty file currently fails.
        // https://github.com/danburkert/memmap-rs/issues/72
        None
    } else {
        // Explicitly set the length of the memory map, so that filesystem
        // changes can't race to violate the invariants we just checked.
        let map = unsafe {
            memmap::MmapOptions::new()
                .len(file_size as usize)
                .map(&file)?
        };
        Some(map)
    })
}

// The fast path: Try to hash a file by mem-mapping it first. This is faster if
// it works, but it's not always possible.
fn maybe_hash_memmap(
    _base_hasher: &blake3::Hasher,
    _file: &File,
) -> Result<Option<blake3::OutputReader>> {
    #[cfg(feature = "memmap")]
    {
        if let Some(map) = maybe_memmap_file(_file)? {
            return Ok(Some(_base_hasher.clone().update(&map).finalize_xof()));
        }
    }
    Ok(None)
}

fn write_output(mut output: blake3::OutputReader, mut len: u64) -> Result<()> {
    // Encoding multiples of the block size is most efficient.
    let mut block = [0; blake3::BLOCK_LEN];
    while len > 0 {
        output.fill(&mut block);
        let hex_str = hex::encode(&block[..]);
        let take_bytes = cmp::min(len, block.len() as u64);
        print!("{}", &hex_str[..2 * take_bytes as usize]);
        len -= take_bytes;
    }
    Ok(())
}

// Errors from this function get handled by the file loop and printed per-file.
fn hash_file(
    base_hasher: &blake3::Hasher,
    filepath: &std::ffi::OsStr,
) -> Result<blake3::OutputReader> {
    let file = File::open(filepath)?;
    if let Some(output) = maybe_hash_memmap(&base_hasher, &file)? {
        Ok(output) // the fast path
    } else {
        // the slow path
        hash_reader(&base_hasher, file)
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

fn main() -> Result<()> {
    let args = clap_parse_argv();
    let len: u64 = args
        .value_of(LENGTH_ARG)
        .unwrap()
        .parse()
        .context("Failed to parse length.")?;
    let base_hasher = if args.is_present(KEYED_ARG) {
        blake3::Hasher::new_keyed(&read_key_from_stdin()?)
    } else if args.is_present(DERIVE_KEY_ARG) {
        blake3::Hasher::new_derive_key(&read_key_from_stdin()?)
    } else {
        blake3::Hasher::new()
    };
    let print_names = !args.is_present(NO_NAMES_ARG);
    let mut did_error = false;
    if let Some(files) = args.values_of_os(FILE_ARG) {
        for filepath in files {
            let filepath_str = filepath.to_string_lossy();
            match hash_file(&base_hasher, filepath) {
                Ok(output) => {
                    write_output(output, len)?;
                    if print_names {
                        println!("  {}", filepath_str);
                    } else {
                        println!();
                    }
                }
                Err(e) => {
                    did_error = true;
                    println!("b3sum: {}: {}", filepath_str, e);
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        let output = hash_reader(&base_hasher, stdin)?;
        write_output(output, len)?;
        println!();
    }
    std::process::exit(if did_error { 1 } else { 0 });
}
