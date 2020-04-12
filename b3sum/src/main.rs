use anyhow::{bail, Context, Result};
use clap::{App, Arg};
use std::cmp;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[cfg(feature = "vulkan")]
mod gpu;

#[cfg(feature = "vulkan")]
mod vulkan;

#[cfg(not(feature = "vulkan"))]
mod gpu {
    use super::*;

    pub struct Gpu;

    impl Gpu {
        #[inline]
        pub fn new() -> Self {
            Gpu
        }

        #[inline]
        pub fn maybe_hash(
            &mut self,
            _base_hasher: &blake3::gpu::GpuHasher,
            _file: &File,
        ) -> Result<Option<blake3::OutputReader>> {
            Ok(None)
        }
    }
}

const FILE_ARG: &str = "file";
const DERIVE_KEY_ARG: &str = "derive-key";
const KEYED_ARG: &str = "keyed";
const LENGTH_ARG: &str = "length";
const NO_MMAP_ARG: &str = "no-mmap";
const VULKAN_ARG: &str = "vulkan";
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
            Arg::with_name(VULKAN_ARG)
                .long(VULKAN_ARG)
                .help("Uses Vulkan for large files"),
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
    base_hasher: &blake3::gpu::GpuHasher,
    filepath: &std::ffi::OsStr,
    mmap_disabled: bool,
    gpu: Option<&mut gpu::Gpu>,
) -> Result<blake3::OutputReader> {
    let file = File::open(filepath)?;
    if let Some(gpu) = gpu {
        if let Some(output) = gpu.maybe_hash(&base_hasher, &file)? {
            return Ok(output); // the GPU path
        }
    }
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

fn main() -> Result<()> {
    let args = clap_parse_argv();
    let len = if let Some(length) = args.value_of(LENGTH_ARG) {
        length.parse::<u64>().context("Failed to parse length.")?
    } else {
        blake3::OUT_LEN as u64
    };
    let base_hasher = if args.is_present(KEYED_ARG) {
        blake3::gpu::GpuHasher::new_keyed(&read_key_from_stdin()?)
    } else if let Some(context) = args.value_of(DERIVE_KEY_ARG) {
        blake3::gpu::GpuHasher::new_derive_key(context)
    } else {
        blake3::gpu::GpuHasher::new()
    };
    let mmap_disabled = args.is_present(NO_MMAP_ARG);
    let vulkan_enabled = args.is_present(VULKAN_ARG);
    let print_names = !args.is_present(NO_NAMES_ARG);
    let raw_output = args.is_present(RAW_ARG);
    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(num_threads_str) = args.value_of(NUM_THREADS_ARG) {
        let num_threads: usize = num_threads_str
            .parse()
            .context("Failed to parse num threads.")?;
        thread_pool_builder = thread_pool_builder.num_threads(num_threads);
    }

    let mut gpu = if vulkan_enabled {
        Some(gpu::Gpu::new())
    } else {
        None
    };
    let thread_pool = thread_pool_builder.build()?;
    thread_pool.install(|| {
        let mut did_error = false;
        if let Some(files) = args.values_of_os(FILE_ARG) {
            if raw_output && files.len() > 1 {
                bail!("b3sum: Only one filename can be provided when using --raw");
            }
            for filepath in files {
                let filepath_str = filepath.to_string_lossy();
                match hash_file(&base_hasher, filepath, mmap_disabled, gpu.as_mut()) {
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
