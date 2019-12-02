use core::cmp::min;
use core::convert::TryInto;

const OUT_LEN: usize = 32;
const KEY_LEN: usize = 32;
const BLOCK_LEN: usize = 64;
const CHUNK_LEN: usize = 2048;
const ROUNDS: usize = 7;

const CHUNK_START: u32 = 1 << 0;
const CHUNK_END: u32 = 1 << 1;
const PARENT: u32 = 1 << 2;
const ROOT: u32 = 1 << 3;
const KEYED_HASH: u32 = 1 << 4;
const DERIVE_KEY: u32 = 1 << 5;

const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const MSG_SCHEDULE: [[usize; 16]; ROUNDS] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
];

// The mixing function, G, which mixes either a column or a diagonal.
fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(mx);
    state[d] = (state[d] ^ state[a]).rotate_right(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(12);
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(my);
    state[d] = (state[d] ^ state[a]).rotate_right(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(7);
}

fn round(state: &mut [u32; 16], m: &[u32; 16], schedule: &[usize; 16]) {
    // Mix the columns.
    g(state, 0, 4, 8, 12, m[schedule[0]], m[schedule[1]]);
    g(state, 1, 5, 9, 13, m[schedule[2]], m[schedule[3]]);
    g(state, 2, 6, 10, 14, m[schedule[4]], m[schedule[5]]);
    g(state, 3, 7, 11, 15, m[schedule[6]], m[schedule[7]]);
    // Mix the diagonals.
    g(state, 0, 5, 10, 15, m[schedule[8]], m[schedule[9]]);
    g(state, 1, 6, 11, 12, m[schedule[10]], m[schedule[11]]);
    g(state, 2, 7, 8, 13, m[schedule[12]], m[schedule[13]]);
    g(state, 3, 4, 9, 14, m[schedule[14]], m[schedule[15]]);
}

fn compress(
    chaining_value: &[u32; 8],
    block_words: &[u32; 16],
    offset: u64,
    block_len: u32,
    flags: u32,
) -> [u32; 16] {
    let mut state = [
        chaining_value[0],
        chaining_value[1],
        chaining_value[2],
        chaining_value[3],
        chaining_value[4],
        chaining_value[5],
        chaining_value[6],
        chaining_value[7],
        IV[0],
        IV[1],
        IV[2],
        IV[3],
        offset as u32,
        (offset >> 32) as u32,
        block_len,
        flags,
    ];
    for r in 0..ROUNDS {
        round(&mut state, &block_words, &MSG_SCHEDULE[r]);
    }
    for i in 0..8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= chaining_value[i];
    }
    state
}

fn first_8_words(compression_output: [u32; 16]) -> [u32; 8] {
    compression_output[0..8].try_into().unwrap()
}

fn words_from_litte_endian_bytes(bytes: &[u8], words: &mut [u32]) {
    for (bytes_block, word) in bytes.chunks_exact(4).zip(words.iter_mut()) {
        *word = u32::from_le_bytes(bytes_block.try_into().unwrap());
    }
}

// Each chunk or parent node can produce either an 8-word chaining value or, by
// setting the ROOT flag, any number of final output bytes. The Output struct
// captures the state just prior to choosing between those two possibilities.
struct Output {
    input_chaining_value: [u32; 8],
    block_words: [u32; 16],
    offset: u64,
    block_len: u32,
    flags: u32,
}

impl Output {
    fn chaining_value(&self) -> [u32; 8] {
        first_8_words(compress(
            &self.input_chaining_value,
            &self.block_words,
            self.offset,
            self.block_len,
            self.flags,
        ))
    }

    fn root_output_bytes(&self, out_slice: &mut [u8]) {
        let mut offset = 0;
        for out_block in out_slice.chunks_mut(2 * OUT_LEN) {
            let words = compress(
                &self.input_chaining_value,
                &self.block_words,
                offset,
                self.block_len,
                self.flags | ROOT,
            );
            // The output length might not be a multiple of 4.
            for (word, out_word) in words.iter().zip(out_block.chunks_mut(4)) {
                out_word.copy_from_slice(&word.to_le_bytes()[..out_word.len()]);
            }
            offset += 2 * OUT_LEN as u64;
        }
    }
}

struct ChunkState {
    chaining_value: [u32; 8],
    offset: u64,
    block: [u8; BLOCK_LEN],
    block_len: u8,
    blocks_compressed: u8,
    flags: u32,
}

impl ChunkState {
    fn new(key: &[u32; 8], offset: u64, flags: u32) -> Self {
        Self {
            chaining_value: *key,
            offset,
            block: [0; BLOCK_LEN],
            block_len: 0,
            blocks_compressed: 0,
            flags,
        }
    }

    fn len(&self) -> usize {
        BLOCK_LEN * self.blocks_compressed as usize + self.block_len as usize
    }

    fn start_flag(&self) -> u32 {
        if self.blocks_compressed == 0 {
            CHUNK_START
        } else {
            0
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        while !input.is_empty() {
            if self.block_len as usize == BLOCK_LEN {
                let mut block_words = [0; 16];
                words_from_litte_endian_bytes(&self.block, &mut block_words);
                self.chaining_value = first_8_words(compress(
                    &self.chaining_value,
                    &block_words,
                    self.offset,
                    BLOCK_LEN as u32,
                    self.flags | self.start_flag(),
                ));
                self.blocks_compressed += 1;
                self.block = [0; BLOCK_LEN];
                self.block_len = 0;
            }

            let want = BLOCK_LEN - self.block_len as usize;
            let take = min(want, input.len());
            self.block[self.block_len as usize..][..take].copy_from_slice(&input[..take]);
            self.block_len += take as u8;
            input = &input[take..];
        }
    }

    fn output(&self) -> Output {
        let mut block_words = [0; 16];
        words_from_litte_endian_bytes(&self.block, &mut block_words);
        Output {
            input_chaining_value: self.chaining_value,
            block_words,
            block_len: self.block_len as u32,
            offset: self.offset,
            flags: self.flags | self.start_flag() | CHUNK_END,
        }
    }
}

fn parent_output(
    left_child_cv: &[u32; 8],
    right_child_cv: &[u32; 8],
    key: &[u32; 8],
    flags: u32,
) -> Output {
    let mut block_words = [0; 16];
    block_words[..8].copy_from_slice(left_child_cv);
    block_words[8..].copy_from_slice(right_child_cv);
    Output {
        input_chaining_value: *key,
        block_words,
        offset: 0,                   // Always 0 for parent nodes.
        block_len: BLOCK_LEN as u32, // Always BLOCK_LEN (64) for parent nodes.
        flags: PARENT | flags,
    }
}

/// An incremental hasher that can accept any number of writes.
pub struct Hasher {
    chunk_state: ChunkState,
    key: [u32; 8],
    subtree_stack: [[u32; 8]; 53], // Space for 53 subtree chaining values:
    subtree_stack_len: u8,         // 2^53 * CHUNK_LEN = 2^64
}

impl Hasher {
    fn new_internal(key: &[u32; 8], flags: u32) -> Self {
        Self {
            chunk_state: ChunkState::new(key, 0, flags),
            key: *key,
            subtree_stack: [[0; 8]; 53],
            subtree_stack_len: 0,
        }
    }

    /// Construct a new `Hasher` for the default **hash** mode.
    pub fn new() -> Self {
        Self::new_internal(&IV, 0)
    }

    /// Construct a new `Hasher` for the **keyed_hash** mode.
    pub fn new_keyed(key: &[u8; KEY_LEN]) -> Self {
        let mut key_words = [0; 8];
        words_from_litte_endian_bytes(key, &mut key_words);
        Self::new_internal(&key_words, KEYED_HASH)
    }

    /// Construct a new `Hasher` for the **derive_key** mode.
    pub fn new_derive_key(key: &[u8; KEY_LEN]) -> Self {
        let mut key_words = [0; 8];
        words_from_litte_endian_bytes(key, &mut key_words);
        Self::new_internal(&key_words, DERIVE_KEY)
    }

    fn push_stack(&mut self, cv: &[u32; 8]) {
        self.subtree_stack[self.subtree_stack_len as usize] = *cv;
        self.subtree_stack_len += 1;
    }

    fn pop_stack(&mut self) -> [u32; 8] {
        self.subtree_stack_len -= 1;
        self.subtree_stack[self.subtree_stack_len as usize]
    }

    fn push_chunk_chaining_value(&mut self, mut cv: [u32; 8], total_bytes: u64) {
        // The new chunk chaining value might complete some subtrees along the
        // right edge of the growing tree. For each completed subtree, pop its
        // left child CV off the stack and compress a new parent CV. After as
        // many parent compressions as possible, push the new CV onto the
        // stack. The final length of the stack will be the count of 1 bits in
        // the total number of chunks or (equivalently) input bytes so far.
        let final_stack_len = total_bytes.count_ones() as u8;
        while self.subtree_stack_len >= final_stack_len {
            cv = parent_output(&self.pop_stack(), &cv, &self.key, self.chunk_state.flags)
                .chaining_value();
        }
        self.push_stack(&cv);
    }

    /// Add input to the hash state. This can be called any number of times.
    pub fn update(&mut self, mut input: &[u8]) {
        while !input.is_empty() {
            if self.chunk_state.len() == CHUNK_LEN {
                let chunk_cv = self.chunk_state.output().chaining_value();
                let new_chunk_offset = self.chunk_state.offset + CHUNK_LEN as u64;
                self.push_chunk_chaining_value(chunk_cv, new_chunk_offset);
                self.chunk_state =
                    ChunkState::new(&self.key, new_chunk_offset, self.chunk_state.flags);
            }

            let want = CHUNK_LEN - self.chunk_state.len();
            let take = min(want, input.len());
            self.chunk_state.update(&input[..take]);
            input = &input[take..];
        }
    }

    /// Finalize the hash and write any number of output bytes.
    pub fn finalize(&self, out_slice: &mut [u8]) {
        // Starting with the Output from the current chunk, compute all the
        // parent chaining values along the right edge of the tree, until we
        // have the root Output.
        let mut output = self.chunk_state.output();
        let mut parent_nodes_remaining = self.subtree_stack_len as usize;
        while parent_nodes_remaining > 0 {
            parent_nodes_remaining -= 1;
            output = parent_output(
                &self.subtree_stack[parent_nodes_remaining],
                &output.chaining_value(),
                &self.key,
                self.chunk_state.flags,
            );
        }
        output.root_output_bytes(out_slice);
    }
}
