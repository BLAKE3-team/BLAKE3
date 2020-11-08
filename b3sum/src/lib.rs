use anyhow::{bail, ensure, Error, Result};
use std::cmp;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::iter::from_fn;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(test)]
mod unit_tests;

pub const NAME: &str = "b3sum";

/// Key Strategy
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyStrategy {
    Keyed,
    DeriveKey,
    Default,
}

/// Hasher which remembers its `KeyStrategy`
#[derive(Clone, Debug)]
pub struct HasherGenerator {
    hasher: blake3::Hasher,
    key_type: KeyStrategy,
}

impl HasherGenerator {
    /// Make a `HasherGenerator` which produces keyed hashers.
    pub fn new_keyed(key: &[u8; blake3::KEY_LEN]) -> Self {
        Self {
            hasher: blake3::Hasher::new_keyed(key),
            key_type: KeyStrategy::Keyed,
        }
    }

    /// Make a `HasherGenerator` which produces hashers which derive keys from this context.
    pub fn new_derive_key(context: &str) -> Self {
        Self {
            hasher: blake3::Hasher::new_derive_key(context),
            key_type: KeyStrategy::DeriveKey,
        }
    }

    /// Make a `HasherGenerator` which produces the default hashers.
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
            key_type: KeyStrategy::Default,
        }
    }

    /// Return `true` if the generator produces hashers which were keyed.
    pub fn is_keyed(&self) -> bool {
        matches!(self.key_type, KeyStrategy::Keyed)
    }

    /// Return `true` if the generator produces hashers which come from a derived key.
    pub fn uses_derived_key(&self) -> bool {
        matches!(self.key_type, KeyStrategy::Default)
    }

    /// Return `true` if the generator produces the default hashers.
    pub fn is_default(&self) -> bool {
        matches!(self.key_type, KeyStrategy::Default)
    }

    /// Make a new hasher with a fixed `KeyStrategy`.
    pub fn make_hasher(&self) -> blake3::Hasher {
        self.hasher.clone()
    }
}

impl Default for HasherGenerator {
    fn default() -> Self {
        Self::new()
    }
}

enum Input {
    Mmap(io::Cursor<memmap::Mmap>),
    File(File),
    Stdin,
}

impl Input {
    // Open an input file, using mmap if appropriate. "-" means stdin. Note
    // that this convention applies both to command line arguments, and to
    // filepaths that appear in a checkfile.
    fn open<P>(path: P, keyed: bool, no_mmap: bool) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        if path.as_ref() == Path::new("-") {
            if keyed {
                bail!("Cannot open `-` in keyed mode");
            }
            return Ok(Self::Stdin);
        }
        let file = File::open(path)?;
        if !no_mmap {
            if let Some(mmap) = maybe_memmap_file(&file)? {
                return Ok(Self::Mmap(io::Cursor::new(mmap)));
            }
        }
        Ok(Self::File(file))
    }

    fn hash(&mut self, mut hasher: blake3::Hasher) -> Result<blake3::OutputReader> {
        match self {
            // The fast path: If we mmapped the file successfully, hash using
            // multiple threads. This doesn't work on stdin, or on some files,
            // and it can also be disabled with --no-mmap.
            Self::Mmap(cursor) => {
                hasher.update_with_join::<blake3::join::RayonJoin>(cursor.get_ref());
            }
            // The slower paths, for stdin or files we didn't/couldn't mmap.
            // This is currently all single-threaded. Doing multi-threaded
            // hashing without memory mapping is tricky, since all your worker
            // threads have to stop every time you refill the buffer, and that
            // ends up being a lot of overhead. To solve that, we need a more
            // complicated double-buffering strategy where a background thread
            // fills one buffer while the worker threads are hashing the other
            // one. We might implement that in the future, but since this is
            // the slow path anyway, it's not high priority.
            Self::File(file) => {
                copy_wide(file, &mut hasher)?;
            }
            Self::Stdin => {
                let stdin = io::stdin();
                let lock = stdin.lock();
                copy_wide(lock, &mut hasher)?;
            }
        }
        Ok(hasher.finalize_xof())
    }
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

// Mmap a file, if it looks like a good idea. Return None in cases where we
// know mmap will fail, or if the file is short enough that mmapping isn't
// worth it. However, if we do try to mmap and it fails, return the error.
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

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Mmap(cursor) => cursor.read(buf),
            Self::File(file) => file.read(buf),
            Self::Stdin => io::stdin().read(buf),
        }
    }
}

/// Open a file and hash it.
#[inline]
pub fn hash<P>(path: P, gen: &HasherGenerator, no_mmap: bool) -> Result<blake3::OutputReader>
where
    P: AsRef<Path>,
{
    Input::open(path, gen.is_keyed(), no_mmap).and_then(|mut input| input.hash(gen.make_hasher()))
}

/// Hash each file in the iterator.
pub fn hash_many<P, I>(
    paths: I,
    gen: HasherGenerator,
    no_mmap: bool,
) -> impl Iterator<Item = Result<blake3::OutputReader>>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    paths.into_iter().map(move |p| hash(p, &gen, no_mmap))
}

/// Write truncated hash output to a writer.
#[inline]
pub fn write_hash_raw_output<W>(
    output: blake3::OutputReader,
    writer: &mut W,
    len: u64,
) -> io::Result<u64>
where
    W: Write,
{
    io::copy(&mut output.take(len), writer)
}

/// Print truncated hash output to `stdout`.
#[inline]
pub fn print_hash_raw_output(output: blake3::OutputReader, len: u64) -> io::Result<u64> {
    write_hash_raw_output(output, &mut io::stdout().lock(), len)
}

/// Hash a file and write truncated hash output to a writer.
#[inline]
pub fn hash_raw<P, W>(
    path: P,
    gen: &HasherGenerator,
    no_mmap: bool,
    writer: &mut W,
    len: u64,
) -> Result<u64>
where
    P: AsRef<Path>,
    W: Write,
{
    write_hash_raw_output(hash(path, gen, no_mmap)?, writer, len).map_err(Error::new)
}

/// Hash a file and print truncated hash output to `stdout`.
#[inline]
pub fn print_hash_raw<P, W>(path: P, gen: &HasherGenerator, no_mmap: bool, len: u64) -> Result<u64>
where
    P: AsRef<Path>,
{
    hash_raw(path, gen, no_mmap, &mut io::stdout().lock(), len)
}

/// Write truncated hex encoding of hash output to a writer.
pub fn write_hash_hex_output<W>(
    mut output: blake3::OutputReader,
    writer: &mut W,
    mut len: u64,
) -> io::Result<()>
where
    W: Write,
{
    // Encoding multiples of the block size is most efficient.
    let mut block = [0; blake3::BLOCK_LEN];
    while len > 0 {
        output.fill(&mut block);
        let hex_str = hex::encode(&block[..]);
        let take_bytes = cmp::min(len, block.len() as u64);
        writer.write_all(&hex_str[..2 * take_bytes as usize].as_bytes())?;
        len -= take_bytes;
    }
    Ok(())
}

/// Print truncated hex encoding of hash output to `stdout`.
#[inline]
pub fn print_hash_hex_output(output: blake3::OutputReader, len: u64) -> io::Result<()> {
    write_hash_hex_output(output, &mut io::stdout().lock(), len)
}

/// Hash a file and write truncated hex encoding of hash output to a writer.
#[inline]
pub fn hash_hex<P, W>(
    path: P,
    gen: &HasherGenerator,
    no_mmap: bool,
    writer: &mut W,
    len: u64,
) -> Result<()>
where
    P: AsRef<Path>,
    W: Write,
{
    write_hash_hex_output(hash(path, gen, no_mmap)?, writer, len).map_err(Error::new)
}

/// Hash a file and print truncated hex encoding of hash output to `stdout`.
#[inline]
pub fn print_hash_hex<P>(path: P, gen: &HasherGenerator, no_mmap: bool, len: u64) -> Result<()>
where
    P: AsRef<Path>,
{
    hash_hex(path, gen, no_mmap, &mut io::stdout().lock(), len)
}

/// CheckLine Structure
///
/// Container for a parsed line of a checkfile.
#[derive(Debug)]
pub struct CheckLine {
    file_string: String,
    is_escaped: bool,
    file_path: PathBuf,
    expected_hash: blake3::Hash,
}

impl CheckLine {
    /// Get formatted file string from `Checkline`.
    pub fn file_string(&self) -> String {
        if self.is_escaped {
            "\\".to_string() + &self.file_string
        } else {
            self.file_string.clone()
        }
    }

    /// Get file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the expected hash.
    pub fn expected_hash(&self) -> &blake3::Hash {
        &self.expected_hash
    }

    /// Check that the hash of the file matches the expected hash.
    pub fn check(&self, gen: &HasherGenerator, no_mmap: bool) -> Result<bool> {
        let hash: blake3::Hash = Input::open(&self.file_path, gen.is_keyed(), no_mmap)
            .and_then(|mut input| input.hash(gen.make_hasher()))
            .map(|mut hash_output| {
                let mut found_hash_bytes = [0; blake3::OUT_LEN];
                hash_output.fill(&mut found_hash_bytes);
                found_hash_bytes.into()
            })?;
        // This is a constant-time comparison.
        Ok(self.expected_hash == hash)
    }
}

impl FromStr for CheckLine {
    type Err = Error;

    fn from_str(mut line: &str) -> Result<Self> {
        // Trim off the trailing newline, if any.
        line = line.trim_end_matches('\n');
        // If there's a backslash at the front of the line, that means we need to
        // unescape the path below. This matches the behavior of e.g. md5sum.
        let first = if let Some(c) = line.chars().next() {
            c
        } else {
            bail!("Empty line");
        };
        let mut is_escaped = false;
        if first == '\\' {
            is_escaped = true;
            line = &line[1..];
        }
        // The front of the line must be a hash of the usual length, followed by
        // two spaces. The hex characters in the hash must be lowercase for now,
        // though we could support uppercase too if we wanted to.
        let hash_hex_len = 2 * blake3::OUT_LEN;
        let num_spaces = 2;
        let prefix_len = hash_hex_len + num_spaces;
        ensure!(line.len() > prefix_len, "Short line");
        ensure!(
            line.chars().take(prefix_len).all(|c| c.is_ascii()),
            "Non-ASCII prefix"
        );
        ensure!(&line[hash_hex_len..][..2] == "  ", "Invalid space");
        // Decode the hash hex.
        let mut hash_bytes = [0; blake3::OUT_LEN];
        let mut hex_chars = line[..hash_hex_len].chars();
        for byte in &mut hash_bytes {
            let high_char = hex_chars.next().unwrap();
            let low_char = hex_chars.next().unwrap();
            *byte = 16 * hex_half_byte(high_char)? + hex_half_byte(low_char)?;
        }
        let expected_hash: blake3::Hash = hash_bytes.into();
        let file_string = line[prefix_len..].to_string();
        let file_path_string = if is_escaped {
            // If we detected a backslash at the start of the line earlier, now we
            // need to unescape backslashes and newlines.
            unescape(&file_string)?
        } else {
            file_string.clone().into()
        };
        check_for_invalid_characters(&file_path_string)?;
        Ok(CheckLine {
            file_string,
            is_escaped,
            file_path: file_path_string.into(),
            expected_hash,
        })
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
    bail!("Invalid hex");
}

// The `check` command is a security tool. That means it's much better for a
// check to fail more often than it should (a false negative), than for a check
// to ever succeed when it shouldn't (a false positive). By forbidding certain
// characters in checked filepaths, we avoid a class of false positives where
// two different filepaths can get confused with each other.
fn check_for_invalid_characters(utf8_path: &str) -> Result<()> {
    // Null characters in paths should never happen, but they can result in a
    // path getting silently truncated on Unix.
    if utf8_path.contains('\0') {
        bail!("Null character in path");
    }
    // Because we convert invalid UTF-8 sequences in paths to the Unicode
    // replacement character, multiple different invalid paths can map to the
    // same UTF-8 string.
    if utf8_path.contains('�') {
        bail!("Unicode replacement character in path");
    }
    // We normalize all Windows backslashes to forward slashes in our output,
    // so the only natural way to get a backslash in a checkfile on Windows is
    // to construct it on Unix and copy it over. (Or of course you could just
    // doctor it by hand.) To avoid confusing this with a directory separator,
    // we forbid backslashes entirely on Windows. Note that this check comes
    // after unescaping has been done.
    if cfg!(windows) && utf8_path.contains('\\') {
        bail!("Backslash in path");
    }
    Ok(())
}

fn unescape(mut path: &str) -> Result<String> {
    let mut unescaped = String::with_capacity(2 * path.len());
    while let Some(i) = path.find('\\') {
        ensure!(i < path.len() - 1, "Invalid backslash escape");
        unescaped.push_str(&path[..i]);
        match path[i + 1..].chars().next().unwrap() {
            // Anything other than a recognized escape sequence is an error.
            'n' => unescaped.push_str("\n"),
            '\\' => unescaped.push_str("\\"),
            _ => bail!("Invalid backslash escape"),
        }
        path = &path[i + 2..];
    }
    unescaped.push_str(path);
    Ok(unescaped)
}

/// Parse a string into a `CheckLine` and check that the computed and expected hash match.
#[inline]
pub fn check_line<S>(line: S, gen: &HasherGenerator, no_mmap: bool) -> Result<bool>
where
    S: AsRef<str>,
{
    line.as_ref()
        .parse::<CheckLine>()
        .and_then(|p| p.check(&gen, no_mmap))
}

/// Check a checkfile by checking that each line parses to a `CheckLine` and that every hash
/// matches.
///
/// This function returns `Ok(false)` if any line in the checkfile failed. To iterate over each
/// line and get each `bool` response, use the [`check_checkfile_iter`] function.
///
/// [`check_checkfile_iter`]: fn.check_checkfile_iter.html
pub fn check_checkfile<P>(path: P, gen: HasherGenerator, no_mmap: bool) -> Result<bool>
where
    P: AsRef<Path>,
{
    let checkfile_input = Input::open(path, gen.is_keyed(), no_mmap)?;
    let mut bufreader = io::BufReader::new(checkfile_input);
    let mut line = String::new();
    let mut some_file_failed = false;
    loop {
        line.clear();
        let n = bufreader.read_line(&mut line)?;
        if n == 0 {
            return Ok(some_file_failed);
        }
        if let Ok(false) | Err(_) = check_line(&line, &gen, no_mmap) {
            some_file_failed = true;
        }
    }
}

/// Iterate over each line in a checkfile and check each `CheckLine`.
pub fn check_checkfile_iter<P>(
    path: P,
    gen: HasherGenerator,
    no_mmap: bool,
) -> Result<impl Iterator<Item = Result<bool>>>
where
    P: AsRef<Path>,
{
    let checkfile_input = Input::open(path, gen.is_keyed(), no_mmap)?;
    let mut bufreader = io::BufReader::new(checkfile_input);
    let mut line = String::new();
    Ok(from_fn(move || {
        line.clear();
        match bufreader.read_line(&mut line) {
            Ok(n) if n == 0 => return None,
            Err(e) => return Some(Err(Error::new(e))),
            _ => {}
        }
        Some(check_line(&line, &gen, no_mmap))
    }))
}

/// Check a `CheckLine` and print success/failure messages to `stdout`.
///
/// The `quiet` argument suppresses success messages.
pub fn print_check_line(line: &str, gen: &HasherGenerator, no_mmap: bool, quiet: bool) -> bool {
    // Returns true for success. Having a boolean return value here, instead of
    // passing down the some_file_failed reference, makes it less likely that we
    // might forget to set it in some error condition.
    match line.parse::<CheckLine>() {
        Ok(parsed) => match parsed.check(gen, no_mmap) {
            Ok(checked) => {
                if checked {
                    if !quiet {
                        println!("{}: OK", parsed.file_string());
                    }
                    return true;
                } else {
                    println!("{}: FAILED", parsed.file_string());
                }
            }
            Err(e) => println!("{}: FAILED ({})", parsed.file_string(), e),
        },
        Err(e) => eprintln!("{}: {}", NAME, e),
    }
    false
}

/// Check a checkfile and print success/failure messages to `stdout`.
///
/// Returns `Ok(false)` if any line fails to check with success. Related: [`print_check_line`].
///
/// [`print_check_line`]: fn.print_check_line.html
pub fn print_check_checkfile<P>(
    path: P,
    gen: &HasherGenerator,
    no_mmap: bool,
    quiet: bool,
) -> Result<bool>
where
    P: AsRef<Path>,
{
    let checkfile_input = Input::open(path, gen.is_keyed(), no_mmap)?;
    let mut bufreader = io::BufReader::new(checkfile_input);
    let mut line = String::new();
    let mut some_file_failed = false;
    loop {
        line.clear();
        let n = bufreader.read_line(&mut line)?;
        if n == 0 {
            return Ok(some_file_failed);
        }
        // print_check_line() prints errors and turns them into a success=false
        // return, so it doesn't return a Result.
        let success = print_check_line(&line, &gen, no_mmap, quiet);
        if !success {
            some_file_failed = true;
        }
    }
}
