//! The file-related utilities.
//!
//! # Examples
//!
//! ```no_run
//! use std::io;
//!
//! use blake3::file::hash_path_maybe_mmap;
//!
//! fn main() -> io::Result<()> {
//!     let args: Vec<_> = std::env::args_os().collect();
//!     assert_eq!(args.len(), 2);
//!     let path = &args[1];
//!     let mut hasher = blake3::Hasher::new();
//!     hash_path_maybe_mmap(&mut hasher, path)?;
//!     println!("{}", hasher.finalize());
//!     Ok(())
//! }
//! ```

use std::{fs::File, io, path::Path};

/// Mmap a file, if it looks like a good idea. Return None in cases where we
/// know mmap will fail, or if the file is short enough that mmapping isn't
/// worth it. However, if we do try to mmap and it fails, return the error.
pub fn maybe_memmap_file(file: &File) -> io::Result<Option<memmap2::Mmap>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    #[allow(clippy::if_same_then_else)]
    if !metadata.is_file() {
        // Not a real file.
        Ok(None)
    } else if file_size > isize::max_value() as u64 {
        // Too long to safely map.
        // https://github.com/danburkert/memmap-rs/issues/69
        Ok(None)
    } else if file_size == 0 {
        // Mapping an empty file currently fails.
        // https://github.com/danburkert/memmap-rs/issues/72
        Ok(None)
    } else if file_size < 16 * 1024 {
        // Mapping small files is not worth it.
        Ok(None)
    } else {
        // Explicitly set the length of the memory map, so that filesystem
        // changes can't race to violate the invariants we just checked.
        let map = unsafe {
            memmap2::MmapOptions::new()
                .len(file_size as usize)
                .map(file)?
        };
        Ok(Some(map))
    }
}

/// Hash a file fast.
///
/// It may use mmap if the file is big enough. If not, it will read the whole file into a buffer.
pub fn hash_path_maybe_mmap(hasher: &mut crate::Hasher, path: impl AsRef<Path>) -> io::Result<()> {
    let file = File::open(path.as_ref())?;
    if let Some(mmap) = maybe_memmap_file(&file)? {
        hasher.update_rayon(&mmap);
    } else {
        crate::copy_wide(&file, hasher)?;
    }
    Ok(())
}
