use anyhow::{bail, ensure, Context, Result};
use clap::{App, Arg};
use std::borrow::Cow;
use std::cmp;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[cfg(test)]
mod unit_tests;

const FILE_ARG: &str = "file";
const DERIVE_KEY_ARG: &str = "derive-key";
const KEYED_ARG: &str = "keyed";
const LENGTH_ARG: &str = "length";
const NO_MMAP_ARG: &str = "no-mmap";
const NO_NAMES_ARG: &str = "no-names";
const NUM_THREADS_ARG: &str = "num-threads";
const RAW_ARG: &str = "raw";

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
        .arg(
            Arg::with_name(NO_MMAP_ARG)
                .long(NO_MMAP_ARG)
                .help("Disables memory mapping"),
        )
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
        .get_matches()
}

// A 16 KiB buffer is enough to take advantage of all the SIMD instruction sets
// that we support, but `std::io::copy` currently uses 8 KiB. Most platforms
// can support at least 64 KiB, and there's some performance benefit to using
// bigger reads, so that's what we use here.
fn copy_wide(mut reader: impl Read, hasher: &mut blake3::Hasher) -> io::Result<u64> {
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

// The slow path, for inputs that we can't memmap.
fn hash_reader(base_hasher: &blake3::Hasher, reader: impl Read) -> Result<blake3::OutputReader> {
    let mut hasher = base_hasher.clone();
    // This is currently all single-threaded. Doing multi-threaded hashing
    // without memory mapping is tricky, since all your worker threads have to
    // stop every time you refill the buffer, and that ends up being a lot of
    // overhead. To solve that, we need a more complicated double-buffering
    // strategy where a background thread fills one buffer while the worker
    // threads are hashing the other one. We might implement that in the
    // future, but since this is the slow path anyway, it's not high priority.
    copy_wide(reader, &mut hasher)?;
    Ok(hasher.finalize_xof())
}

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
    } else if file_size < 16 * 1024 {
        // Mapping small files is not worth it.
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
    if let Some(map) = maybe_memmap_file(_file)? {
        // Memory mapping worked. Use Rayon-based multi-threading to split
        // up the whole file across many worker threads.
        return Ok(Some(
            _base_hasher
                .clone()
                .update_with_join::<blake3::join::RayonJoin>(&map)
                .finalize_xof(),
        ));
    }
    Ok(None)
}

fn write_hex_output(mut output: blake3::OutputReader, mut len: u64) -> Result<()> {
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

fn write_raw_output(output: blake3::OutputReader, len: u64) -> Result<()> {
    let mut output = output.take(len);
    let stdout = std::io::stdout();
    let mut handler = stdout.lock();
    std::io::copy(&mut output, &mut handler)?;

    Ok(())
}

// Errors from this function get handled by the file loop and printed per-file.
fn hash_file(
    base_hasher: &blake3::Hasher,
    filepath: &std::ffi::OsStr,
    mmap_disabled: bool,
) -> Result<blake3::OutputReader> {
    let file = File::open(filepath)?;
    if !mmap_disabled {
        if let Some(output) = maybe_hash_memmap(&base_hasher, &file)? {
            return Ok(output); // the fast path
        }
    }
    // the slow path
    hash_reader(&base_hasher, file)
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
    has_escapes: bool,
}

// returns (string, did_escape)
fn filepath_to_string(filepath_osstr: &OsStr) -> FilepathString {
    let unicode_cow = filepath_osstr.to_string_lossy();
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
    let mut has_escapes = false;
    if filepath_string.contains('\\') || filepath_string.contains('\n') {
        filepath_string = filepath_string.replace('\\', "\\\\").replace('\n', "\\n");
        has_escapes = true;
    }
    FilepathString {
        filepath_string,
        has_escapes,
    }
}

fn hex_half_byte(c: char) -> Result<u8> {
    // The hex characters in the hash must be lowercase for now, though we
    // could support uppercase too if we wanted to.
    if '0' <= c && c <= '9' {
        return Ok(c as u8 - '0' as u8);
    }
    if 'a' <= c && c <= 'f' {
        return Ok(c as u8 - 'a' as u8 + 10);
    }
    bail!("b3sum: Invalid hex");
}

// The `check` command is a security tool. That means it's much better for a
// check to fail more often than it should (a false negative), than for a check
// to ever succeed when it shouldn't (a false positive). By forbidding certain
// characters in checked filepaths, we avoid a class of false positives where
// two different filepaths can get confused with each other.
fn check_for_invalid_characters(path: &str) -> Result<()> {
    // Null characters in paths should never happen, but they can result in a
    // path getting silently truncated on Unix.
    if path.contains('\0') {
        bail!("b3sum: Null character in path");
    }
    // Because we convert invalid UTF-8 sequences in paths to the Unicode
    // replacement character, multiple different invalid paths can map to the
    // same UTF-8 string.
    if path.contains('�') {
        bail!("b3sum: Unicode replacement character in path");
    }
    // We normalize all Windows backslashes to forward slashes in our output,
    // so the only natural way to get a backslash in a checkfile on Windows is
    // to construct it on Unix and copy it over. (Or of course you could just
    // doctor it by hand.) To avoid confusing this with a directory separator,
    // we forbid backslashes entirely on Windows. Note that this check comes
    // after unescaping has been done.
    if cfg!(windows) && path.contains('\\') {
        bail!("b3sum: Backslash in path");
    }
    Ok(())
}

fn unescape(mut path: &str) -> Result<String> {
    let mut unescaped = String::with_capacity(2 * path.len());
    while let Some(i) = path.find('\\') {
        ensure!(i < path.len() - 1, "b3sum: Invalid backslash escape");
        unescaped.push_str(&path[..i]);
        match path[i + 1..].chars().next().unwrap() {
            // Anything other than a recognized escape sequence is an error.
            'n' => unescaped.push_str("\n"),
            '\\' => unescaped.push_str("\\"),
            _ => bail!("b3sum: Invalid backslash escape"),
        }
        path = &path[i + 2..];
    }
    unescaped.push_str(path);
    Ok(unescaped)
}

fn parse_check_line(mut line: &str) -> Result<(blake3::Hash, Cow<str>)> {
    // Trim off the trailing newline, if any.
    line = line.trim_end_matches('\n');
    // If there's a backslash at the front of the line, that means we need to
    // unescape the path below. This matches the behavior of e.g. md5sum.
    let first = if let Some(c) = line.chars().next() {
        c
    } else {
        bail!("b3sum: Empty line");
    };
    let mut escaped = false;
    if first == '\\' {
        escaped = true;
        line = &line[1..];
    }
    // The front of the line must be a hash of the usual length, followed by
    // two spaces. The hex characters in the hash must be lowercase for now,
    // though we could support uppercase too if we wanted to.
    let hash_hex_len = 2 * blake3::OUT_LEN;
    let num_spaces = 2;
    let prefix_len = hash_hex_len + num_spaces;
    ensure!(line.len() > prefix_len, "b3sum: Short line");
    ensure!(
        line.chars().take(prefix_len).all(|c| c.is_ascii()),
        "b3sum: Non-ASCII prefix"
    );
    ensure!(&line[hash_hex_len..][..2] == "  ", "b3sum: Invalid space");
    // Decode the hash hex.
    let mut hash_bytes = [0; blake3::OUT_LEN];
    let mut hex_chars = line[..hash_hex_len].chars();
    for byte in &mut hash_bytes {
        let high_char = hex_chars.next().unwrap();
        let low_char = hex_chars.next().unwrap();
        *byte = 16 * hex_half_byte(high_char)? + hex_half_byte(low_char)?;
    }
    let hash: blake3::Hash = hash_bytes.into();
    let path_str = &line[prefix_len..];
    let path_cow: Cow<str> = if escaped {
        // If we detected a backslash at the start of the line earlier, now we
        // need to unescape backslashes and newlines.
        Cow::Owned(unescape(path_str)?)
    } else {
        Cow::Borrowed(path_str)
    };
    check_for_invalid_characters(&path_cow)?;
    Ok((hash, path_cow))
}

fn main() -> Result<()> {
    let args = clap_parse_argv();
    let len = if let Some(length) = args.value_of(LENGTH_ARG) {
        length.parse::<u64>().context("Failed to parse length.")?
    } else {
        blake3::OUT_LEN as u64
    };
    let base_hasher = if args.is_present(KEYED_ARG) {
        blake3::Hasher::new_keyed(&read_key_from_stdin()?)
    } else if let Some(context) = args.value_of(DERIVE_KEY_ARG) {
        blake3::Hasher::new_derive_key(context)
    } else {
        blake3::Hasher::new()
    };
    let mmap_disabled = args.is_present(NO_MMAP_ARG);
    let print_names = !args.is_present(NO_NAMES_ARG);
    let raw_output = args.is_present(RAW_ARG);
    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(num_threads_str) = args.value_of(NUM_THREADS_ARG) {
        let num_threads: usize = num_threads_str
            .parse()
            .context("Failed to parse num threads.")?;
        thread_pool_builder = thread_pool_builder.num_threads(num_threads);
    }

    let thread_pool = thread_pool_builder.build()?;
    thread_pool.install(|| {
        let mut did_error = false;
        if let Some(files) = args.values_of_os(FILE_ARG) {
            if raw_output && files.len() > 1 {
                bail!("b3sum: Only one filename can be provided when using --raw");
            }
            for filepath_osstr in files {
                let FilepathString {
                    filepath_string,
                    has_escapes,
                } = filepath_to_string(filepath_osstr);
                match hash_file(&base_hasher, filepath_osstr, mmap_disabled) {
                    Ok(output) => {
                        if raw_output {
                            write_raw_output(output, len)?;
                        } else {
                            if has_escapes {
                                print!("\\");
                            }
                            write_hex_output(output, len)?;
                            if print_names {
                                println!("  {}", filepath_string);
                            } else {
                                println!();
                            }
                        }
                    }
                    Err(e) => {
                        did_error = true;
                        eprintln!("b3sum: {}: {}", filepath_string, e);
                    }
                }
            }
        } else {
            let stdin = std::io::stdin();
            let stdin = stdin.lock();
            let output = hash_reader(&base_hasher, stdin)?;
            if raw_output {
                write_raw_output(output, len)?;
            } else {
                write_hex_output(output, len)?;
                println!();
            }
        }
        std::process::exit(if did_error { 1 } else { 0 });
    })
}
