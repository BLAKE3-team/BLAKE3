//! This undocumented and unstable module is for use cases like the `bao` crate,
//! which need to traverse the BLAKE3 Merkle tree and work with chunk and parent
//! chaining values directly. There might be breaking changes to this module
//! between patch versions.
//!
//! We could stabilize something like this module in the future. If you have a
//! use case for it, please let us know by filing a GitHub issue.

pub const BLOCK_LEN: usize = 64;
pub const CHUNK_LEN: usize = 1024;

#[derive(Clone, Copy, Debug)]
pub struct Guts {
    cv: crate::CVWords,
    flags: u8,
    platform: crate::platform::Platform,
}

impl Guts {
    fn new_internal(key: &crate::CVWords, flags: u8) -> Self {
        Self {
            cv: *key,
            flags,
            platform: crate::platform::Platform::detect(),
        }
    }

    pub fn new() -> Self {
        Self::new_internal(crate::IV, 0)
    }

    pub fn new_keyed(key: &[u8; crate::KEY_LEN]) -> Self {
        let key_words = crate::platform::words_from_le_bytes_32(key);
        Self::new_internal(&key_words, crate::KEYED_HASH)
    }

    pub fn new_derive_key(context: &str) -> Self {
        let context_key = crate::hash_all_at_once::<crate::join::SerialJoin>(
            context.as_bytes(),
            crate::IV,
            crate::DERIVE_KEY_CONTEXT,
        )
        .root_hash();
        let context_key_words = crate::platform::words_from_le_bytes_32(context_key.as_bytes());
        Self::new_internal(&context_key_words, crate::DERIVE_KEY_MATERIAL)
    }

    pub fn chunk_state(&self, chunk_counter: u64) -> ChunkState {
        ChunkState(crate::ChunkState::new(
            &self.cv,
            chunk_counter,
            self.flags,
            self.platform,
        ))
    }

    pub fn parent_cv(
        &self,
        left_child: &crate::Hash,
        right_child: &crate::Hash,
        is_root: bool,
    ) -> crate::Hash {
        let output = crate::parent_node_output(
            left_child.as_bytes(),
            right_child.as_bytes(),
            &self.cv,
            self.flags,
            self.platform,
        );
        if is_root {
            output.root_hash()
        } else {
            output.chaining_value().into()
        }
    }
}

impl Default for Guts {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ChunkState(crate::ChunkState);

impl ChunkState {
    pub fn new(chunk_counter: u64) -> Self {
        Guts::new().chunk_state(chunk_counter)
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

pub fn parent_cv(
    left_child: &crate::Hash,
    right_child: &crate::Hash,
    is_root: bool,
) -> crate::Hash {
    Guts::new().parent_cv(left_child, right_child, is_root)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk() {
        let input = b"foo";
        assert_eq!(
            crate::hash(input),
            Guts::new().chunk_state(0).update(input).finalize(true)
        );
    }

    #[test]
    fn test_keyed_chunk() {
        let key = &[42u8; crate::KEY_LEN];
        let input = b"foo";
        assert_eq!(
            crate::keyed_hash(&key, input),
            Guts::new_keyed(&key)
                .chunk_state(0)
                .update(input)
                .finalize(true)
        );
    }

    #[test]
    fn test_derive_key() {
        let context = "bar";
        let key_material = b"key material, not a password";
        assert_eq!(
            crate::derive_key(&context, key_material),
            Guts::new_derive_key(&context)
                .chunk_state(0)
                .update(key_material)
                .finalize(true)
                .0
        );
    }

    #[test]
    fn test_parents() {
        do_updates(&Guts::new(), &mut crate::Hasher::new());
    }

    #[test]
    fn test_keyed_parents() {
        let key = &[42u8; crate::KEY_LEN];
        do_updates(&Guts::new_keyed(key), &mut crate::Hasher::new_keyed(key));
    }

    #[test]
    fn test_derive_key_parents() {
        let context = "bar";
        do_updates(
            &Guts::new_derive_key(&context),
            &mut crate::Hasher::new_derive_key(&context),
        );
    }

    fn do_updates(guts: &Guts, hasher: &mut crate::Hasher) {
        let mut buf = [0; CHUNK_LEN];
        buf[0] = 'a' as u8;
        hasher.update(&buf);
        let chunk0_cv = guts.chunk_state(0).update(&buf).finalize(false);

        buf[0] = 'b' as u8;
        hasher.update(&buf);
        let chunk1_cv = guts.chunk_state(1).update(&buf).finalize(false);

        hasher.update(b"c");
        let chunk2_cv = guts.chunk_state(2).update(b"c").finalize(false);

        let parent = guts.parent_cv(&chunk0_cv, &chunk1_cv, false);
        let root = guts.parent_cv(&parent, &chunk2_cv, true);
        assert_eq!(hasher.finalize(), root);
    }
}
