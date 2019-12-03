#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod platform;
mod portable;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;
#[cfg(test)]
mod test;

use arrayref::array_ref;
use arrayvec::ArrayString;
use core::cmp;
use core::fmt;
use platform::Platform;

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

fn offset_low(offset: u64) -> u32 {
    offset as u32
}

fn offset_high(offset: u64) -> u32 {
    (offset >> 32) as u32
}

/// A BLAKE3 output of the default size, 32 bytes, which implements
/// constant-time equality.
#[derive(Clone, Copy)]
pub struct Hash([u8; OUT_LEN]);

impl Hash {
    pub fn as_bytes(&self) -> &[u8; OUT_LEN] {
        &self.0
    }

    pub fn to_hex(&self) -> ArrayString<[u8; 2 * OUT_LEN]> {
        let mut s = ArrayString::new();
        let table = b"0123456789abcdef";
        for &b in self.0.iter() {
            s.push(table[(b >> 4) as usize] as char);
            s.push(table[(b & 0xf) as usize] as char);
        }
        s
    }
}

impl From<[u8; OUT_LEN]> for Hash {
    fn from(bytes: [u8; OUT_LEN]) -> Self {
        Self(bytes)
    }
}

impl From<Hash> for [u8; OUT_LEN] {
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

/// This implementation is constant-time.
impl PartialEq for Hash {
    fn eq(&self, other: &Hash) -> bool {
        constant_time_eq::constant_time_eq(&self.0[..], &other.0[..])
    }
}

/// This implementation is constant-time.
impl PartialEq<[u8; OUT_LEN]> for Hash {
    fn eq(&self, other: &[u8; OUT_LEN]) -> bool {
        constant_time_eq::constant_time_eq(&self.0[..], other)
    }
}

impl Eq for Hash {}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash(0x{})", self.to_hex())
    }
}

// Each chunk or parent node can produce either a 32-byte chaining value or, by
// setting the ROOT flag, any number of final output bytes. The Output struct
// captures the state just prior to choosing between those two possibilities.
struct Output {
    input_chaining_value: [u8; 32],
    block: [u8; 64],
    block_len: u8,
    offset: u64,
    flags: Flags,
    platform: Platform,
}

impl Output {
    fn chaining_value(&self) -> [u8; 32] {
        let out = self.platform.compress(
            &self.input_chaining_value,
            &self.block,
            self.block_len,
            self.offset,
            self.flags,
        );
        *array_ref!(out, 0, 32)
    }

    fn root_hash(&self) -> Hash {
        debug_assert_eq!(self.offset, 0);
        let out = self.platform.compress(
            &self.input_chaining_value,
            &self.block,
            self.block_len,
            0,
            self.flags | Flags::ROOT,
        );
        Hash(*array_ref!(out, 0, 32))
    }

    fn root_output_bytes(&self, out_slice: &mut [u8]) {
        let mut offset = 0;
        for out_block in out_slice.chunks_mut(2 * OUT_LEN) {
            let out_bytes = self.platform.compress(
                &self.input_chaining_value,
                &self.block,
                self.block_len,
                offset,
                self.flags | Flags::ROOT,
            );
            out_block.copy_from_slice(&out_bytes[..out_block.len()]);
            offset += 2 * OUT_LEN as u64;
        }
    }
}

#[derive(Clone)]
struct ChunkState {
    cv: [u8; 32],
    offset: u64,
    buf: [u8; BLOCK_LEN],
    buf_len: u8,
    blocks_compressed: u8,
    flags: Flags,
    platform: Platform,
}

impl ChunkState {
    fn new(key: &[u8; 32], offset: u64, flags: Flags, platform: Platform) -> Self {
        Self {
            cv: *key,
            offset: 0,
            buf: [0; BLOCK_LEN],
            buf_len: 0,
            blocks_compressed: 0,
            flags,
            platform,
        }
    }

    fn len(&self) -> u64 {
        BLOCK_LEN as u64 * self.blocks_compressed as u64 + self.buf_len as u64
    }

    fn fill_buf(&mut self, input: &mut &[u8]) {
        let want = BLOCK_LEN - self.buf_len as usize;
        let take = cmp::min(want, input.len());
        self.buf[self.buf_len as usize..][..take].copy_from_slice(&input[..take]);
        self.buf_len += take as u8;
        *input = &input[take..];
    }

    fn start_flag(&self) -> Flags {
        if self.blocks_compressed == 0 {
            Flags::CHUNK_START
        } else {
            Flags::empty()
        }
    }

    // Try to avoid buffering as much as possible, by compressing directly from
    // the input slice when full blocks are available.
    fn update(&mut self, mut input: &[u8]) {
        if self.buf_len > 0 {
            self.fill_buf(&mut input);
            if !input.is_empty() {
                debug_assert_eq!(self.buf_len as usize, BLOCK_LEN);
                let block_flags = self.flags | self.start_flag(); // borrowck
                self.platform.compress(
                    &mut self.cv,
                    &self.buf,
                    BLOCK_LEN as u8,
                    self.offset,
                    block_flags,
                );
                self.buf_len = 0;
                self.buf = [0; BLOCK_LEN];
                self.blocks_compressed += 1;
            }
        }

        while input.len() > BLOCK_LEN {
            debug_assert_eq!(self.buf_len, 0);
            let block_flags = self.flags | self.start_flag(); // borrowck
            self.platform.compress(
                &mut self.cv,
                array_ref!(input, 0, BLOCK_LEN),
                BLOCK_LEN as u8,
                self.offset,
                block_flags,
            );
            self.blocks_compressed += 1;
            input = &input[BLOCK_LEN..];
        }

        self.fill_buf(&mut input);
        debug_assert!(input.is_empty());
        debug_assert!(self.len() <= CHUNK_LEN as u64);
    }

    fn finalize(&self) -> Output {
        let block_flags = self.flags | self.start_flag() | Flags::CHUNK_END;
        Output {
            input_chaining_value: self.cv,
            block: self.buf,
            block_len: self.buf_len,
            offset: self.offset,
            flags: block_flags,
            platform: self.platform,
        }
    }
}

// Don't derive(Debug), because the state may be secret.
impl fmt::Debug for ChunkState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ChunkState {{ len: {}, offset: {}, flags: {:?}, platform: {:?} }}",
            self.len(),
            self.offset,
            self.flags,
            self.platform
        )
    }
}
