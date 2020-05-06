//! Semi-public, semi-stable APIs for "peeking under the hood."
//!
//! This module is hidden from the docs, and the vast majority of callers
//! should not use it. These APIs are experimental, and much more likely to
//! change than the rest of the crate.
//!
//! This moduls supports manipulating subtree chaining values directly, and
//! serializing `Hasher` state to external storage. All of these tools have the
//! potential to give you incorrect finalized hashes if they're misused, which
//! creates very tricky bugs and violates all sorts of security invariants. If
//! you're thinking about using these, consider [filing a GitHub
//! issue](https://github.com/BLAKE3-team/BLAKE3/issues/new) to discuss your
//! use case. We'd love to hear about it.

use crate::platform::{le_bytes_from_words_32, words_from_le_bytes_32};
use arrayvec::ArrayVec;

#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};

/// An incremental state like `Hasher`, for computing possibly-non-root
/// chaining values of single chunks in the interior of the tree.
///
/// This currently supports only the default hashing mode. If an incremental
/// user needs to expose `keyed_hash` or `derive_key`, we can add that.
///
/// NOTE: This type is probably going to be replaced. See [this
/// discussion](https://github.com/BLAKE3-team/BLAKE3/issues/82).
#[derive(Clone, Debug)]
pub struct ChunkState(crate::ChunkState);

impl ChunkState {
    // Currently this type only supports the regular hash mode. If an
    // incremental user needs keyed_hash or derive_key, we can add that.
    pub fn new(chunk_counter: u64) -> Self {
        Self(crate::ChunkState::new(
            crate::IV,
            chunk_counter,
            0,
            crate::platform::Platform::detect(),
        ))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn update(&mut self, input: &[u8]) -> &mut Self {
        self.0.update(input);
        self
    }

    pub fn finalize(&self, is_root: bool) -> crate::Hash {
        let output = self.0.output();
        if is_root {
            output.root_hash()
        } else {
            output.chaining_value().into()
        }
    }
}

/// Similar to `ChunkState`, but for parent chaining values.
///
/// This currently supports only the default hashing mode. If an incremental
/// user needs to expose `keyed_hash` or `derive_key`, we can add that.
pub fn parent_cv(
    left_child: &crate::Hash,
    right_child: &crate::Hash,
    is_root: bool,
) -> crate::Hash {
    let output = crate::parent_node_output(
        left_child.as_bytes(),
        right_child.as_bytes(),
        crate::IV,
        0,
        crate::platform::Platform::detect(),
    );
    if is_root {
        output.root_hash()
    } else {
        output.chaining_value().into()
    }
}

/// A complete copy of the internal state of a `Hasher`, for serialization
/// purposes. This struct can contain buffered input bytes, and if you use a
/// secret key, a copy of that key. Is is very security sensitive.
#[derive(Clone)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct ExportedHasher {
    pub key: [u8; 32],
    pub cv_stack: ArrayVec<[[u8; 32]; crate::MAX_DEPTH + 1]>,
    pub chunk_cv: [u8; 32],
    pub chunk_counter: u64,
    pub chunk_buf: ArrayVec<[u8; 64]>,
    pub blocks_compressed: u8,
    pub flags: u8,
}

/// Copy all the internal state of a `Hasher` to construct an `ExportedHasher`.
/// This is very security sensitive.
pub fn export_hasher(hasher: &crate::Hasher) -> ExportedHasher {
    let mut chunk_buf = ArrayVec::new();
    chunk_buf
        .try_extend_from_slice(&hasher.chunk_state.buf[..hasher.chunk_state.buf_len as usize])
        .unwrap();
    ExportedHasher {
        key: le_bytes_from_words_32(&hasher.key),
        cv_stack: hasher.cv_stack.clone(),
        chunk_cv: le_bytes_from_words_32(&hasher.chunk_state.cv),
        chunk_counter: hasher.chunk_state.chunk_counter,
        chunk_buf,
        blocks_compressed: hasher.chunk_state.blocks_compressed,
        flags: hasher.chunk_state.flags,
    }
}

/// Reconstitute a `Hasher` from an `ExportedHasher`. The `ExportedHasher` must
/// come from a call to `export_hasher`, and its contents must not be modified
/// in any way. The input to this `unsafe` function is fully trusted. If a bug
/// or an attacker corrupts the input, it might trigger UB. If it doesn't
/// trigger UB today, this crate might be modified such that it triggers UB in
/// the future.
pub unsafe fn import_hasher(exported: &ExportedHasher) -> crate::Hasher {
    // We don't assert everything here, but these are some essentials.
    assert_eq!(
        exported.chunk_counter.count_ones() as usize,
        exported.cv_stack.len(),
        "wrong number of entries in the CV stack",
    );
    let allowed_flags = crate::KEYED_HASH | crate::DERIVE_KEY_MATERIAL;
    assert_eq!(0, exported.flags & !allowed_flags, "unexpected flags");

    let mut chunk_buf = [0; crate::BLOCK_LEN];
    chunk_buf[..exported.chunk_buf.len()].copy_from_slice(&exported.chunk_buf);
    let chunk_state = crate::ChunkState {
        cv: words_from_le_bytes_32(&exported.chunk_cv),
        chunk_counter: exported.chunk_counter,
        buf: chunk_buf,
        buf_len: exported.chunk_buf.len() as u8,
        blocks_compressed: exported.blocks_compressed,
        flags: exported.flags,
        platform: crate::platform::Platform::detect(),
    };
    crate::Hasher {
        key: words_from_le_bytes_32(&exported.key),
        chunk_state,
        cv_stack: exported.cv_stack.clone(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk() {
        assert_eq!(
            crate::hash(b"foo"),
            ChunkState::new(0).update(b"foo").finalize(true)
        );
    }

    #[test]
    fn test_parents() {
        let mut hasher = crate::Hasher::new();
        let mut buf = [0; crate::CHUNK_LEN];

        buf[0] = 'a' as u8;
        hasher.update(&buf);
        let chunk0_cv = ChunkState::new(0).update(&buf).finalize(false);

        buf[0] = 'b' as u8;
        hasher.update(&buf);
        let chunk1_cv = ChunkState::new(1).update(&buf).finalize(false);

        hasher.update(b"c");
        let chunk2_cv = ChunkState::new(2).update(b"c").finalize(false);

        let parent = parent_cv(&chunk0_cv, &chunk1_cv, false);
        let root = parent_cv(&parent, &chunk2_cv, true);
        assert_eq!(hasher.finalize(), root);
    }

    #[test]
    fn test_export_and_import_hasher() {
        let mut input = [0; 23456]; // results in a non-zero block counter
        crate::test::paint_test_input(&mut input);
        let mut test_key = [0; 32];
        test_key.copy_from_slice(&input[1000..1032]);
        let mut hasher = crate::Hasher::new_keyed(&test_key);
        hasher.update(&input);

        let exported = export_hasher(&hasher);
        let mut imported = unsafe { import_hasher(&exported) };

        hasher.update(&input);
        imported.update(&input);
        assert_eq!(hasher.finalize(), imported.finalize());
    }

    #[test]
    #[cfg(feature = "serde_derive")]
    fn test_serialize_hasher() {
        let mut input = [0; 23456]; // results in a non-zero block counter
        crate::test::paint_test_input(&mut input);
        let test_context = "BLAKE3 2020-05-06 12:21:23 test serde";
        let mut hasher = crate::Hasher::new_derive_key(test_context);
        hasher.update(&input);

        let exported = export_hasher(&hasher);
        let serialized = serde_json::to_string_pretty(&exported).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        let mut imported = unsafe { import_hasher(&deserialized) };

        hasher.update(&input);
        imported.update(&input);
        assert_eq!(hasher.finalize(), imported.finalize());
    }

    #[test]
    #[should_panic]
    fn test_bad_chunk_counter_panics() {
        let mut exported = export_hasher(&crate::Hasher::new());
        // The chunk_counter needs to correspond to the number of chaining
        // values in the CV stack.
        exported.chunk_counter += 1;
        unsafe {
            import_hasher(&exported);
        }
    }

    #[test]
    #[should_panic]
    fn test_bad_flags_panic() {
        let mut exported = export_hasher(&crate::Hasher::new());
        // It shouldn't be possible to set random bitflags.
        exported.flags |= 1 << 7;
        unsafe {
            import_hasher(&exported);
        }
    }
}
