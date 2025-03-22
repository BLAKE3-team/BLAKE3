//! This undocumented and unstable module is for use cases like the `bao` crate,
//! which need to traverse the BLAKE3 Merkle tree and work with chunk and parent
//! chaining values directly. There might be breaking changes to this module
//! between patch versions.
//!
//! We could stabilize something like this module in the future. If you have a
//! use case for it, please let us know by filing a GitHub issue.

use crate::platform::Platform;
use crate::{CVWords, Hash, Hasher, IV, KEY_LEN, OUT_LEN};

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
    DeriveKeyMaterial(&'a [u8; KEY_LEN]),
}

impl<'a> Mode<'a> {
    #[inline(always)]
    fn key_words(&self) -> CVWords {
        match self {
            Mode::Hash => *IV,
            Mode::KeyedHash(key) => crate::platform::words_from_le_bytes_32(key),
            Mode::DeriveKeyMaterial(cx_key) => crate::platform::words_from_le_bytes_32(cx_key),
        }
    }

    fn flags_byte(&self) -> u8 {
        match self {
            Mode::Hash => 0,
            Mode::KeyedHash(_) => crate::KEYED_HASH,
            Mode::DeriveKeyMaterial(_) => crate::DERIVE_KEY_MATERIAL,
        }
    }
}

// In the diagram below, the subtree that starts with chunk 2 includes chunk 3 but not chunk 4. The
// subtree that starts with chunk 4 includes chunk 7 but (if the tree was bigger) would not include
// chunk 8. For a subtree starting at chunk index N, the maximum number of chunks in the tree is
// 2 ^ (trailing zero bits of N). If you try to hash more input than this in a subtree, you'll
// merge parent nodes that should never be merged, and your output will be garbage.
//                .
//            /       \
//          .           .
//        /   \       /   \
//       .     .     .     .
//      / \   / \   / \   / \
//     0  1  2  3  4  5  6  7
pub(crate) fn max_subtree_len(counter: u64) -> u64 {
    debug_assert_ne!(counter, 0);
    (1 << counter.trailing_zeros()) * CHUNK_LEN as u64
}

/// The academic term for a "non-root hash" is a "chaining value".
#[derive(Copy, Clone, Debug, Eq)]
pub struct NonRootHash(pub [u8; OUT_LEN]);

impl PartialEq for NonRootHash {
    #[inline]
    fn eq(&self, other: &NonRootHash) -> bool {
        constant_time_eq::constant_time_eq_32(&self.0, &other.0)
    }
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

fn merge_subtrees_inner(
    left_child: &NonRootHash,
    right_child: &NonRootHash,
    mode: Mode,
) -> crate::Output {
    crate::parent_node_output(
        &left_child.0,
        &right_child.0,
        &mode.key_words(),
        mode.flags_byte(),
        Platform::detect(),
    )
}

/// Compute a non-root chaining value. It's never correct to cast this function's return value to
/// Hash.
pub fn merge_subtrees_non_root(
    left_child: &NonRootHash,
    right_child: &NonRootHash,
    mode: Mode,
) -> NonRootHash {
    NonRootHash(merge_subtrees_inner(left_child, right_child, mode).chaining_value())
}

/// Compute the root hash, similar to [`Hasher::finalize`](crate::Hasher::finalize).
pub fn merge_subtrees_root(
    left_child: &NonRootHash,
    right_child: &NonRootHash,
    mode: Mode,
) -> crate::Hash {
    merge_subtrees_inner(left_child, right_child, mode).root_hash()
}

/// Return a root [`OutputReader`](crate::OutputReader), similar to
/// [`Hasher::finalize_xof`](crate::Hasher::finalize_xof).
pub fn merge_subtrees_xof(
    left_child: &NonRootHash,
    right_child: &NonRootHash,
    mode: Mode,
) -> crate::OutputReader {
    crate::OutputReader::new(merge_subtrees_inner(left_child, right_child, mode))
}

pub fn context_key(context: &str) -> [u8; crate::KEY_LEN] {
    crate::hash_all_at_once::<crate::join::SerialJoin>(
        context.as_bytes(),
        IV,
        crate::DERIVE_KEY_CONTEXT,
        0,
    )
    .root_hash()
    .0
}

pub trait HasherGutsExt {
    fn new_from_context_key(context_key: &[u8; KEY_LEN]) -> Self;
    fn set_input_offset(&mut self, offset: u64) -> &mut Self;
    fn finalize_non_root(&self) -> NonRootHash;
}

impl HasherGutsExt for Hasher {
    fn new_from_context_key(context_key: &[u8; KEY_LEN]) -> Hasher {
        let context_key_words = crate::platform::words_from_le_bytes_32(context_key);
        Hasher::new_internal(&context_key_words, crate::DERIVE_KEY_MATERIAL)
    }

    fn set_input_offset(&mut self, offset: u64) -> &mut Hasher {
        debug_assert_eq!(self.count(), 0, "hasher has already accepted input");
        debug_assert_eq!(
            offset % CHUNK_LEN as u64,
            0,
            "offset ({offset}) must be a chunk boundary (divisible by {CHUNK_LEN})",
        );
        let counter = offset / CHUNK_LEN as u64;
        self.chunk_state.chunk_counter = counter;
        self.initial_chunk_counter = counter;
        self
    }

    fn finalize_non_root(&self) -> NonRootHash {
        assert_ne!(self.count(), 0, "empty subtrees are never valid");
        NonRootHash(self.final_output().chaining_value())
    }
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
        let mut hasher = Hasher::new();
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
        Hasher::new().finalize_non_root();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_unaligned_offset_should_panic() {
        Hasher::new().set_input_offset(1);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_hasher_already_accepted_input_should_panic() {
        Hasher::new().update(b"x").set_input_offset(0);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_too_much_input_should_panic() {
        Hasher::new()
            .set_input_offset(CHUNK_LEN as u64)
            .update(&[0; CHUNK_LEN + 1]);
    }

    #[test]
    fn test_grouped_hash() {
        const MAX_CHUNKS: usize = (crate::test::TEST_CASES_MAX + 1) / CHUNK_LEN;
        let mut input_buf = [0; crate::test::TEST_CASES_MAX];
        crate::test::paint_test_input(&mut input_buf);
        for subtree_chunks in [1, 2, 4, 8, 16, 32] {
            #[cfg(feature = "std")]
            dbg!(subtree_chunks);
            let subtree_len = subtree_chunks * CHUNK_LEN;
            for &case in crate::test::TEST_CASES {
                if case <= subtree_len {
                    continue;
                }
                #[cfg(feature = "std")]
                dbg!(case);
                let input = &input_buf[..case];
                let expected_hash = crate::hash(input);

                // Collect all the group chaining values.
                let mut chaining_values = arrayvec::ArrayVec::<NonRootHash, MAX_CHUNKS>::new();
                let mut subtree_offset = 0;
                while subtree_offset < input.len() {
                    let take = core::cmp::min(subtree_len, input.len() - subtree_offset);
                    let subtree_input = &input[subtree_offset..][..take];
                    let subtree_cv = Hasher::new()
                        .set_input_offset(subtree_offset as u64)
                        .update(subtree_input)
                        .finalize_non_root();
                    chaining_values.push(subtree_cv);
                    subtree_offset += take;
                }

                // Compress all the chaining_values together, layer by layer.
                assert!(chaining_values.len() >= 2);
                while chaining_values.len() > 2 {
                    let n = chaining_values.len();
                    // Merge each side-by-side pair in place, overwriting the front half of the
                    // array with the merged results. This moves us "up one level" in the tree.
                    for i in 0..(n / 2) {
                        chaining_values[i] = merge_subtrees_non_root(
                            &chaining_values[2 * i],
                            &chaining_values[2 * i + 1],
                            Mode::Hash,
                        );
                    }
                    // If there's an odd CV out, it moves up.
                    if n % 2 == 1 {
                        chaining_values[n / 2] = chaining_values[n - 1];
                    }
                    chaining_values.truncate(n / 2 + n % 2);
                }
                assert_eq!(chaining_values.len(), 2);
                let root_hash =
                    merge_subtrees_root(&chaining_values[0], &chaining_values[1], Mode::Hash);
                assert_eq!(expected_hash, root_hash);
            }
        }
    }

    #[test]
    fn test_keyed_hash_xof() {
        let group0 = &[42; 4096];
        let group1 = &[43; 4095];
        let mut input = [0; 8191];
        input[..4096].copy_from_slice(group0);
        input[4096..].copy_from_slice(group1);
        let key = &[44; 32];

        let mut expected_output = [0; 100];
        Hasher::new_keyed(&key)
            .update(&input)
            .finalize_xof()
            .fill(&mut expected_output);

        let mut guts_output = [0; 100];
        let left = Hasher::new_keyed(key).update(group0).finalize_non_root();
        let right = Hasher::new_keyed(key)
            .set_input_offset(group0.len() as u64)
            .update(group1)
            .finalize_non_root();
        merge_subtrees_xof(&left, &right, Mode::KeyedHash(&key)).fill(&mut guts_output);
        assert_eq!(expected_output, guts_output);
    }

    #[test]
    fn test_derive_key() {
        let context = "foo";
        let mut input = [0; 1025];
        crate::test::paint_test_input(&mut input);
        let expected = crate::derive_key(context, &input);

        let cx_key = context_key(context);
        let left = Hasher::new_from_context_key(&cx_key)
            .update(&input[..1024])
            .finalize_non_root();
        let right = Hasher::new_from_context_key(&cx_key)
            .set_input_offset(1024)
            .update(&input[1024..])
            .finalize_non_root();
        let derived_key = merge_subtrees_root(&left, &right, Mode::DeriveKeyMaterial(&cx_key)).0;
        assert_eq!(expected, derived_key);
    }
}
