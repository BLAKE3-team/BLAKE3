//! This undocumented and unstable module is for use cases like the `bao` crate,
//! which need to traverse the BLAKE3 Merkle tree and work with chunk and parent
//! chaining values directly. There might be breaking changes to this module
//! between patch versions.
//!
//! We could stabilize something like this module in the future. If you have a
//! use case for it, please let us know by filing a GitHub issue.

use crate::{Hash, Hasher};

pub const BLOCK_LEN: usize = 64;
pub const CHUNK_LEN: usize = 1024;

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
pub fn parent_cv(left_child: &Hash, right_child: &Hash, is_root: bool) -> Hash {
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

/// Adjust a regular Hasher so that it can hash a subtree whose starting byte offset is something
/// other than zero. Morally speaking, this parameter should be in a Hasher constructor function,
/// but those are public APIs, and I don't want to complicate them while this is still unstable.
/// This should only be called immediately after a Hasher is constructed, and we do our best to
/// assert that rule. (Changing this parameter after some input has already been compressed would
/// lead to a garbage hash.) Subtree hashes that aren't the root must use finalize_nonroot() below,
/// and we also do our best to assert that.
pub fn set_offset(hasher: &mut Hasher, starting_offset: u64) {
    assert_eq!(0, hasher.cv_stack.len());
    assert_eq!(0, hasher.chunk_state.len());
    assert_eq!(0, starting_offset % CHUNK_LEN as u64);
    hasher.initial_chunk_counter = starting_offset / CHUNK_LEN as u64;
    hasher.chunk_state.chunk_counter = hasher.initial_chunk_counter;
}

/// Finalize a Hasher as a non-root subtree. Callers using set_offset() above must use this instead
/// of the regular .finalize() method for any non-root subtrees, or else they'll get garbage hash.
/// BUT BE CAREFUL: Whatever your subtree size might be, you probably need to account for the case
/// where you have only one subtree (with offset zero), and in that case it *does* need to be
/// root-finalized.
pub fn finalize_nonroot(hasher: &Hasher) -> Hash {
    Hash(hasher.final_output().chaining_value())
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
    fn test_offset() {
        let mut input = [0; 12 * CHUNK_LEN + 1];
        crate::test::paint_test_input(&mut input);

        let mut hasher0 = Hasher::new();
        set_offset(&mut hasher0, 0 * CHUNK_LEN as u64);
        hasher0.update(&input[0 * CHUNK_LEN..][..4 * CHUNK_LEN]);
        let subtree0 = finalize_nonroot(&hasher0);

        let mut hasher1 = Hasher::new();
        set_offset(&mut hasher1, 4 * CHUNK_LEN as u64);
        hasher1.update(&input[4 * CHUNK_LEN..][..4 * CHUNK_LEN]);
        let subtree1 = finalize_nonroot(&hasher1);

        let mut hasher2 = Hasher::new();
        set_offset(&mut hasher2, 8 * CHUNK_LEN as u64);
        hasher2.update(&input[8 * CHUNK_LEN..][..4 * CHUNK_LEN]);
        let subtree2 = finalize_nonroot(&hasher2);

        let mut hasher3 = Hasher::new();
        set_offset(&mut hasher3, 12 * CHUNK_LEN as u64);
        hasher3.update(&input[12 * CHUNK_LEN..][..1]);
        let subtree3 = finalize_nonroot(&hasher3);

        let parent0 = parent_cv(&subtree0, &subtree1, false);
        let parent1 = parent_cv(&subtree2, &subtree3, false);
        let root = parent_cv(&parent0, &parent1, true);

        assert_eq!(crate::hash(&input), root);
    }

    #[test]
    #[should_panic]
    fn test_odd_offset() {
        let mut hasher = Hasher::new();
        set_offset(&mut hasher, 1);
    }

    #[test]
    #[should_panic]
    fn test_nonempty_offset_short() {
        let mut hasher = Hasher::new();
        hasher.update(b"hello");
        set_offset(&mut hasher, 0);
    }

    #[test]
    #[should_panic]
    fn test_nonempty_offset_long() {
        let mut hasher = Hasher::new();
        hasher.update(&[0; 4096]);
        set_offset(&mut hasher, 0);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_offset_then_update_too_much() {
        let mut hasher = Hasher::new();
        set_offset(&mut hasher, 12 * CHUNK_LEN as u64);
        hasher.update(&[0; 4 * CHUNK_LEN + 1]);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_offset_then_root_finalize_xof() {
        let mut hasher = Hasher::new();
        set_offset(&mut hasher, 2 * CHUNK_LEN as u64);
        hasher.update(&[0; 2 * CHUNK_LEN]);
        hasher.finalize_xof();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_offset_then_root_finalize() {
        let mut hasher = Hasher::new();
        set_offset(&mut hasher, 2 * CHUNK_LEN as u64);
        hasher.update(&[0; 2 * CHUNK_LEN]);
        hasher.finalize();
    }
}
