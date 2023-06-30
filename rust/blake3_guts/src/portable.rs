use crate::{IV, MAX_SIMD_DEGREE, MSG_SCHEDULE, WORD_LEN};

pub const DEGREE: usize = MAX_SIMD_DEGREE;

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
fn compress_safe(
    block: &[u8; 64],
    block_len: u32,
    cv: &[u32; 8],
    counter: u64,
    flags: u32,
) -> [u32; 16] {
    let mut block_words = [0u32; 16];
    for word_index in 0..16 {
        block_words[word_index] = u32::from_le_bytes(
            block[WORD_LEN * word_index..][..WORD_LEN]
                .try_into()
                .unwrap(),
        );
    }
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
        counter as u32,
        (counter >> 32) as u32,
        block_len as u32,
        flags as u32,
    ];
    for round_number in 0..7 {
        round(&mut state, &block_words, round_number);
    }
    for i in 0..8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= (*cv)[i];
    }
    state
}

pub unsafe extern "C" fn compress(
    block: *const [u8; 64],
    block_len: u32,
    cv: *const [u32; 8],
    counter: u64,
    flags: u32,
    out: *mut [u32; 16],
) {
    *out = compress_safe(&*block, block_len, &*cv, counter, flags);
}

pub unsafe extern "C" fn hash_chunks(
    input: *const u8,
    input_len: usize,
    key: *const [u32; 8],
    counter: u64,
    flags: u32,
    transposed_output: *mut u32,
) {
    crate::hash_chunks_using_compress(
        compress,
        input,
        input_len,
        key,
        counter,
        flags,
        transposed_output,
    )
}

pub unsafe extern "C" fn hash_parents(
    transposed_input: *const u32,
    num_parents: usize,
    key: *const [u32; 8],
    flags: u32,
    transposed_output: *mut u32, // may overlap the input
) {
    crate::hash_parents_using_compress(
        compress,
        transposed_input,
        num_parents,
        key,
        flags,
        transposed_output,
    )
}

pub unsafe extern "C" fn xof(
    block: *const [u8; 64],
    block_len: u32,
    cv: *const [u32; 8],
    counter: u64,
    flags: u32,
    out: *mut u8,
    out_len: usize,
) {
    crate::xof_using_compress(compress, block, block_len, cv, counter, flags, out, out_len)
}

pub unsafe extern "C" fn xof_xor(
    block: *const [u8; 64],
    block_len: u32,
    cv: *const [u32; 8],
    counter: u64,
    flags: u32,
    out: *mut u8,
    out_len: usize,
) {
    crate::xof_xor_using_compress(compress, block, block_len, cv, counter, flags, out, out_len)
}

pub unsafe extern "C" fn universal_hash(
    input: *const u8,
    input_len: usize,
    key: *const [u32; 8],
    counter: u64,
    out: *mut [u8; 16],
) {
    crate::universal_hash_using_compress(compress, input, input_len, key, counter, out)
}
