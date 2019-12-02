use crate::{offset_high, offset_low, BLOCK_LEN, IV, MSG_SCHEDULE, OUT_LEN};
use arrayref::{array_mut_ref, array_ref, array_refs};

#[inline(always)]
fn words_from_block(bytes: &[u8; BLOCK_LEN]) -> [u32; 16] {
    let refs = array_refs!(bytes, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4);
    [
        u32::from_le_bytes(*refs.0),
        u32::from_le_bytes(*refs.1),
        u32::from_le_bytes(*refs.2),
        u32::from_le_bytes(*refs.3),
        u32::from_le_bytes(*refs.4),
        u32::from_le_bytes(*refs.5),
        u32::from_le_bytes(*refs.6),
        u32::from_le_bytes(*refs.7),
        u32::from_le_bytes(*refs.8),
        u32::from_le_bytes(*refs.9),
        u32::from_le_bytes(*refs.10),
        u32::from_le_bytes(*refs.11),
        u32::from_le_bytes(*refs.12),
        u32::from_le_bytes(*refs.13),
        u32::from_le_bytes(*refs.14),
        u32::from_le_bytes(*refs.15),
    ]
}

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

pub fn compress(
    cv: &[u32; 8],
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    offset: u64,
    flags: u8,
) -> [u32; 16] {
    let block_words = words_from_block(block);
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
        offset_low(offset),
        offset_high(offset),
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

    state
}

pub fn hash1<A: arrayvec::Array<Item = u8>>(
    input: &A,
    key: &[u32; 8],
    offset: u64,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8; OUT_LEN],
) {
    debug_assert_eq!(A::CAPACITY % BLOCK_LEN, 0, "uneven blocks");
    let mut cv = *key;
    let mut block_flags = flags | flags_start;
    let mut slice = input.as_slice();
    while slice.len() >= BLOCK_LEN {
        if slice.len() == BLOCK_LEN {
            block_flags |= flags_end;
        }
        let output = compress(
            &cv,
            array_ref!(slice, 0, BLOCK_LEN),
            BLOCK_LEN as u8,
            offset,
            block_flags,
        );
        cv = *array_ref!(output, 0, 8);
        block_flags = flags;
        slice = &slice[BLOCK_LEN..];
    }
    *out = crate::bytes_from_state_words(&cv);
}

pub fn hash_many<A: arrayvec::Array<Item = u8>>(
    inputs: &[&A],
    key: &[u32; 8],
    mut offset: u64,
    offset_deltas: &[u64; 16],
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
            offset,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(output, 0, OUT_LEN),
        );
        offset += offset_deltas[1];
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_hash1_1() {
        let block = [1; BLOCK_LEN];
        let key = [2; 8];
        let offset = 3 * crate::CHUNK_LEN as u64;
        let flags = 4;
        let flags_start = 8;
        let flags_end = 16;

        let out = compress(
            &key,
            &block,
            BLOCK_LEN as u8,
            offset,
            flags | flags_start | flags_end,
        );
        let expected_out = crate::bytes_from_state_words(array_ref!(out, 0, 8));

        let mut test_out = [0; OUT_LEN];
        hash1(
            &block,
            &key,
            offset,
            flags,
            flags_start,
            flags_end,
            &mut test_out,
        );

        assert_eq!(expected_out, test_out);
    }

    #[test]
    fn test_hash1_3() {
        let mut blocks = [0; BLOCK_LEN * 3];
        crate::test::paint_test_input(&mut blocks);
        let key = [2; 8];
        let offset = 3 * crate::CHUNK_LEN as u64;
        let flags = 4;
        let flags_start = 8;
        let flags_end = 16;

        let mut expected_cv = key;
        let out = compress(
            &mut expected_cv,
            array_ref!(blocks, 0, BLOCK_LEN),
            BLOCK_LEN as u8,
            offset,
            flags | flags_start,
        );
        expected_cv = *array_ref!(out, 0, 8);
        let out = compress(
            &mut expected_cv,
            array_ref!(blocks, BLOCK_LEN, BLOCK_LEN),
            BLOCK_LEN as u8,
            offset,
            flags,
        );
        expected_cv = *array_ref!(out, 0, 8);
        let out = compress(
            &mut expected_cv,
            array_ref!(blocks, 2 * BLOCK_LEN, BLOCK_LEN),
            BLOCK_LEN as u8,
            offset,
            flags | flags_end,
        );
        expected_cv = *array_ref!(out, 0, 8);
        let expected_out = crate::bytes_from_state_words(&expected_cv);

        let mut test_out = [0; OUT_LEN];
        hash1(
            &blocks,
            &key,
            offset,
            flags,
            flags_start,
            flags_end,
            &mut test_out,
        );

        assert_eq!(expected_out, test_out);
    }

    #[test]
    fn test_hash_many() {
        let mut input_buf = [0; BLOCK_LEN * 9];
        crate::test::paint_test_input(&mut input_buf);
        let inputs = [
            array_ref!(input_buf, 0 * BLOCK_LEN, 3 * BLOCK_LEN),
            array_ref!(input_buf, 3 * BLOCK_LEN, 3 * BLOCK_LEN),
            array_ref!(input_buf, 6 * BLOCK_LEN, 3 * BLOCK_LEN),
        ];
        let key = [2; 8];
        let offset = 3 * crate::CHUNK_LEN as u64;
        let delta = crate::CHUNK_LEN as u64;
        let flags = 4;
        let flags_start = 8;
        let flags_end = 16;

        let mut expected_out = [0; 3 * OUT_LEN];
        hash1(
            inputs[0],
            &key,
            offset + 0 * delta,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(&mut expected_out, 0 * OUT_LEN, OUT_LEN),
        );
        hash1(
            inputs[1],
            &key,
            offset + 1 * delta,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(&mut expected_out, 1 * OUT_LEN, OUT_LEN),
        );
        hash1(
            inputs[2],
            &key,
            offset + 2 * delta,
            flags,
            flags_start,
            flags_end,
            array_mut_ref!(&mut expected_out, 2 * OUT_LEN, OUT_LEN),
        );

        let mut test_out = [0; OUT_LEN * 3];
        hash_many(
            &inputs,
            &key,
            offset,
            crate::CHUNK_OFFSET_DELTAS,
            flags,
            flags_start,
            flags_end,
            &mut test_out,
        );

        assert_eq!(&expected_out[..], &test_out[..]);
    }
}
