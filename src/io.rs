//! Helper functions for efficient IO.

#[cfg(feature = "mmap")]
use std::fs::File;
use std::io;

#[cfg(feature = "mmap")]
const MINIMUM_MMAP_SIZE: u64 = 16 * 1024; // 16 KiB

pub(crate) fn copy_wide(mut reader: impl io::Read, hasher: &mut crate::Hasher) -> io::Result<u64> {
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            // see test_update_reader_interrupted
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

// Try to mmap a file, if it looks like a good idea. Return None if mmap fails, or if the file is
// short enough that it's not worth it.
//
// SAFETY: Mmaps are fundamentally unsafe, because you can call invariant-checking functions like
// str::from_utf8 on them and then have them change out from under you. Letting a safe caller get
// their hands on an mmap, or even a &[u8] that's backed by an mmap, is unsound. However, because
// this function is crate-private, we can guarantee that all can ever happen in the event of a race
// condition is that we either hash nonsense bytes or crash with SIGBUS or similar, neither of
// which should risk memory corruption in a safe caller.
//
// PARANOIA: But a data race...is a data race...is a data race...right? Even if we know that no
// platform in the "real world" is ever going to do anything other than compute the "wrong answer"
// if we race on this mmap while we hash it, aren't we still supposed to feel bad about doing this?
// Well, maybe. This is IO, and IO gets special carve-outs in the memory model. Consider a
// memory-mapped register that returns random 32-bit words. (This is actually realistic if you have
// a hardware RNG.) It's probably sound to construct a *const i32 pointing to that register and do
// some raw pointer reads from it. Those reads should be volatile if you don't want the compiler to
// coalesce them, but either way the compiler isn't allowed to just _go nuts_ and insert
// should-never-happen branches to wipe your hard drive if two adjacent reads happen to give
// different values. As far as I'm aware, there's no such thing as a read that's allowed if it's
// volatile but prohibited if it's not (unlike atomics). As mentioned above, it's not ok to
// construct a safe &i32 to the register if you're going to leak that reference to unknown callers.
// But if you "know what you're doing," I don't think *const i32 and &i32 are fundamentally
// different here. Feedback needed.
#[cfg(feature = "mmap")]
pub(crate) fn maybe_mmap_file(file: &File) -> io::Result<Option<memmap2::Mmap>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    if !metadata.is_file() {
        // Not a real file.
        Ok(None)
    } else if file_size < MINIMUM_MMAP_SIZE {
        // Mapping small files is not worth it, and some special files that can't be mapped report
        // a size of zero.
        Ok(None)
    } else {
        // If the mmap itself fails (as opposed to opening the File previously, or reading its
        // metadata above), swallow the error and return Ok(None).
        Ok(unsafe { memmap2::Mmap::map(file) }.ok())
    }
}

#[cfg(all(test, feature = "mmap"))]
mod test {
    use super::*;
    use std::io;
    use std::io::prelude::*;

    #[test]
    fn test_maybe_mmap_current_exe() -> io::Result<()> {
        // The current executable should always be a regular file larger than 16 KiB, so mmap
        // should ~always succeed. (A filesystem might not support mmap at all, but we don't test
        // any of those in CI.)
        let exe_file = File::open(std::env::current_exe()?)?;
        assert!(exe_file.metadata()?.len() > MINIMUM_MMAP_SIZE);
        let mmap = maybe_mmap_file(&exe_file)?.expect("maybe_mmap_file should return Some");
        // Mainly we're testing that we got `Some` above, but go ahead and read the mmap just to
        // make sure it doesn't bus fault or anything like that.
        assert_eq!(
            crate::hash(&mmap),
            crate::Hasher::new().update_reader(&exe_file)?.finalize(),
        );
        Ok(())
    }

    #[test]
    fn test_maybe_mmap_small_file() -> io::Result<()> {
        // Create a file smaller than 16 KiB. `maybe_mmap_file` returns `None` because of its size.
        let mut f = tempfile::NamedTempFile::new()?;
        f.write_all(b"hello world")?;
        f.flush()?;
        assert!(maybe_mmap_file(&File::open(f.path())?)?.is_none());
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_unmappable_linux() -> io::Result<()> {
        // I'm not aware of any similarly unmappable paths on macOS or Windows, so this test is
        // Linux-only for now.
        let unmappable_path = "/sys/kernel/btf/vmlinux";
        let mut unmappable_file = File::open(unmappable_path)?;
        // The file is large enough to attempt mmapping.
        assert!(unmappable_file.metadata()?.len() > MINIMUM_MMAP_SIZE);
        // We're allowed to read the file.
        assert_eq!(unmappable_file.read(&mut [0])?, 1);
        // But mmapping the file fails.
        unsafe { memmap2::Mmap::map(&unmappable_file) }.unwrap_err();
        // `maybe_mmap_file` swallows that error and returns `None`.
        assert!(maybe_mmap_file(&unmappable_file)?.is_none());
        Ok(())
    }
}
