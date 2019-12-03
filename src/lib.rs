use arrayref::{array_refs, mut_array_refs};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod platform;
mod portable;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;
#[cfg(test)]
mod test;

/// The default number of bytes in a hash, 32.
pub const OUT_LEN: usize = 32;

/// The number of bytes in a key, 32.
pub const KEY_LEN: usize = 32;

// These are pub for tests and benchmarks. Callers don't need them.
#[doc(hidden)]
pub const BLOCK_LEN: usize = 64;
#[doc(hidden)]
pub const CHUNK_LEN: usize = 2048;

const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const MSG_SCHEDULE: [[usize; 16]; 7] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
];

const CHUNK_OFFSET_DELTAS: &[u64; 16] = &[
    CHUNK_LEN as u64 * 0,
    CHUNK_LEN as u64 * 1,
    CHUNK_LEN as u64 * 2,
    CHUNK_LEN as u64 * 3,
    CHUNK_LEN as u64 * 4,
    CHUNK_LEN as u64 * 5,
    CHUNK_LEN as u64 * 6,
    CHUNK_LEN as u64 * 7,
    CHUNK_LEN as u64 * 8,
    CHUNK_LEN as u64 * 9,
    CHUNK_LEN as u64 * 10,
    CHUNK_LEN as u64 * 11,
    CHUNK_LEN as u64 * 12,
    CHUNK_LEN as u64 * 13,
    CHUNK_LEN as u64 * 14,
    CHUNK_LEN as u64 * 15,
];

const PARENT_OFFSET_DELTAS: &[u64; 16] = &[0; 16];

// These are the internal flags that we use to domain separate root/non-root,
// chunk/parent, and chunk beginning/middle/end. These get set at the high end
// of the block flags word in the compression function, so their values start
// high and go down.
bitflags::bitflags! {
    struct Flags: u8 {
        const CHUNK_START = 1 << 0;
        const CHUNK_END = 1 << 1;
        const PARENT = 1 << 2;
        const ROOT = 1 << 3;
        const KEYED_HASH = 1 << 4;
        const DERIVE_KEY = 1 << 5;
    }
}

fn words_from_key_bytes(bytes: &[u8; KEY_LEN]) -> [u32; 8] {
    // Parse the message bytes as little endian words.
    let refs = array_refs!(bytes, 4, 4, 4, 4, 4, 4, 4, 4);
    [
        u32::from_le_bytes(*refs.0),
        u32::from_le_bytes(*refs.1),
        u32::from_le_bytes(*refs.2),
        u32::from_le_bytes(*refs.3),
        u32::from_le_bytes(*refs.4),
        u32::from_le_bytes(*refs.5),
        u32::from_le_bytes(*refs.6),
        u32::from_le_bytes(*refs.7),
    ]
}

fn bytes_from_state_words(words: &[u32; 8]) -> [u8; OUT_LEN] {
    let mut bytes = [0; OUT_LEN];
    {
        let refs = mut_array_refs!(&mut bytes, 4, 4, 4, 4, 4, 4, 4, 4);
        *refs.0 = words[0].to_le_bytes();
        *refs.1 = words[1].to_le_bytes();
        *refs.2 = words[2].to_le_bytes();
        *refs.3 = words[3].to_le_bytes();
        *refs.4 = words[4].to_le_bytes();
        *refs.5 = words[5].to_le_bytes();
        *refs.6 = words[6].to_le_bytes();
        *refs.7 = words[7].to_le_bytes();
    }
    bytes
}

fn offset_low(offset: u64) -> u32 {
    offset as u32
}

fn offset_high(offset: u64) -> u32 {
    (offset >> 32) as u32
}
