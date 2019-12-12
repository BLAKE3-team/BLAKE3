use anyhow::{bail, Context, Result};
use arrayref::array_ref;
use clap::{App, Arg};
use std::cmp;
use std::fs::File;
use std::io::prelude::*;

fn clap_parse_argv() -> clap::ArgMatches<'static> {
    App::new("b3sum")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("file").multiple(true))
        .arg(
            Arg::with_name("length")
                .long("length")
                .short("l")
                .takes_value(true)
                .value_name("LEN")
                .help("The number of output bytes, prior to hex."),
        )
        .arg(
            Arg::with_name("key")
                .long("key")
                .takes_value(true)
                .value_name("KEY")
                .help("The keyed hashing mode."),
        )
        .arg(
            Arg::with_name("derive-key")
                .long("derive-key")
                .takes_value(true)
                .value_name("KEY")
                .conflicts_with("key")
                .help("The key derivation mode."),
        )
        .get_matches()
}

fn parse_key(key_str: &str) -> Result<[u8; blake3::KEY_LEN]> {
    if key_str.len() != 2 * blake3::KEY_LEN {
        bail!("Key must be 64 hex chars, got {}.", key_str.len());
    }
    let bytes = hex::decode(key_str).context("Failed to parse key bytes.")?;
    Ok(*array_ref!(bytes, 0, blake3::KEY_LEN))
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
    base_hasher: &blake3::Hasher,
    file: &File,
) -> Result<Option<blake3::OutputReader>> {
    #[cfg(feature = "memmap")]
    {
        if let Some(map) = maybe_memmap_file(file)? {
            return Ok(Some(base_hasher.clone().update(&map).finalize_xof()));
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

fn main() -> Result<()> {
    let matches = clap_parse_argv();
    let len: u64 = if let Some(len) = matches.value_of("length") {
        len.parse().context("Failed to parse length.")?
    } else {
        blake3::OUT_LEN as u64
    };
    let base_hasher = if let Some(key_str) = matches.value_of("key") {
        blake3::Hasher::new_keyed(&parse_key(key_str)?)
    } else if let Some(key_str) = matches.value_of("derive-key") {
        blake3::Hasher::new_derive_key(&parse_key(key_str)?)
    } else {
        blake3::Hasher::new()
    };
    let mut did_error = false;
    if let Some(files) = matches.values_of_os("file") {
        let print_names = files.len() > 1;
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
