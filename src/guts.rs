//! This undocumented and unstable module is for use cases like the `bao` crate,
//! which need to traverse the BLAKE3 Merkle tree and work with chunk and parent
//! chaining values directly. There might be breaking changes to this module
//! between patch versions.
//!
//! We could stabilize something like this module in the future. If you have a
//! use case for it, please let us know by filing a GitHub issue.

use crate::platform::Platform;
use crate::{CVBytes, CVWords, Hash, IV, KEY_LEN};

pub const BLOCK_LEN: usize = 64;
pub const CHUNK_LEN: usize = 1024;

#[deprecated]
#[derive(Clone, Debug)]
pub struct ChunkState(crate::ChunkState);

#[allow(deprecated)]
impl ChunkState {
    // Currently this type only supports the regular hash mode. If an
    // incremental user needs keyed_hash or derive_key, we can add that.
    pub fn new(chunk_counter: u64) -> Self {
        Self(crate::ChunkState::new(
            IV,
            chunk_counter,
            0,
            Platform::detect(),
        ))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.count()
    }

    #[inline]
    pub fn update(&mut self, input: &[u8]) -> &mut Self {
        self.0.update(input);
        self
    }

    pub fn finalize(&self, is_root: bool) -> Hash {
        let output = self.0.output();
        if is_root {
            output.root_hash()
        } else {
            output.chaining_value().into()
        }
    }
}

// As above, this currently assumes the regular hash mode. If an incremental
// user needs keyed_hash or derive_key, we can add that.
#[deprecated]
pub fn parent_cv(left_child: &Hash, right_child: &Hash, is_root: bool) -> Hash {
    let output = crate::parent_node_output(
        left_child.as_bytes(),
        right_child.as_bytes(),
        IV,
        0,
        Platform::detect(),
    );
    if is_root {
        output.root_hash()
    } else {
        output.chaining_value().into()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Mode<'a> {
    Hash,
    KeyedHash(&'a [u8; KEY_LEN]),
    DeriveKeyContext,
    DeriveKeyMaterial,
}

impl<'a> Mode<'a> {
    #[inline(always)]
    fn key_words(&self) -> CVWords {
        match self {
            Mode::KeyedHash(key) => crate::platform::words_from_le_bytes_32(key),
            _ => *IV,
        }
    }

    fn flags_byte(&self) -> u8 {
        match self {
            Mode::Hash => 0,
            Mode::KeyedHash(_) => crate::KEYED_HASH,
            Mode::DeriveKeyContext => crate::DERIVE_KEY_CONTEXT,
            Mode::DeriveKeyMaterial => crate::DERIVE_KEY_MATERIAL,
        }
    }
}

// In the diagram below, the subtree that starts with chunk 2 includes chunk 3 but not chunk 4. The
// subtree that starts with chunk 4 includes chunk 7 but (if the tree was bigger) would not include
// chunk 8. For a subtree starting at chunk index N, the maximum number of chunks in the tree is
// 2 ^ (trailing zero bits of N). If you try to hash more input than this in a subtree, you'll
// merge parent nodes that should never be merged, and your output will be garbage.
//                 .
//              /    \
//          .           .
//        /   \       /   \
//       .     .     .     .
//      / \   / \   / \   / \
//     0  1  2  3  4  5  6  7
pub(crate) fn max_subtree_len(counter: u64) -> u64 {
    debug_assert_ne!(counter, 0);
    (1 << counter.trailing_zeros()) * CHUNK_LEN as u64
}

#[test]
fn test_max_subtree_len() {
    // (chunk index, max chunks)
    let cases = [
        (1, 1),
        (2, 2),
        (3, 1),
        (4, 4),
        (5, 1),
        (6, 2),
        (7, 1),
        (8, 8),
    ];
    for (counter, chunks) in cases {
        assert_eq!(max_subtree_len(counter), chunks * CHUNK_LEN as u64);
    }
}

fn hash_subtree_inner<J: crate::join::Join>(input: &[u8], offset: u64, mode: Mode) -> CVBytes {
    debug_assert!(input.len() != 0, "empty subtrees are never valid");
    debug_assert_eq!(
        offset % CHUNK_LEN as u64,
        0,
        "offset ({offset}) must be a chunk boundary (divisible by {CHUNK_LEN})",
    );
    let counter = offset / CHUNK_LEN as u64;
    if counter != 0 {
        let max = max_subtree_len(counter);
        debug_assert!(
            input.len() as u64 <= max,
            "the subtree starting at {offset} contains at most {max} bytes (found {})",
            input.len(),
        );
    }
    crate::hash_all_at_once::<J>(input, &mode.key_words(), 0, 0).chaining_value()
}

/// This always returns a non-root chaining value. It's never correct to cast this function's
/// return value to Hash. If offset is 0, there must be more input to merge.
pub fn hash_subtree(input: &[u8], offset: u64, mode: Mode) -> CVBytes {
    hash_subtree_inner::<crate::join::SerialJoin>(input, offset, mode)
}

/// This always returns a non-root chaining value. It's never correct to cast this function's
/// return value to Hash. If offset is 0, there must be more input to merge.
#[cfg(feature = "rayon")]
pub fn hash_subtree_rayon(input: &[u8], offset: u64, mode: Mode) -> CVBytes {
    hash_subtree_inner::<crate::join::RayonJoin>(input, offset, mode)
}

fn merge_subtrees_inner(left_hash: &CVBytes, right_hash: &CVBytes, mode: Mode) -> crate::Output {
    crate::parent_node_output(
        left_hash,
        right_hash,
        &mode.key_words(),
        mode.flags_byte(),
        Platform::detect(),
    )
}

/// Compute a non-root chaining value. It's never correct to cast this function's return value to
/// Hash.
pub fn merge_subtrees_non_root(left_hash: &CVBytes, right_hash: &CVBytes, mode: Mode) -> CVBytes {
    merge_subtrees_inner(left_hash, right_hash, mode).chaining_value()
}

/// Compute the root hash, similar to [`Hasher::finalize`](crate::Hasher::finalize).
pub fn merge_subtrees_root(left_hash: &CVBytes, right_hash: &CVBytes, mode: Mode) -> crate::Hash {
    merge_subtrees_inner(left_hash, right_hash, mode).root_hash()
}

/// Return a root [`OutputReader`](crate::OutputReader), similar to
/// [`Hasher::finalize_xof`](crate::Hasher::finalize_xof).
pub fn merge_subtrees_xof(
    left_hash: &CVBytes,
    right_hash: &CVBytes,
    mode: Mode,
) -> crate::OutputReader {
    crate::OutputReader::new(merge_subtrees_inner(left_hash, right_hash, mode))
}

pub fn set_input_offset(hasher: &mut crate::Hasher, offset: u64) {
    debug_assert_eq!(hasher.count(), 0, "hasher has already accepted input");
    debug_assert_eq!(
        offset % CHUNK_LEN as u64,
        0,
        "offset ({offset}) must be a chunk boundary (divisible by {CHUNK_LEN})",
    );
    let counter = offset / CHUNK_LEN as u64;
    hasher.chunk_state.chunk_counter = counter;
    hasher.initial_chunk_counter = counter;
}

pub fn finalize_non_root(hasher: &crate::Hasher) -> CVBytes {
    assert_ne!(hasher.count(), 0, "empty subtrees are never valid");
    hasher.final_output().chaining_value()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_chunk() {
        assert_eq!(
            crate::hash(b"foo"),
            ChunkState::new(0).update(b"foo").finalize(true)
        );
    }

    #[test]
    #[allow(deprecated)]
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
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_empty_subtree_should_panic() {
        hash_subtree(b"", 0, Mode::Hash);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_hasher_empty_subtree_should_panic() {
        _ = finalize_non_root(&crate::Hasher::new());
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_unaligned_offset_should_panic() {
        hash_subtree(b"x", 1, Mode::Hash);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_hasher_unaligned_offset_should_panic() {
        let mut hasher = crate::Hasher::new();
        set_input_offset(&mut hasher, 1);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_too_much_input_should_panic() {
        hash_subtree(&[0; CHUNK_LEN + 1], CHUNK_LEN as u64, Mode::Hash);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_hasher_too_much_input_should_panic() {
        let mut hasher = crate::Hasher::new();
        set_input_offset(&mut hasher, CHUNK_LEN as u64);
        hasher.update(&[0; CHUNK_LEN + 1]);
    }
}
