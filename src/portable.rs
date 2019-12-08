use crate::{offset_high, offset_low, OffsetDeltas, BLOCK_LEN, IV, KEY_LEN, MSG_SCHEDULE, OUT_LEN};
use arrayref::{array_mut_ref, array_ref};

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
    cv: &[u8; 32],
    block: &[u8; BLOCK_LEN],
    block_len: u8,
    offset: u64,
    flags: u8,
) -> [u8; 64] {
    let block_words = [
        u32::from_le_bytes(*array_ref!(block, 0 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 1 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 2 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 3 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 4 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 5 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 6 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 7 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 8 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 9 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 10 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 11 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 12 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 13 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 14 * 4, 4)),
        u32::from_le_bytes(*array_ref!(block, 15 * 4, 4)),
    ];

    let mut state = [
        u32::from_le_bytes(*array_ref!(cv, 0 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 1 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 2 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 3 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 4 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 5 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 6 * 4, 4)),
        u32::from_le_bytes(*array_ref!(cv, 7 * 4, 4)),
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
    state[8] ^= u32::from_le_bytes(*array_ref!(cv, 0 * 4, 4));
    state[9] ^= u32::from_le_bytes(*array_ref!(cv, 1 * 4, 4));
    state[10] ^= u32::from_le_bytes(*array_ref!(cv, 2 * 4, 4));
    state[11] ^= u32::from_le_bytes(*array_ref!(cv, 3 * 4, 4));
    state[12] ^= u32::from_le_bytes(*array_ref!(cv, 4 * 4, 4));
    state[13] ^= u32::from_le_bytes(*array_ref!(cv, 5 * 4, 4));
    state[14] ^= u32::from_le_bytes(*array_ref!(cv, 6 * 4, 4));
    state[15] ^= u32::from_le_bytes(*array_ref!(cv, 7 * 4, 4));

    let mut out = [0; 64];
    out[0 * 4..][..4].copy_from_slice(&state[0].to_le_bytes());
    out[1 * 4..][..4].copy_from_slice(&state[1].to_le_bytes());
    out[2 * 4..][..4].copy_from_slice(&state[2].to_le_bytes());
    out[3 * 4..][..4].copy_from_slice(&state[3].to_le_bytes());
    out[4 * 4..][..4].copy_from_slice(&state[4].to_le_bytes());
    out[5 * 4..][..4].copy_from_slice(&state[5].to_le_bytes());
    out[6 * 4..][..4].copy_from_slice(&state[6].to_le_bytes());
    out[7 * 4..][..4].copy_from_slice(&state[7].to_le_bytes());
    out[8 * 4..][..4].copy_from_slice(&state[8].to_le_bytes());
    out[9 * 4..][..4].copy_from_slice(&state[9].to_le_bytes());
    out[10 * 4..][..4].copy_from_slice(&state[10].to_le_bytes());
    out[11 * 4..][..4].copy_from_slice(&state[11].to_le_bytes());
    out[12 * 4..][..4].copy_from_slice(&state[12].to_le_bytes());
    out[13 * 4..][..4].copy_from_slice(&state[13].to_le_bytes());
    out[14 * 4..][..4].copy_from_slice(&state[14].to_le_bytes());
    out[15 * 4..][..4].copy_from_slice(&state[15].to_le_bytes());
    out
}

pub fn hash1<A: arrayvec::Array<Item = u8>>(
    input: &A,
    key: &[u8; KEY_LEN],
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
        cv = *array_ref!(output, 0, 32);
        block_flags = flags;
        slice = &slice[BLOCK_LEN..];
    }
    *out = cv;
}

pub fn hash_many<A: arrayvec::Array<Item = u8>>(
    inputs: &[&A],
    key: &[u8; KEY_LEN],
    mut offset: u64,
    offset_deltas: &OffsetDeltas,
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

    // This is basically testing the portable implementation against itself,
    // but we do it anyway for completeness. Other implementations will test
    // themselves against portable. We also have several tests against the
    // reference implementation in test.rs.
    #[test]
    fn test_compress() {
        crate::test::test_compress_fn(compress);
    }

    // Ditto.
    #[test]
    fn test_hash_many() {
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }
}
