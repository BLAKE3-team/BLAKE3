//! Low-level tree manipulations and other sharp tools
//!
//! <div class="warning">
//!
//! **Warning:** This whole module is *hazardous material*. If you've heard folks say *don't roll
//! your own crypto,* this is the sort of thing they were talking about. The rules for using these
//! functions correctly are complicated, and tiny mistakes can give you garbage output and/or break
//! the security properties that BLAKE3 is supposed to have. Read [the BLAKE3
//! paper](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf), particularly
//! sections 2.1 and 5.1, to understand the tree structure that you need to maintain. Test your
//! code against [`blake3::hash`](../fn.hash.html) and friends and make sure you can get the same
//! output for [lots of different
//! inputs](https://github.com/BLAKE3-team/BLAKE3/blob/master/test_vectors/test_vectors.json).
//!
//! **Encouragement:** Playing with these functions is a great way to learn how BLAKE3 works on the
//! inside. Have fun!
//!
//! </div>
//!
//! The main entrypoint for this module is the [`HasherExt`] trait, particularly the
//! [`set_input_offset`](HasherExt::set_input_offset) and
//! [`finalize_non_root`](HasherExt::finalize_non_root) methods. These let you compute the non-root
//! hashes ("chaining values") of individual chunks
//!
//! # Examples
//!
//! Here's an example of computing all the interior hashes in a 3-chunk tree:
//!
//! ```text
//!            root
//!          /      \
//!      parent      \
//!    /       \      \
//! chunk0  chunk1  chunk2
//! ```
//!
//! ```
//! # fn main() {
//! use blake3::Hasher;
//! use blake3::hazmat::{Mode, HasherExt, merge_subtrees_non_root, merge_subtrees_root};
//!
//! let chunk0 = [b'a'; 1024];
//! let chunk1 = [b'b'; 1024];
//! let chunk2 = [b'c'; 1024];
//!
//! // Hash all three chunks. Chunks or subtrees that don't begin at the start of the input use
//! // `set_input_offset` to say where they begin.
//! let chunk0_hash = Hasher::new().update(&chunk0).finalize_non_root();
//! let chunk1_hash = Hasher::new().set_input_offset(1024).update(&chunk1).finalize_non_root();
//! let chunk2_hash = Hasher::new().set_input_offset(2048).update(&chunk2).finalize_non_root();
//!
//! // Join the first two chunks with a non-root parent node.
//! let parent_hash = merge_subtrees_non_root(&chunk0_hash, &chunk1_hash, Mode::Hash);
//!
//! // Join that parent node and the third chunk at the root of the tree.
//! let root_hash = merge_subtrees_root(&parent_hash, &chunk2_hash, Mode::Hash);
//!
//! // Double check that we got the right answer.
//! let mut combined_input = [0; 1024 * 3];
//! combined_input[..1024].copy_from_slice(&chunk0);
//! combined_input[1024..2048].copy_from_slice(&chunk1);
//! combined_input[2048..].copy_from_slice(&chunk2);
//! assert_eq!(root_hash, blake3::hash(&combined_input));
//! # }
//! ```
//!
//! Hashing many chunks together is important for performance, because it allows the implementation
//! to use SIMD parallelism internally. ([AVX-512](https://en.wikipedia.org/wiki/AVX-512) for
//! example needs 16 chunks to really get going.) We can reproduce `parent_hash` by hashing
//! `chunk0` and `chunk1` at the same time:
//!
//! ```
//! # fn main() {
//! # use blake3::Hasher;
//! # use blake3::hazmat::{Mode, HasherExt, merge_subtrees_non_root, merge_subtrees_root};
//! # let chunk0 = [b'a'; 1024];
//! # let chunk1 = [b'b'; 1024];
//! # let chunk2 = [b'c'; 1024];
//! # let mut combined_input = [0; 1024 * 3];
//! # combined_input[..1024].copy_from_slice(&chunk0);
//! # combined_input[1024..2048].copy_from_slice(&chunk1);
//! # combined_input[2048..].copy_from_slice(&chunk2);
//! # let chunk0_hash = Hasher::new().update(&chunk0).finalize_non_root();
//! # let chunk1_hash = Hasher::new().set_input_offset(1024).update(&chunk1).finalize_non_root();
//! # let parent_hash = merge_subtrees_non_root(&chunk0_hash, &chunk1_hash, Mode::Hash);
//! let left_subtree_hash = Hasher::new().update(&combined_input[..2048]).finalize_non_root();
//! assert_eq!(left_subtree_hash, parent_hash);
//! # }
//! ```
//!
//! However, hashing multiple chunks together **must** respect the overall tree structure. Hashing
//! `chunk0` and `chunk1` together is valid, but hashing `chunk1` and `chunk2` together is
//! incorrect and gives a garbage result that will never match a standard BLAKE3 hash. The
//! implementation includes a few best-effort asserts to catch some of these mistakes, but these
//! checks aren't guaranteed. For example, this call to `update` currently panics:
//!
//! ```should_panic
//! # fn main() {
//! # use blake3::Hasher;
//! # use blake3::hazmat::HasherExt;
//! # let chunk0 = [b'a'; 1024];
//! # let chunk1 = [b'b'; 1024];
//! # let chunk2 = [b'c'; 1024];
//! # let mut combined_input = [0; 1024 * 3];
//! # combined_input[..1024].copy_from_slice(&chunk0);
//! # combined_input[1024..2048].copy_from_slice(&chunk1);
//! # combined_input[2048..].copy_from_slice(&chunk2);
//! let oops = Hasher::new()
//!     .set_input_offset(1024)
//!     // PANIC: "the subtree starting at 1024 contains at most 1024 bytes"
//!     .update(&combined_input[1024..])
//!     .finalize_non_root();
//! # }
//! ```
//!
//! For more on what makes a valid subtree, see [`max_subtree_len`] and its doc comments. Note that
//! the merging functions ([`merge_subtrees_non_root`] and friends) don't know the shape of the
//! left and right subtrees you're giving them, and they can't help you catch mistakes. The only
//! way to catch mistakes with those is to compare your root output to the
//! [`blake3::hash`](crate::hash) or similar of the same input.

use crate::platform::Platform;
use crate::{CVWords, Hasher, CHUNK_LEN, IV, KEY_LEN, OUT_LEN};

/// Extension methods for [`Hasher`]. This is the main entrypoint to the `hazmat` module.
pub trait HasherExt {
    /// Similar to [`Hasher::new_derive_key`] but using a pre-hashed [`ContextKey`] from
    /// [`hash_derive_key_context`].
    ///
    /// The [`hash_derive_key_context`] function is _only_ valid source of the [`ContextKey`]
    ///
    /// # Example
    ///
    /// ```
    /// use blake3::Hasher;
    /// use blake3::hazmat::HasherExt;
    ///
    /// let context_key = blake3::hazmat::hash_derive_key_context("foo");
    /// let mut hasher = Hasher::new_from_context_key(&context_key);
    /// hasher.update(b"bar");
    /// let derived_key = *hasher.finalize().as_bytes();
    ///
    /// assert_eq!(derived_key, blake3::derive_key("foo", b"bar"));
    /// ```
    fn new_from_context_key(context_key: &ContextKey) -> Self;

    /// Configure the `Hasher` to process a chunk or subtree starting at `offset` bytes into the
    /// whole input.
    ///
    /// This isn't required for the first chunk, or for a subtree that includes the first chunk
    /// (i.e. when the `offset` is zero), but it's required for all other chunks and subtrees.
    ///
    /// # Panics
    ///
    /// This should always be paired with [`finalize_non_root`](HasherExt::finalize_non_root). It's
    /// never correct to use a non-zero input offset with [`finalize`](Hasher::finalize) or
    /// [`finalize_xof`](Hasher::finalize_xof). The `offset` must also be a multiple of
    /// `CHUNK_LEN`. Violating either of these rules will currently fail an assertion and panic,
    /// but this is not guaranteed.
    fn set_input_offset(&mut self, offset: u64) -> &mut Self;

    /// Finalize the non-root hash ("chaining value") of the current chunk or subtree.
    ///
    /// Afterwards you can merge subtree chaining values into parent nodes using
    /// [`merge_subtrees_non_root`] and ultimately into the root node with either
    /// [`merge_subtrees_root`] (similar to [`Hasher::finalize`]) or [`merge_subtrees_xof`]
    /// (similar to [`Hasher::finalize_xof`]).
    fn finalize_non_root(&self) -> ChainingValue;
}

impl HasherExt for Hasher {
    fn new_from_context_key(context_key: &[u8; KEY_LEN]) -> Hasher {
        let context_key_words = crate::platform::words_from_le_bytes_32(context_key);
        Hasher::new_internal(&context_key_words, crate::DERIVE_KEY_MATERIAL)
    }

    fn set_input_offset(&mut self, offset: u64) -> &mut Hasher {
        assert_eq!(self.count(), 0, "hasher has already accepted input");
        assert_eq!(
            offset % CHUNK_LEN as u64,
            0,
            "offset ({offset}) must be a chunk boundary (divisible by {CHUNK_LEN})",
        );
        let counter = offset / CHUNK_LEN as u64;
        self.chunk_state.chunk_counter = counter;
        self.initial_chunk_counter = counter;
        self
    }

    fn finalize_non_root(&self) -> ChainingValue {
        assert_ne!(self.count(), 0, "empty subtrees are never valid");
        self.final_output().chaining_value()
    }
}

/// Compute the maximum length of a subtree, given its starting offset.
///
/// If you try to hash more input than this in a subtree, you'll merge parent nodes that should
/// never be merged, and your output will be garbage. [`Hasher::update`] will currently panic if
/// you try this, but this is not guaranteed.
///
/// For input offset zero (the default), there is no maximum length, and this function returns
/// `None`. For all other offsets it returns `Some`. Note that valid offset must be a multiple of
/// `CHUNK_LEN`; it's not possible to start hashing a chunk in the middle.
///
/// In the tree below, the subtree that starts with chunk 3 (input offset 3 * CHUNK_LEN) includes
/// only that one chunk, so its max length is `Some(1024)` (`CHUNK_LEN`). The subtree that starts
/// with chunk 6 includes chunk 7 but not chunk 8, so its max length is `Some(2048)` (2 *
/// `CHUNK_LEN`). The subtree that starts with chunk 12 includes chunks 13 through 15, but if the
/// tree were bigger it would not include chunk 16, so its max length is `Some(4096)` (4 *
/// CHUNK_LEN).
///
/// ```text
///                       root
///                /               \
///            .                       .
///        /       \               /       \
///      .           .           .           .
///    /   \       /   \       /   \       /   \
///   .     .     .     .     .     .     .     .
///  / \   / \   / \   / \   / \   / \   / \   / \
/// 0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15
/// ```
///
/// The general rule turns out to be that for a subtree starting at chunk index N (greater than
/// zero), the maximum number of chunks in the tree is 2 ^ (trailing zero bits of N).
///
/// # Panics
///
/// This function currently panics if `input_offset` is not a multiple of `CHUNK_LEN`. This is not
/// guaranteed.
#[inline(always)]
pub fn max_subtree_len(input_offset: u64) -> Option<u64> {
    if input_offset == 0 {
        return None;
    }
    assert_eq!(input_offset % CHUNK_LEN as u64, 0);
    let counter = input_offset / CHUNK_LEN as u64;
    let max_chunks = 1 << counter.trailing_zeros();
    Some(max_chunks * CHUNK_LEN as u64)
}

#[test]
fn test_max_subtree_len() {
    assert_eq!(max_subtree_len(0), None);
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
    for (chunk_index, max_chunks) in cases {
        let input_offset = chunk_index * CHUNK_LEN as u64;
        assert_eq!(
            max_subtree_len(input_offset),
            Some(max_chunks * CHUNK_LEN as u64),
        );
    }
}

/// The `mode` argument to [`merge_subtrees_non_root`] and friends
///
/// See the module level examples.
#[derive(Copy, Clone, Debug)]
pub enum Mode<'a> {
    /// Corresponding to [`hash`](crate::hash)
    Hash,

    /// Corresponding to [`keyed_hash`](crate::hash)
    KeyedHash(&'a [u8; KEY_LEN]),

    /// Corresponding to [`derive_key`](crate::hash)
    ///
    /// The [`ContextKey`] comes from [`hash_derive_key_context`].
    DeriveKeyMaterial(&'a ContextKey),
}

impl<'a> Mode<'a> {
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

/// "Chaining value" is the academic term for a non-root or non-final hash.
pub type ChainingValue = [u8; OUT_LEN];

fn merge_subtrees_inner(
    left_child: &ChainingValue,
    right_child: &ChainingValue,
    mode: Mode,
) -> crate::Output {
    crate::parent_node_output(
        &left_child,
        &right_child,
        &mode.key_words(),
        mode.flags_byte(),
        Platform::detect(),
    )
}

/// Compute a non-root chaining value, similar to [`Hasher::finalize_non_root`].
///
/// See the module level examples. "Chaining value" is the academic term for a non-root or
/// non-final hash.
pub fn merge_subtrees_non_root(
    left_child: &ChainingValue,
    right_child: &ChainingValue,
    mode: Mode,
) -> ChainingValue {
    merge_subtrees_inner(left_child, right_child, mode).chaining_value()
}

/// Compute a root hash, similar to [`Hasher::finalize`](crate::Hasher::finalize).
///
/// See the module level examples.
pub fn merge_subtrees_root(
    left_child: &ChainingValue,
    right_child: &ChainingValue,
    mode: Mode,
) -> crate::Hash {
    merge_subtrees_inner(left_child, right_child, mode).root_hash()
}

/// Return a root [`OutputReader`](crate::OutputReader), similar to
/// [`Hasher::finalize_xof`](crate::Hasher::finalize_xof).
pub fn merge_subtrees_xof(
    left_child: &ChainingValue,
    right_child: &ChainingValue,
    mode: Mode,
) -> crate::OutputReader {
    crate::OutputReader::new(merge_subtrees_inner(left_child, right_child, mode))
}

/// An alias to distinguish [`hash_derive_key_context`] outputs from other keys.
pub type ContextKey = [u8; KEY_LEN];

/// Hash a [`derive_key`](crate::derive_key) context string and return a [`ContextKey`].
///
/// The _only_ valid uses for the returned [`ContextKey`] are [`Hasher::new_from_context_key`] and
/// [`Mode::DeriveKeyMaterial`] (together with the merge subtree functions).
///
/// # Example
///
/// ```
/// use blake3::Hasher;
/// use blake3::hazmat::HasherExt;
///
/// let context_key = blake3::hazmat::hash_derive_key_context("foo");
/// let mut hasher = Hasher::new_from_context_key(&context_key);
/// hasher.update(b"bar");
/// let derived_key = *hasher.finalize().as_bytes();
///
/// assert_eq!(derived_key, blake3::derive_key("foo", b"bar"));
/// ```
pub fn hash_derive_key_context(context: &str) -> ContextKey {
    crate::hash_all_at_once::<crate::join::SerialJoin>(
        context.as_bytes(),
        IV,
        crate::DERIVE_KEY_CONTEXT,
        0,
    )
    .root_hash()
    .0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_empty_subtree_should_panic() {
        Hasher::new().finalize_non_root();
    }

    #[test]
    #[should_panic]
    fn test_unaligned_offset_should_panic() {
        Hasher::new().set_input_offset(1);
    }

    #[test]
    #[should_panic]
    fn test_hasher_already_accepted_input_should_panic() {
        Hasher::new().update(b"x").set_input_offset(0);
    }

    #[test]
    #[should_panic]
    fn test_too_much_input_should_panic() {
        Hasher::new()
            .set_input_offset(CHUNK_LEN as u64)
            .update(&[0; CHUNK_LEN + 1]);
    }

    #[test]
    #[should_panic]
    fn test_set_input_offset_cant_finalize() {
        Hasher::new().set_input_offset(CHUNK_LEN as u64).finalize();
    }

    #[test]
    #[should_panic]
    fn test_set_input_offset_cant_finalize_xof() {
        Hasher::new()
            .set_input_offset(CHUNK_LEN as u64)
            .finalize_xof();
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
                let mut chaining_values = arrayvec::ArrayVec::<ChainingValue, MAX_CHUNKS>::new();
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

        let mut hazmat_output = [0; 100];
        let left = Hasher::new_keyed(key).update(group0).finalize_non_root();
        let right = Hasher::new_keyed(key)
            .set_input_offset(group0.len() as u64)
            .update(group1)
            .finalize_non_root();
        merge_subtrees_xof(&left, &right, Mode::KeyedHash(&key)).fill(&mut hazmat_output);
        assert_eq!(expected_output, hazmat_output);
    }

    #[test]
    fn test_derive_key() {
        let context = "foo";
        let mut input = [0; 1025];
        crate::test::paint_test_input(&mut input);
        let expected = crate::derive_key(context, &input);

        let cx_key = hash_derive_key_context(context);
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
