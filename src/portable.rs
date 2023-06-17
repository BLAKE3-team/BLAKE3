use crate::{
    counter_high, counter_low, platform::TransposedVectors, CVBytes, CVWords, IncrementCounter,
    BLOCK_LEN, CHUNK_LEN, IV, MSG_SCHEDULE, OUT_LEN, UNIVERSAL_HASH_LEN,
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

pub fn hash_chunks(
    input: &[u8],
    key: &[u32; 8],
    counter: u64,
    flags: u8,
    output: &mut TransposedVectors,
    output_offset: usize,
) {
    const LAST_BLOCK_INDEX: usize = (CHUNK_LEN / BLOCK_LEN) - 1;
    // There might be a partial chunk at the end. If so, we ignore it here, and the caller will
    // hash it separately.
    let num_chunks = input.len() / CHUNK_LEN;
    for chunk_index in 0..num_chunks {
        let mut cv = *key;
        for block_index in 0..CHUNK_LEN / BLOCK_LEN {
            compress_in_place(
                &mut cv,
                input[CHUNK_LEN * chunk_index + BLOCK_LEN * block_index..][..BLOCK_LEN]
                    .try_into()
                    .unwrap(),
                BLOCK_LEN as u8,
                counter + chunk_index as u64,
                match block_index {
                    0 => flags | crate::CHUNK_START,
                    LAST_BLOCK_INDEX => flags | crate::CHUNK_END,
                    _ => flags,
                },
            );
        }
        for word_index in 0..cv.len() {
            output.0[word_index][output_offset + chunk_index] = cv[word_index];
        }
    }
}

pub fn hash_parents(cvs: &mut TransposedVectors, num_cvs: usize, key: &[u32; 8], flags: u8) {
    // Note that there may be an odd number of children. If there's a leftover child, it gets
    // appended to the outputs by the caller. We will not overwrite it.
    let num_parents = num_cvs / 2;
    todo!()
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

    #[test]
    fn test_xof_and_xor() {
        crate::test::test_xof_and_xor_fns(xof, xof_xor);
    }

    #[test]
    fn test_universal_hash() {
        crate::test::test_universal_hash_fn(universal_hash);
    }
}
