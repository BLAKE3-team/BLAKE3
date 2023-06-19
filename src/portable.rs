use crate::platform::{ParentInOut, TransposedVectors, MAX_SIMD_DEGREE};
use crate::{
    counter_high, counter_low, CVBytes, CVWords, IncrementCounter, BLOCK_LEN, CHUNK_LEN, IV,
    MSG_SCHEDULE, OUT_LEN, UNIVERSAL_HASH_LEN,
};
use arrayref::{array_mut_ref, array_ref};
use core::cmp;

#[inline(always)]
fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32) {
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(x);
    state[d] = (state[d] ^ state[a]).rotate_right(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(12);
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(y);
    state[d] = (state[d] ^ state[a]).rotate_right(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(7);
}

#[inline(always)]
fn round(state: &mut [u32; 16], msg: &[u32; 16], round: usize) {
    // Select the message schedule based on the round.
    let schedule = MSG_SCHEDULE[round];

    // Mix the columns.
    g(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
    g(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
    g(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
    g(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);

    // Mix the diagonals.
    g(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]]);
    g(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]]);
    g(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]]);
    g(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]]);
}

#[inline(always)]
fn compress_pre(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) -> [u32; 16] {
    let block_words = crate::platform::words_from_le_bytes_64(block);

    let mut state = [
        cv[0],
        cv[1],
        cv[2],
        cv[3],
        cv[4],
        cv[5],
        cv[6],
        cv[7],
        IV[0],
        IV[1],
        IV[2],
        IV[3],
        counter_low(counter),
        counter_high(counter),
        block_len as u32,
        flags as u32,
    ];

    round(&mut state, &block_words, 0);
    round(&mut state, &block_words, 1);
    round(&mut state, &block_words, 2);
    round(&mut state, &block_words, 3);
    round(&mut state, &block_words, 4);
    round(&mut state, &block_words, 5);
    round(&mut state, &block_words, 6);

    state
}

pub fn compress_in_place(
    cv: &mut CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) {
    let state = compress_pre(cv, block, block_len, counter, flags);

    cv[0] = state[0] ^ state[8];
    cv[1] = state[1] ^ state[9];
    cv[2] = state[2] ^ state[10];
    cv[3] = state[3] ^ state[11];
    cv[4] = state[4] ^ state[12];
    cv[5] = state[5] ^ state[13];
    cv[6] = state[6] ^ state[14];
    cv[7] = state[7] ^ state[15];
}

pub fn compress_xof(
    cv: &CVWords,
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    counter: u64,
    flags: u8,
) -> [u8; 64] {
    let mut state = compress_pre(cv, block, block_len, counter, flags);
    state[0] ^= state[8];
    state[1] ^= state[9];
    state[2] ^= state[10];
    state[3] ^= state[11];
    state[4] ^= state[12];
    state[5] ^= state[13];
    state[6] ^= state[14];
    state[7] ^= state[15];
    state[8] ^= cv[0];
    state[9] ^= cv[1];
    state[10] ^= cv[2];
    state[11] ^= cv[3];
    state[12] ^= cv[4];
    state[13] ^= cv[5];
    state[14] ^= cv[6];
    state[15] ^= cv[7];
    crate::platform::le_bytes_from_words_64(&state)
}

pub fn hash1<const N: usize>(
    input: &[u8; N],
    key: &CVWords,
    counter: u64,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut CVBytes,
) {
    debug_assert_eq!(N % BLOCK_LEN, 0, "uneven blocks");
    let mut cv = *key;
    let mut block_flags = flags | flags_start;
    let mut slice = &input[..];
    while slice.len() >= BLOCK_LEN {
        if slice.len() == BLOCK_LEN {
            block_flags |= flags_end;
        }
        compress_in_place(
            &mut cv,
            array_ref!(slice, 0, BLOCK_LEN),
            BLOCK_LEN as u8,
            counter,
            block_flags,
        );
        block_flags = flags;
        slice = &slice[BLOCK_LEN..];
    }
    *out = crate::platform::le_bytes_from_words_32(&cv);
}

pub fn hash_many<const N: usize>(
    inputs: &[&[u8; N]],
    key: &CVWords,
    mut counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8],
) {
    debug_assert!(out.len() >= inputs.len() * OUT_LEN, "out too short");
    for (&input, output) in inputs.iter().zip(out.chunks_exact_mut(OUT_LEN)) {
        hash1(
            input,
            key,
            counter,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(output, 0, OUT_LEN),
        );
        if increment_counter.yes() {
            counter += 1;
        }
    }
}

/// General contract:
/// - `input` is N chunks, each exactly 1 KiB, 1 <= N <= DEGREE
/// - `output_column` is a multiple of DEGREE.
/// The CHUNK_START and CHUNK_END flags are set internally. Writes N transposed CVs to the output,
/// from `output_column` to `output_column+N-1`. Columns prior to `output_column` must be
/// unmodified.
///
/// This portable implementation has no particular DEGREE. It will accept any number of chunks up
/// to MAX_SIMD_DEGREE.
pub fn hash_chunks(
    input: &[u8],
    key: &[u32; 8],
    counter: u64,
    flags: u8,
    output: &mut TransposedVectors,
    output_column: usize,
) {
    debug_assert_eq!(input.len() % CHUNK_LEN, 0);
    let num_chunks = input.len() / CHUNK_LEN;
    debug_assert!(num_chunks <= MAX_SIMD_DEGREE);
    for chunk_index in 0..num_chunks {
        let mut cv = *key;
        for block_index in 0..16 {
            let block_flags = match block_index {
                0 => flags | crate::CHUNK_START,
                15 => flags | crate::CHUNK_END,
                _ => flags,
            };
            compress_in_place(
                &mut cv,
                input[CHUNK_LEN * chunk_index + BLOCK_LEN * block_index..][..BLOCK_LEN]
                    .try_into()
                    .unwrap(),
                BLOCK_LEN as u8,
                counter + chunk_index as u64,
                block_flags,
            );
        }
        for word_index in 0..cv.len() {
            output[word_index][output_column + chunk_index] = cv[word_index];
        }
    }
}

/// General contract:
/// - `cvs` contains `2*num_parents` transposed CVs, 1 <= num_parents <= DEGREE, starting at column 0
/// There may be additional CVs present beyond the `2*num_parents` CVs indicated, but this function
/// isn't aware of them and must not modify them. (The caller will take care of an odd remaining
/// CV, if any.) No flags are set internally. (The caller must set `PARENT` in `flags`). Writes
/// `num_parents` transposed parent CVs to the output, starting at column 0.
///
/// This portable implementation has no particular DEGREE. It will accept any number of parents up
/// to MAX_SIMD_DEGREE.
pub fn hash_parents(mut in_out: ParentInOut, key: &[u32; 8], flags: u8) {
    let (_, num_parents) = in_out.input();
    debug_assert!(num_parents <= MAX_SIMD_DEGREE);
    for parent_index in 0..num_parents {
        let (input, _) = in_out.input();
        let mut block = [0u8; BLOCK_LEN];
        for i in 0..8 {
            block[4 * i..][..4].copy_from_slice(&input[i][2 * parent_index].to_le_bytes());
            block[4 * (i + 8)..][..4]
                .copy_from_slice(&input[i][2 * parent_index + 1].to_le_bytes());
        }
        let mut cv = *key;
        compress_in_place(&mut cv, &block, BLOCK_LEN as u8, 0, flags);
        let (output, output_column) = in_out.output();
        for i in 0..8 {
            output[i][output_column + parent_index] = cv[i];
        }
    }
}

pub fn xof(
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    cv: &[u32; 8],
    mut counter: u64,
    flags: u8,
    mut out: &mut [u8],
) {
    while !out.is_empty() {
        let block_output = compress_xof(cv, block, block_len, counter, flags);
        let take = cmp::min(BLOCK_LEN, out.len());
        out[..take].copy_from_slice(&block_output[..take]);
        out = &mut out[take..];
        counter += 1;
    }
}

pub fn xof_xor(
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    cv: &[u32; 8],
    mut counter: u64,
    flags: u8,
    mut out: &mut [u8],
) {
    while !out.is_empty() {
        let block_output = compress_xof(cv, block, block_len, counter, flags);
        let take = cmp::min(BLOCK_LEN, out.len());
        for i in 0..take {
            out[i] ^= block_output[i];
        }
        out = &mut out[take..];
        counter += 1;
    }
}

pub fn universal_hash(
    mut input: &[u8],
    key: &[u32; 8],
    mut counter: u64,
) -> [u8; UNIVERSAL_HASH_LEN] {
    let flags = crate::KEYED_HASH | crate::CHUNK_START | crate::CHUNK_END | crate::ROOT;
    let mut result = [0u8; UNIVERSAL_HASH_LEN];
    while input.len() > BLOCK_LEN {
        let block_output = compress_xof(
            key,
            &input[..BLOCK_LEN].try_into().unwrap(),
            BLOCK_LEN as u8,
            counter,
            flags,
        );
        for i in 0..UNIVERSAL_HASH_LEN {
            result[i] ^= block_output[i];
        }
        input = &input[BLOCK_LEN..];
        counter += 1;
    }
    let mut final_block = [0u8; BLOCK_LEN];
    final_block[..input.len()].copy_from_slice(input);
    let final_output = compress_xof(key, &final_block, input.len() as u8, counter, flags);
    for i in 0..UNIVERSAL_HASH_LEN {
        result[i] ^= final_output[i];
    }
    result
}

#[cfg(test)]
pub mod test {
    use super::*;

    // These are basically testing the portable implementation against itself, but we also check
    // that compress_in_place and compress_xof are consistent. And there are tests against the
    // reference implementation and against hardcoded test vectors elsewhere.

    #[test]
    fn test_compress() {
        crate::test::test_compress_fn(compress_in_place, compress_xof);
    }

    // Ditto.
    #[test]
    fn test_hash_many() {
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }

    // The portable implementations of the vectorized APIs aren't actually vectorized and don't
    // have any inherent DEGREE. They loop internally over any number of inputs. Here we
    // arbitrarily pick degree 4 to test them (against themselves, so not an especially interesting
    // test).
    const TEST_DEGREE: usize = 4;

    #[test]
    fn test_hash_chunks() {
        crate::test::test_hash_chunks_fn(hash_chunks, TEST_DEGREE);
    }

    #[test]
    fn test_hash_parents() {
        crate::test::test_hash_parents_fn(hash_parents, TEST_DEGREE);
    }

    #[test]
    fn test_xof_and_xor() {
        crate::test::test_xof_and_xor_fns(xof, xof_xor);
    }

    #[test]
    fn test_universal_hash() {
        crate::test::test_universal_hash_fn(universal_hash);
    }
}
