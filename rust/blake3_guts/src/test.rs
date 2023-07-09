use crate::*;

pub const TEST_KEY: CVBytes = *b"whats the Elvish word for friend";

// Test a few different initial counter values.
// - 0: The base case.
// - i32::MAX: *No* overflow. But carry bugs in tricky SIMD code can screw this up, if you XOR when
//   you're supposed to ANDNOT.
// - u32::MAX: The low word of the counter overflows for all inputs except the first.
// - (42 << 32) + u32::MAX: Same but with a non-zero value in the high word.
const INITIAL_COUNTERS: [u64; 4] = [
    0,
    i32::MAX as u64,
    u32::MAX as u64,
    (42u64 << 32) + u32::MAX as u64,
];

const BLOCK_LENGTHS: [usize; 4] = [0, 1, 63, 64];

pub fn paint_test_input(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
}

pub fn test_compress_vs_portable(compress_fn: CompressFn) {
    let flags = KEYED_HASH;
    for block_len in BLOCK_LENGTHS {
        dbg!(block_len);
        let mut block = [0; BLOCK_LEN];
        paint_test_input(&mut block[..block_len]);
        for counter in INITIAL_COUNTERS {
            dbg!(counter);
            let portable_cv = Implementation::portable().compress(
                &block,
                block_len as u32,
                &TEST_KEY,
                counter,
                flags,
            );

            let mut test_cv = TEST_KEY;
            unsafe {
                let test_cv_ptr: *mut CVBytes = &mut test_cv;
                compress_fn(
                    &block,
                    block_len as u32,
                    test_cv_ptr,
                    counter,
                    flags,
                    test_cv_ptr,
                );
            }

            assert_eq!(portable_cv, test_cv);
        }
    }
}

pub fn test_compress_vs_reference(compress_fn: CompressFn) {
    let flags = CHUNK_START | CHUNK_END | ROOT | KEYED_HASH;
    for block_len in BLOCK_LENGTHS {
        dbg!(block_len);
        let mut block = [0; BLOCK_LEN];
        paint_test_input(&mut block[..block_len]);

        let mut ref_hasher = reference_impl::Hasher::new_keyed(&TEST_KEY);
        ref_hasher.update(&block[..block_len]);
        let mut ref_hash = [0u8; 32];
        ref_hasher.finalize(&mut ref_hash);

        let mut test_cv = TEST_KEY;
        unsafe {
            let test_cv_ptr: *mut CVBytes = &mut test_cv;
            compress_fn(&block, block_len as u32, test_cv_ptr, 0, flags, test_cv_ptr);
        }

        assert_eq!(ref_hash, test_cv);
    }
}
