use anyhow::{bail, Context, Result};
use clap::{App, Arg};
use std::cmp;
use std::convert::TryInto;
use std::fs::File;
use std::io::{prelude::*, ErrorKind};

const FILE_ARG: &str = "file";
const LENGTH_ARG: &str = "length";
const KEYED_ARG: &str = "keyed";
const DERIVE_KEY_ARG: &str = "derive-key";
const BUFFER_SIZE: &str = "buffer-size";
const NO_MMAP_ARG: &str = "no-mmap";
const NO_NAMES_ARG: &str = "no-names";
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
                .help("The number of output bytes, prior to hex encoding (default 32)"),
        )
        .arg(
            Arg::with_name(KEYED_ARG)
                .long(KEYED_ARG)
                .requires(FILE_ARG)
                .help("Uses the keyed mode, with the raw 32-byte key read from stdin"),
        )
        .arg(
            Arg::with_name(DERIVE_KEY_ARG)
                .long(DERIVE_KEY_ARG)
                .conflicts_with(KEYED_ARG)
                .takes_value(true)
                .value_name("CONTEXT")
                .help("Uses the key derivation mode, with the input as key material"),
        )
        .arg(
            Arg::with_name(BUFFER_SIZE)
                .long(BUFFER_SIZE)
                .takes_value(true)
                .value_name("SIZE")
                .hidden(true)
                .help("Input buffer size"),
        )
        .arg(
            Arg::with_name(NO_MMAP_ARG)
                .long(NO_MMAP_ARG)
                .help("Never use memory maps"),
        )
        .arg(
            Arg::with_name(NO_NAMES_ARG)
                .long(NO_NAMES_ARG)
                .help("Omits filenames in the output"),
        )
        .arg(
            Arg::with_name(RAW_ARG)
                .long(RAW_ARG)
                .help("Writes raw output bytes to stdout, rather than hex. --no-names is implied.\nIn this case, only a single input is allowed"),
        )
        .get_matches()
}

enum LazyBuffer {
    Size(usize),
    Buffer(Box<[u8]>),
}

impl LazyBuffer {
    fn new(size: usize) -> Self {
        Self::Size(size)
    }

    fn get(&mut self) -> &mut [u8] {
        if let Self::Size(size) = *self {
            *self = Self::Buffer(vec![0; size].into_boxed_slice());
        }

        match *self {
            Self::Size(_) => unreachable!(),
            Self::Buffer(ref mut buf) => buf,
        }
    }
}

// The buffer should be as large of possible, while still fitting into the L3
// cache. Most desktop processors have at least 1 MiB of L3 cache per physical
// core, and at most 2 threads per core, so we should use at most 512 KiB per
// logical core. Use half that value just to be safe. TODO: benchmark
fn default_buffer_size() -> usize {
    256 * 1024 * num_cpus::get()
}

// The slow path, for inputs that we can't memmap.
fn hash_reader(
    base_hasher: &blake3::Hasher,
    mut reader: impl Read,
    buffer: &mut [u8],
) -> Result<blake3::OutputReader> {
    let mut hasher = base_hasher.clone();
    // TODO: A double-buffering strategy might also be helpful, where a
    // dedicated background thread reads input into one buffer while another
    // thread is calling update() on a second buffer.
    loop {
        match reader.read(buffer) {
            Ok(0) => return Ok(hasher.finalize_xof()),
            Ok(len) => {
                hasher.update(&buffer[..len]);
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e)?,
        }
    }
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
    mmap: bool,
) -> Result<Option<blake3::OutputReader>> {
    #[cfg(feature = "memmap")]
    {
        if mmap {
            if let Some(map) = maybe_memmap_file(_file)? {
                return Ok(Some(_base_hasher.clone().update(&map).finalize_xof()));
            }
        }
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
    mmap: bool,
    buffer: &mut LazyBuffer,
) -> Result<blake3::OutputReader> {
    let file = File::open(filepath)?;
    if let Some(output) = maybe_hash_memmap(&base_hasher, &file, mmap)? {
        Ok(output) // the fast path
    } else {
        // the slow path
        hash_reader(&base_hasher, file, buffer.get())
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
        .unwrap_or("32")
        .parse()
        .context("Failed to parse length.")?;
    let base_hasher = if args.is_present(KEYED_ARG) {
        blake3::Hasher::new_keyed(&read_key_from_stdin()?)
    } else if let Some(context) = args.value_of(DERIVE_KEY_ARG) {
        blake3::Hasher::new_derive_key(context)
    } else {
        blake3::Hasher::new()
    };
    let buffer_size = args
        .value_of(BUFFER_SIZE)
        .unwrap_or("0")
        .parse()
        .context("Failed to parse buffer size.")?;
    let mut buffer = LazyBuffer::new(if buffer_size > 0 {
        buffer_size
    } else {
        default_buffer_size()
    });
    let mmap = !args.is_present(NO_MMAP_ARG);
    let print_names = !args.is_present(NO_NAMES_ARG);
    let raw_output = args.is_present(RAW_ARG);
    let mut did_error = false;

    if let Some(files) = args.values_of_os(FILE_ARG) {
        if raw_output && files.len() > 1 {
            bail!("b3sum: Only one filename can be provided when using --raw");
        }
        for filepath in files {
            let filepath_str = filepath.to_string_lossy();
            match hash_file(&base_hasher, filepath, mmap, &mut buffer) {
                Ok(output) => {
                    if raw_output {
                        write_raw_output(output, len)?;
                    } else {
                        write_hex_output(output, len)?;
                        if print_names {
                            println!("  {}", filepath_str);
                        } else {
                            println!();
                        }
                    }
                }
                Err(e) => {
                    did_error = true;
                    eprintln!("b3sum: {}: {}", filepath_str, e);
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        let output = hash_reader(&base_hasher, stdin, buffer.get())?;
        if raw_output {
            write_raw_output(output, len)?;
        } else {
            write_hex_output(output, len)?;
            println!();
        }
    }
    std::process::exit(if did_error { 1 } else { 0 });
}
