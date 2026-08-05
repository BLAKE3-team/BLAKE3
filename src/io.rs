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

// Try to `mmap` a file, unless it's short enough that ordinary reads are faster, currently 16 KiB.
// Return `Ok(None)` if mapping fails or if we don't attempt it. Only return `Err` for unexpected
// failures that could leave the `File` in a bad state.
//
// We seek to (near) end-of-file to get the length, and the prior cursor position is ignored. If
// mapping fails, we [`rewind`] the file cursor to start-of-file before returning `None`. We don't
// `rewind` if the mapping succeeds, however, to avoid unnecessary syscalls. In the unlikely event
// that the caller wants to do ordinary reads in addition to mapped reads, they need to `rewind`
// explicitly in that case. (This function isn't public, but that comes up in test cases.)
//
// [`rewind`]: https://doc.rust-lang.org/std/io/trait.Seek.html#method.rewind
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
pub(crate) fn maybe_mmap_file(file: &mut File) -> io::Result<Option<memmap2::Mmap>> {
    // Seeking is more reliable than `.metadata()` for getting the length. Either is valid for
    // regular files, but block devices sometimes report zero length despite being mappable. See
    // https://github.com/BLAKE3-team/BLAKE3/pull/487.
    use io::Seek;
    // In debug mode only, check that the cursor is at the beginning. If the seek below fails,
    // we'll leave the cursor where it is, and that can be confusing if it isn't at the beginning.
    // (This function isn't public, but this comes up in test cases.)
    if cfg!(debug_assertions) {
        if let Ok(position) = file.stream_position() {
            assert_eq!(position, 0, "initial file offset isn't at the beginning");
        }
    }
    // Seek to "16 KiB less 1 byte" from end-of-file. For short files, this will generally fail
    // with an invalid argument error and leave the cursor at the beginning. For a regular file of
    // exactly 16383 bytes, it will succeed and return 0, again leaving the cursor at the
    // beginning. For some special/device files like /dev/random, it will also return 0. In all
    // those cases, we can return without doing any extra syscalls to reset the cursor, and let the
    // caller fall back to ordinary reads.
    let seek_offset = MINIMUM_MMAP_SIZE - 1;
    let seek_target = io::SeekFrom::End(-(seek_offset as i64));
    let Ok(offset_len) = file.seek(seek_target) else {
        return Ok(None); // short or unseekable files
    };
    if offset_len == 0 {
        return Ok(None); // either exactly 16383 bytes, or e.g. /dev/random
    }
    // It's UB to produce a slice longer than `isize::MAX`. `memmap2` checks that internally, so
    // it's kind of redundant to check it here, but we need to guard the `usize` cast in any case.
    if offset_len <= isize::MAX as u64 - seek_offset {
        // We have a seekable file that's long enough to be worth mapping, but not too long to map
        // (e.g. on a 32-bit system). Try to map it. If this succeeds, we'll assume the caller
        // isn't going to do any ordinary reads, so we don't need to `rewind`.
        let mut mmap_options = memmap2::MmapOptions::new();
        mmap_options.len((offset_len + seek_offset) as usize); // checked above
        if let Ok(mmap) = unsafe { mmap_options.map(&*file) } {
            return Ok(Some(mmap));
        }
    }
    // `mmap` failed (or would've failed), so we need to `rewind` and let the caller do ordinary
    // reads. This is the only failure case where we return `Err`, since if it does fail somehow
    // (not clear if that's possible in practice), the file cursor position will be wrong.
    file.rewind()?;
    Ok(None)
}

#[cfg(all(test, feature = "mmap"))]
mod test {
    use super::*;
    use std::io;
    use std::io::prelude::*;

    #[test]
    fn test_maybe_mmap_small_files() -> io::Result<()> {
        let test_cases = [0, 1, MINIMUM_MMAP_SIZE - 2, MINIMUM_MMAP_SIZE - 1];
        for len in test_cases {
            dbg!(len);
            // Create a file smaller than 16 KiB. `maybe_mmap_file` should return `Ok(None)`.
            let mut input = vec![0; len as usize];
            crate::test::paint_test_input(&mut input);
            let mut f = tempfile::NamedTempFile::new()?;
            f.write_all(&input)?;
            f.flush()?;
            // We have a debug assert that the initial file offset is 0.
            f.rewind()?;
            assert!(maybe_mmap_file(f.as_file_mut())?.is_none());
            // Check that the file cursor remains at start-of-file.
            assert_eq!(f.stream_position()?, 0);
            assert_eq!(
                crate::hash(&input),
                crate::Hasher::new()
                    .update_reader(f.as_file_mut())?
                    .finalize(),
            );
        }
        Ok(())
    }

    #[test]
    fn test_maybe_mmap_mappable_files() -> io::Result<()> {
        let test_cases = [MINIMUM_MMAP_SIZE, MINIMUM_MMAP_SIZE + 1];
        for len in test_cases {
            dbg!(len);
            // Create a file that's 16 KiB or larger. `maybe_mmap_file` should return `Ok(Some(_))`.
            let mut input = vec![0; len as usize];
            crate::test::paint_test_input(&mut input);
            let mut f = tempfile::NamedTempFile::new()?;
            f.write_all(&input)?;
            f.flush()?;
            // We have a debug assert that the initial file offset is 0.
            f.rewind()?;
            let Ok(Some(mmap)) = maybe_mmap_file(f.as_file_mut()) else {
                panic!("mmap failed");
            };
            // `maybe_mmap_file` doesn't `rewind` the file in this case, so we need to do that
            // ourselves.
            assert_ne!(f.stream_position()?, 0);
            f.rewind()?;
            assert_eq!(mmap[..], input[..]);
            assert_eq!(crate::hash(&input), crate::hash(&mmap));
            assert_eq!(
                crate::hash(&input),
                crate::Hasher::new()
                    .update_reader(f.as_file_mut())?
                    .finalize(),
            );
        }
        Ok(())
    }

    #[test]
    fn test_maybe_mmap_current_exe() -> io::Result<()> {
        // The current executable should always be a regular file larger than 16 KiB, so mmap
        // should ~always succeed. (A filesystem might not support mmap at all, but we don't test
        // any of those in CI.)
        let mut exe_file = File::open(std::env::current_exe()?)?;
        assert!(exe_file.metadata()?.len() > MINIMUM_MMAP_SIZE);
        let mmap = maybe_mmap_file(&mut exe_file)?.expect("maybe_mmap_file should return Some");
        // Mainly we're testing that we got `Some` above, but go ahead and read the mmap just to
        // make sure it doesn't bus fault or anything like that. `maybe_mmap_file` doesn't `rewind`
        // the file in this case, so we need to do that ourselves.
        exe_file.rewind()?;
        assert_eq!(
            crate::hash(&mmap),
            crate::Hasher::new().update_reader(&exe_file)?.finalize(),
        );
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
        // But mmapping the file fails. (We have a debug assert that requires `rewind` here.)
        unmappable_file.rewind()?;
        unsafe { memmap2::Mmap::map(&unmappable_file) }.unwrap_err();
        // `maybe_mmap_file` swallows that error and returns `None`.
        assert!(maybe_mmap_file(&mut unmappable_file)?.is_none());
        Ok(())
    }
}
