//! Hash many independent, equal-length inputs at once.
//!
//! The functions in this module are additive: they don't change the behavior of
//! [`crate::hash`], [`crate::keyed_hash`], [`crate::derive_key`], or [`Hasher`](crate::Hasher) in
//! any way, and hashing a single input with one of them always produces exactly the same output as
//! the corresponding single-input function. What they add is the ability to hash a batch of
//! same-length, independent inputs (for example, the leaves of a Merkle tree) while sharing the
//! cost of SIMD parallelism across the whole batch, rather than paying it once per input.
//!
//! The one restriction is that a batch's inputs must all be the same length. That length can
//! be anything, exactly like [`crate::hash`] and friends - there's no block alignment to
//! worry about.
//!
//! # Performance
//!
//! Every input in a batch is hashed as its own complete, independent BLAKE3 output, not as
//! part of one larger tree. So unlike [`Hasher`](crate::Hasher), nothing here changes the hash
//! of any individual input based on what else is in the batch.
//!
//! Internally, this crate already knows how to hash up to `MAX_SIMD_DEGREE` *same-length*
//! blocks of *different* messages in one SIMD pass - that's how it parallelizes the chunks of
//! a single large input, and the parent nodes above them. This module reuses that machinery,
//! but batches it across independent *inputs* instead of across the chunks or parent nodes of
//! one input.
//!
//! For an input no longer than one chunk ([`CHUNK_LEN`], 1024 bytes), that's a single batched
//! call: every input's one-and-only chunk is compressed side by side, one lane per input, and
//! root-finalized directly. For a longer input, this module recurses the way [`crate::hash`] itself
//! does internally, splitting at [`hazmat::left_subtree_len`] to build the same BLAKE3 tree, except
//! every node of that recursion - each chunk, and each parent joining two subtrees - is computed
//! for every input in the batch together. Because the inputs share a length they share a tree
//! shape, so the recursion stays in lockstep.
//!
//! Within a chunk, `hash_many` compresses only *full* 64-byte blocks. Block length isn't part
//! of its interface on any backend; it's baked in as a constant everywhere
//! (`set1(BLOCK_LEN as u32)` in the Rust intrinsics, `set1(BLAKE3_BLOCK_LEN)` in the C
//! intrinsics, and a rodata constant in the hand-written assembly). So a chunk's full leading
//! blocks go through `hash_many` exactly as they always have, and its final block - the only
//! one BLAKE3 ever allows to be short - goes through [`Platform::compress_many`], the multi-lane
//! primitive that does take a runtime block length. Short inputs therefore get the same cross-input
//! SIMD parallelism as long ones.
use crate::platform::{MAX_SIMD_DEGREE, Platform};
use crate::{
    BLOCK_LEN, CHUNK_END, CHUNK_LEN, CHUNK_START, CVWords, DERIVE_KEY_MATERIAL, Hash,
    IncrementCounter, KEY_LEN, KEYED_HASH, OUT_LEN, PARENT, ROOT, hazmat, platform,
};
use arrayref::array_ref;
use arrayvec::ArrayVec;

/// Hash a batch of equal-length, independent inputs, writing one [`struct@Hash`] per input
/// into `out`.
///
/// This is equivalent to calling [`crate::hash`] on each input separately - `out[i] ==
/// crate::hash(inputs[i])` for every `i` - but it shares SIMD work across the whole batch.
/// See the [module documentation](self) for details.
///
/// Returns `false`, having written nothing, if `inputs` and `out` aren't the same length or
/// the inputs aren't all the same length as each other. When the batch size is known at
/// compile time this check folds away entirely.
#[must_use = "returns false, and hashes nothing, if the batch lengths don't line up"]
#[inline]
pub fn hash(inputs: &[&[u8]], out: &mut [Hash]) -> bool {
    hash_batch(inputs, crate::IV, 0, out)
}

/// The keyed hash function, applied to a batch of equal-length, independent inputs.
///
/// This is equivalent to calling [`crate::keyed_hash`] on each input separately, but it shares
/// SIMD work across the whole batch. See the [module documentation](self) for details.
///
/// Returns `false`, having written nothing, if `inputs` and `out` aren't the same length or
/// the inputs aren't all the same length as each other.
#[must_use = "returns false, and hashes nothing, if the batch lengths don't line up"]
#[inline]
pub fn keyed_hash(key: &[u8; KEY_LEN], inputs: &[&[u8]], out: &mut [Hash]) -> bool {
    let key_words = platform::words_from_le_bytes_32(key);
    hash_batch(inputs, &key_words, KEYED_HASH, out)
}

/// The key derivation function, applied to a batch of equal-length, independent key materials
/// sharing one context string.
///
/// This is equivalent to calling [`crate::derive_key`] with the same `context` on each element
/// of `key_materials` separately, but it shares SIMD work across the whole batch. See the
/// [module documentation](self) for details.
///
/// Returns `false`, having written nothing, if `key_materials` and `out` aren't the same
/// length or the key materials aren't all the same length as each other.
#[must_use = "returns false, and derives nothing, if the batch lengths don't line up"]
#[inline]
pub fn derive_key(context: &str, key_materials: &[&[u8]], out: &mut [[u8; OUT_LEN]]) -> bool {
    let context_key = hazmat::hash_derive_key_context(context);
    let context_key_words = platform::words_from_le_bytes_32(&context_key);
    hash_batch_bytes(key_materials, &context_key_words, DERIVE_KEY_MATERIAL, out)
}

// Validates a batch and returns the length its inputs share, or None if the batch is
// malformed. An empty batch is valid and yields 0.
//
// This is the single place the equal-length invariant is established. Everything below takes
// that length as a parameter instead of re-deriving it from `inputs[0]`, which keeps the
// optimizer from re-checking it at every level of the recursion. Always inlined so that a
// caller with a compile-time-known batch shape pays nothing for it.
#[must_use]
#[inline(always)]
fn batch_len<T>(inputs: &[&[u8]], out: &[T]) -> Option<usize> {
    if inputs.len() != out.len() {
        return None;
    }
    let Some(first) = inputs.first() else {
        return Some(0);
    };
    let len = first.len();
    if inputs.iter().all(|input| input.len() == len) {
        Some(len)
    } else {
        None
    }
}

#[inline]
fn hash_batch(inputs: &[&[u8]], key: &CVWords, flags: u8, out: &mut [Hash]) -> bool {
    let Some(len) = batch_len(inputs, out) else {
        return false;
    };
    let platform = Platform::detect();
    let degree = platform.simd_degree().max(1);
    for (in_batch, out_batch) in inputs.chunks(degree).zip(out.chunks_mut(degree)) {
        let mut raw = [[0u8; OUT_LEN]; MAX_SIMD_DEGREE];
        let raw_out = &mut raw[..in_batch.len()];
        hash_batch_group(&platform, in_batch, len, key, flags, raw_out);
        for (out1, raw1) in out_batch.iter_mut().zip(raw_out.iter()) {
            *out1 = Hash::from_bytes(*raw1);
        }
    }
    true
}

#[inline]
fn hash_batch_bytes(inputs: &[&[u8]], key: &CVWords, flags: u8, out: &mut [[u8; OUT_LEN]]) -> bool {
    let Some(len) = batch_len(inputs, out) else {
        return false;
    };
    let platform = Platform::detect();
    let degree = platform.simd_degree().max(1);
    for (in_batch, out_batch) in inputs.chunks(degree).zip(out.chunks_mut(degree)) {
        hash_batch_group(&platform, in_batch, len, key, flags, out_batch);
    }
    true
}

// Computes each input's root hash, batched across inputs at every level of the tree. Every
// input in `in_batch` is `len` bytes long, and there are at most MAX_SIMD_DEGREE of them.
fn hash_batch_group(
    platform: &Platform,
    in_batch: &[&[u8]],
    len: usize,
    key: &CVWords,
    flags: u8,
    out: &mut [[u8; OUT_LEN]],
) {
    debug_assert!(!in_batch.is_empty() && in_batch.len() <= MAX_SIMD_DEGREE);
    if len <= CHUNK_LEN {
        // The whole input is one chunk. Root-finalize its one and only block, which may be
        // empty or partial, the way ChunkState::output().root_hash() does.
        hash_chunk_batch(
            platform,
            in_batch,
            len,
            0,
            key,
            flags,
            CHUNK_END | ROOT,
            out,
        );
        return;
    }

    // More than one chunk: split into a left power-of-two-chunks subtree and a right
    // remainder, the way compress_subtree_wide() and hash_all_at_once() do for a single input.
    // The inputs share a length, so they split at the same point and produce the same tree
    // shape. That's what lets every node below be computed for the whole batch at once.
    let (left_len, right_len) = split_lens(len);
    let mut left_in: ArrayVec<&[u8], MAX_SIMD_DEGREE> = ArrayVec::new();
    let mut right_in: ArrayVec<&[u8], MAX_SIMD_DEGREE> = ArrayVec::new();
    for input in in_batch {
        let (left, right) = input.split_at(left_len);
        left_in.push(left);
        right_in.push(right);
    }
    let n = in_batch.len();
    let mut left_cvs = [[0u8; OUT_LEN]; MAX_SIMD_DEGREE];
    let mut right_cvs = [[0u8; OUT_LEN]; MAX_SIMD_DEGREE];
    let left_chunks = (left_len / CHUNK_LEN) as u64;
    hash_subtree_batch(
        platform,
        &left_in,
        left_len,
        0,
        key,
        flags,
        &mut left_cvs[..n],
    );
    let right_out = &mut right_cvs[..n];
    hash_subtree_batch(
        platform,
        &right_in,
        right_len,
        left_chunks,
        key,
        flags,
        right_out,
    );

    // This is the top of the tree for every input in the batch, so this parent compression is
    // where each input's ROOT flag is applied, the way hash_all_at_once() applies it to the
    // parent node returned by compress_subtree_to_parent_node().
    let flags = flags | ROOT;
    hash_parents_batch(platform, &left_cvs[..n], &right_cvs[..n], key, flags, out);
}

// The same recursion as the multi-chunk branch of hash_batch_group(), but for a non-root
// subtree: it never applies ROOT, and it takes a chunk_counter so this subtree's chunks are
// numbered correctly within their input.
fn hash_subtree_batch(
    platform: &Platform,
    in_batch: &[&[u8]],
    len: usize,
    chunk_counter: u64,
    key: &CVWords,
    flags: u8,
    out: &mut [[u8; OUT_LEN]],
) {
    if len <= CHUNK_LEN {
        hash_chunk_batch(
            platform,
            in_batch,
            len,
            chunk_counter,
            key,
            flags,
            CHUNK_END,
            out,
        );
        return;
    }

    let (left_len, right_len) = split_lens(len);
    let mut left_in: ArrayVec<&[u8], MAX_SIMD_DEGREE> = ArrayVec::new();
    let mut right_in: ArrayVec<&[u8], MAX_SIMD_DEGREE> = ArrayVec::new();
    for input in in_batch {
        let (left, right) = input.split_at(left_len);
        left_in.push(left);
        right_in.push(right);
    }
    let n = in_batch.len();
    let mut left_cvs = [[0u8; OUT_LEN]; MAX_SIMD_DEGREE];
    let mut right_cvs = [[0u8; OUT_LEN]; MAX_SIMD_DEGREE];
    let right_counter = chunk_counter + (left_len / CHUNK_LEN) as u64;
    hash_subtree_batch(
        platform,
        &left_in,
        left_len,
        chunk_counter,
        key,
        flags,
        &mut left_cvs[..n],
    );
    hash_subtree_batch(
        platform,
        &right_in,
        right_len,
        right_counter,
        key,
        flags,
        &mut right_cvs[..n],
    );
    hash_parents_batch(platform, &left_cvs[..n], &right_cvs[..n], key, flags, out);
}

// Splits a multi-chunk length the way this crate splits a single input's subtree.
#[inline(always)]
fn split_lens(len: usize) -> (usize, usize) {
    debug_assert!(len > CHUNK_LEN);
    let left_len = hazmat::left_subtree_len(len as u64) as usize;
    (left_len, len - left_len)
}

// Computes each input's chaining value, where every input is one chunk of `chunk_len` bytes.
// The full leading blocks, if any, are batched across inputs through the existing hash_many()
// kernels. The final block, the only one that may be short, goes through compress_many(),
// which takes a runtime block length.
#[allow(clippy::too_many_arguments)]
fn hash_chunk_batch(
    platform: &Platform,
    in_batch: &[&[u8]],
    chunk_len: usize,
    counter: u64,
    key: &CVWords,
    flags: u8,
    flags_end: u8,
    out: &mut [[u8; OUT_LEN]],
) {
    debug_assert!(!in_batch.is_empty() && in_batch.len() <= MAX_SIMD_DEGREE);
    debug_assert!(chunk_len <= CHUNK_LEN);
    let n = in_batch.len();
    let full_blocks = chunk_len / BLOCK_LEN;
    let remainder = chunk_len % BLOCK_LEN;

    if remainder == 0 && chunk_len > 0 {
        // The chunk is a whole number of full blocks, so every block including the last one
        // can go through hash_many() as-is.
        let mut cv_bytes = [0u8; MAX_SIMD_DEGREE * OUT_LEN];
        let cv_out = &mut cv_bytes[..n * OUT_LEN];
        dispatch_hash_many(
            full_blocks,
            platform,
            in_batch,
            counter,
            key,
            flags,
            flags_end,
            cv_out,
        );
        for (out1, cv) in out.iter_mut().zip(cv_out.chunks_exact(OUT_LEN)) {
            *out1 = *array_ref!(cv, 0, OUT_LEN);
        }
        return;
    }

    // The chunk's last block is short, which includes the empty chunk, where there are no full
    // blocks at all. Batch the full leading blocks through hash_many() to get each lane's
    // chaining value going into the final block.
    let mut cvs = [*key; MAX_SIMD_DEGREE];
    if full_blocks > 0 {
        let leading: ArrayVec<&[u8], MAX_SIMD_DEGREE> = in_batch
            .iter()
            .map(|input| &input[..full_blocks * BLOCK_LEN])
            .collect();
        let mut cv_bytes = [0u8; MAX_SIMD_DEGREE * OUT_LEN];
        let cv_out = &mut cv_bytes[..n * OUT_LEN];
        // Not the chunk's end yet; the real end is the short block finalized below.
        dispatch_hash_many(
            full_blocks,
            platform,
            &leading,
            counter,
            key,
            flags,
            0,
            cv_out,
        );
        for (cv, bytes) in cvs.iter_mut().zip(cv_out.chunks_exact(OUT_LEN)) {
            *cv = platform::words_from_le_bytes_32(array_ref!(bytes, 0, OUT_LEN));
        }
    }
    let start_flag = if full_blocks == 0 { CHUNK_START } else { 0 };
    let block_flags = flags | flags_end | start_flag;

    // Gather each lane's short final block, zero-padded, and compress them all at once. This
    // is the one block hash_many() can't help with, because block length isn't part of its
    // interface. compress_many() is the primitive that batches it with a real block_len.
    let mut blocks = [[0u8; BLOCK_LEN]; MAX_SIMD_DEGREE];
    for (block, input) in blocks[..n].iter_mut().zip(in_batch) {
        let tail = &input[full_blocks * BLOCK_LEN..];
        block[..tail.len()].copy_from_slice(tail);
    }
    let block_len = remainder as u8;
    platform.compress_many(&mut cvs[..n], &blocks[..n], block_len, counter, block_flags);
    for (out1, cv) in out.iter_mut().zip(cvs[..n].iter()) {
        *out1 = platform::le_bytes_from_words_32(cv);
    }
}

// block_count is in 1..=16 (CHUNK_LEN / BLOCK_LEN), so this dispatches to the same
// const-generic hash_many() this crate already uses internally for whole chunks
// (block_count == 16) and for parent nodes (block_count == 1). Values in between are new
// territory for callers, but the block loop inside every hash_many() backend is already a
// runtime loop over `blocks` (see e.g. rust_avx2::hash_many), not special-cased to those two.
#[allow(clippy::too_many_arguments)]
fn dispatch_hash_many(
    block_count: usize,
    platform: &Platform,
    in_batch: &[&[u8]],
    counter: u64,
    key: &CVWords,
    flags: u8,
    flags_end: u8,
    cv_out: &mut [u8],
) {
    macro_rules! dispatch {
        ($($blocks:literal),*) => {
            match block_count {
                $($blocks => hash_many_fixed::<{ $blocks * BLOCK_LEN }>(
                    platform, in_batch, counter, key, flags, flags_end, cv_out,
                ),)*
                _ => unreachable!("block_count out of range"),
            }
        };
    }
    dispatch!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
}

fn hash_many_fixed<const N: usize>(
    platform: &Platform,
    in_batch: &[&[u8]],
    counter: u64,
    key: &CVWords,
    flags: u8,
    flags_end: u8,
    cv_out: &mut [u8],
) {
    let mut arrays: ArrayVec<&[u8; N], MAX_SIMD_DEGREE> = ArrayVec::new();
    for &input in in_batch {
        arrays.push(input.try_into().expect("length already checked"));
    }
    platform.hash_many(
        &arrays,
        key,
        // Every lane is the same chunk index of a *different* independent input. Unlike
        // hashing several chunks of one input, where the counter increments per lane, the
        // counter is the same for every lane here; it only changes between calls, as the
        // recursion walks across each input's chunks.
        counter,
        IncrementCounter::No,
        flags,
        CHUNK_START,
        flags_end,
        cv_out,
    );
}

// Batches the parent-node compression that joins each input's left and right subtree chaining
// values, one lane per input, through the same hash_many() kernels used for chunks above (with
// a single BLOCK_LEN-sized "block" per lane). This is the batched counterpart of
// compress_parents_parallel(), across independent inputs rather than one input's tree.
fn hash_parents_batch(
    platform: &Platform,
    left_cvs: &[[u8; OUT_LEN]],
    right_cvs: &[[u8; OUT_LEN]],
    key: &CVWords,
    flags: u8,
    out: &mut [[u8; OUT_LEN]],
) {
    let n = left_cvs.len();
    debug_assert_eq!(n, right_cvs.len());
    debug_assert_eq!(n, out.len());
    debug_assert!(n <= MAX_SIMD_DEGREE);

    let mut blocks: ArrayVec<[u8; 2 * OUT_LEN], MAX_SIMD_DEGREE> = ArrayVec::new();
    for (left, right) in left_cvs.iter().zip(right_cvs) {
        let mut block = [0u8; 2 * OUT_LEN];
        block[..OUT_LEN].copy_from_slice(left);
        block[OUT_LEN..].copy_from_slice(right);
        blocks.push(block);
    }
    let arrays: ArrayVec<&[u8; 2 * OUT_LEN], MAX_SIMD_DEGREE> = blocks.iter().collect();

    let mut cv_bytes = [0u8; MAX_SIMD_DEGREE * OUT_LEN];
    let cv_out = &mut cv_bytes[..n * OUT_LEN];
    platform.hash_many(
        &arrays,
        key,
        // Parent nodes always use counter 0, same as compress_parents_parallel().
        0,
        IncrementCounter::No,
        flags | PARENT,
        0, // Parents have no start flags.
        0, // Parents have no end flags; callers OR ROOT into `flags` when they need it.
        cv_out,
    );
    for (out1, cv) in out.iter_mut().zip(cv_out.chunks_exact(OUT_LEN)) {
        *out1 = *array_ref!(cv, 0, OUT_LEN);
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use super::*;
    use crate::test::{TEST_CASES_MAX, paint_test_input};
    use alloc::vec;
    use alloc::vec::Vec;

    fn check(inputs: &[&[u8]], key: Option<&[u8; KEY_LEN]>, context: Option<&str>) {
        let mut got = vec![Hash::from_bytes([0; OUT_LEN]); inputs.len()];
        match (key, context) {
            (None, None) => assert!(hash(inputs, &mut got)),
            (Some(k), None) => assert!(keyed_hash(k, inputs, &mut got)),
            (None, Some(ctx)) => {
                let mut raw = vec![[0u8; OUT_LEN]; inputs.len()];
                assert!(derive_key(ctx, inputs, &mut raw));
                for (g, r) in got.iter_mut().zip(raw.iter()) {
                    *g = Hash::from_bytes(*r);
                }
            }
            (Some(_), Some(_)) => unreachable!(),
        }
        for (input, expected_hash) in inputs.iter().zip(got.iter()) {
            let want = match (key, context) {
                (None, None) => crate::hash(input),
                (Some(k), None) => crate::keyed_hash(k, input),
                (None, Some(ctx)) => Hash::from_bytes(crate::derive_key(ctx, input)),
                (Some(_), Some(_)) => unreachable!(),
            };
            assert_eq!(*expected_hash, want, "input length {}", input.len());
        }
    }

    #[test]
    fn test_many_matches_single_for_all_lengths_and_batch_sizes() {
        let mut input_buf = vec![0u8; TEST_CASES_MAX];
        paint_test_input(&mut input_buf);

        for &len in crate::test::TEST_CASES {
            for &batch_size in &[0usize, 1, 2, 3, 8, 17, MAX_SIMD_DEGREE, MAX_SIMD_DEGREE + 1] {
                if len > input_buf.len() {
                    continue;
                }
                let inputs: Vec<&[u8]> = (0..batch_size).map(|_| &input_buf[..len]).collect();
                check(&inputs, None, None);
                check(&inputs, Some(&crate::test::TEST_KEY), None);
                check(&inputs, None, Some("blake3 many() test context"));
            }
        }
    }

    #[test]
    fn test_many_mixed_content_arbitrary_lengths() {
        // Deliberately includes lengths that are not multiples of BLOCK_LEN, at and across
        // chunk and multi-chunk boundaries, to exercise the short-final-block path at every
        // depth of the recursion.
        const LENS: &[usize] = &[
            0,
            1,
            17,
            32,
            63,
            64,
            65,
            100,
            128,
            1000,
            1023,
            1024,
            1025,
            1057,
            2 * CHUNK_LEN,
            2 * CHUNK_LEN + 1,
            2 * CHUNK_LEN + 64,
            2 * CHUNK_LEN + 100,
            5 * CHUNK_LEN + 37,
            8 * CHUNK_LEN - 1,
            8 * CHUNK_LEN,
        ];
        // Start every input at a different offset, so that a lane mixup fails the test. The
        // buffer has to hold the longest length starting from the largest offset. Note that
        // the number of inputs scales with MAX_SIMD_DEGREE but the lengths above don't, so
        // sizing this from MAX_SIMD_DEGREE alone underflows wherever it is 1.
        const STRIDE: usize = 7;
        let num_inputs = 2 * MAX_SIMD_DEGREE + 1;
        let max_len = LENS.iter().copied().max().unwrap();
        let mut input_buf = vec![0u8; STRIDE * (num_inputs - 1) + max_len];
        paint_test_input(&mut input_buf);
        for &len in LENS {
            let inputs: Vec<&[u8]> = (0..num_inputs)
                .map(|i| &input_buf[i * STRIDE..][..len])
                .collect();
            check(&inputs, None, None);
        }
    }

    #[test]
    fn test_many_large_multi_chunk_inputs() {
        // Several chunks per input, a non-block-aligned length, and enough inputs to fill more
        // than one MAX_SIMD_DEGREE-sized group, to exercise the recursive tree-splitting path
        // across multiple batches with a short final block each time.
        const LEN: usize = 20 * CHUNK_LEN + 3 * BLOCK_LEN + 17;
        let mut input_buf = vec![0u8; (2 * MAX_SIMD_DEGREE + 5) * LEN];
        paint_test_input(&mut input_buf);
        let inputs: Vec<&[u8]> = input_buf.chunks_exact(LEN).collect();
        check(&inputs, None, None);
        check(&inputs, Some(&crate::test::TEST_KEY), None);
        check(&inputs, None, Some("blake3 many() large multi-chunk test"));
    }

    #[test]
    fn test_many_rejects_mismatched_input_lengths() {
        let a = [0u8; 32];
        let b = [0u8; 31];
        let inputs: [&[u8]; 2] = [&a, &b];
        let mut out = [Hash::from_bytes([0; OUT_LEN]); 2];
        assert!(!hash(&inputs, &mut out));
        assert!(!keyed_hash(&crate::test::TEST_KEY, &inputs, &mut out));
        let mut raw = [[0u8; OUT_LEN]; 2];
        assert!(!derive_key("ctx", &inputs, &mut raw));
        // Nothing was written.
        assert_eq!(out[0], Hash::from_bytes([0; OUT_LEN]));
        assert_eq!(out[1], Hash::from_bytes([0; OUT_LEN]));
    }

    #[test]
    fn test_many_rejects_mismatched_out_length() {
        let a = [0u8; 32];
        let inputs: [&[u8]; 1] = [&a];
        let mut out: [Hash; 2] = [Hash::from_bytes([0; OUT_LEN]); 2];
        assert!(!hash(&inputs, &mut out));
    }

    #[test]
    fn test_many_empty_batch_is_ok() {
        let inputs: [&[u8]; 0] = [];
        let mut out: [Hash; 0] = [];
        assert!(hash(&inputs, &mut out));
    }
}
