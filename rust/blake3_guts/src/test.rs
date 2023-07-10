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
                KEYED_HASH,
            );

            let mut test_cv = TEST_KEY;
            unsafe {
                let test_cv_ptr: *mut CVBytes = &mut test_cv;
                compress_fn(
                    &block,
                    block_len as u32,
                    test_cv_ptr,
                    counter,
                    KEYED_HASH,
                    test_cv_ptr,
                );
            }

            assert_eq!(portable_cv, test_cv);
        }
    }
}

pub fn test_compress_vs_reference(compress_fn: CompressFn) {
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
            compress_fn(
                &block,
                block_len as u32,
                test_cv_ptr,
                0,
                CHUNK_START | CHUNK_END | ROOT | KEYED_HASH,
                test_cv_ptr,
            );
        }

        assert_eq!(ref_hash, test_cv);
    }
}

fn check_transposed_eq(output_a: &TransposedVectors, output_b: &TransposedVectors) {
    if output_a == output_b {
        return;
    }
    for cv_index in 0..2 * MAX_SIMD_DEGREE {
        let cv_a = output_a.extract_cv(cv_index);
        let cv_b = output_b.extract_cv(cv_index);
        if cv_a == [0; 32] && cv_b == [0; 32] {
            println!("CV {cv_index:2} empty");
        } else if cv_a == cv_b {
            println!("CV {cv_index:2} matches");
        } else {
            println!("CV {cv_index:2} mismatch:");
            println!("    {}", hex::encode(cv_a));
            println!("    {}", hex::encode(cv_b));
        }
    }
    panic!("transposed outputs are not equal");
}

pub fn test_hash_chunks_vs_portable(hash_chunks_fn: HashChunksFn, degree: usize) {
    assert!(degree <= MAX_SIMD_DEGREE);
    let mut input = [0u8; 2 * MAX_SIMD_DEGREE * CHUNK_LEN];
    paint_test_input(&mut input);
    dbg!(degree * CHUNK_LEN);
    let mut input_2_lengths = vec![1];
    let mut next_len = CHUNK_LEN;
    // Try just below, equal to, and just above every power-of-2 number of chunks.
    loop {
        input_2_lengths.push(next_len - 1);
        input_2_lengths.push(next_len);
        if next_len == MAX_SIMD_DEGREE * CHUNK_LEN {
            break;
        }
        input_2_lengths.push(next_len + 1);
        next_len *= 2;
    }
    for input_2_len in input_2_lengths {
        dbg!(input_2_len);
        let input1 = &input[..degree * CHUNK_LEN];
        let input2 = &input[degree * CHUNK_LEN..][..input_2_len];
        for initial_counter in INITIAL_COUNTERS {
            // Make two calls, to test the output_column parameter.
            let mut portable_output = TransposedVectors::default();
            let (portable_left, portable_right) = portable_output.split(degree);
            Implementation::portable().hash_chunks(
                input1,
                &IV_BYTES,
                initial_counter,
                0,
                portable_left,
            );
            Implementation::portable().hash_chunks(
                input2,
                &TEST_KEY,
                initial_counter + degree as u64,
                KEYED_HASH,
                portable_right,
            );

            let mut test_output = TransposedVectors::default();
            let (test_left, test_right) = test_output.split(degree);
            unsafe {
                hash_chunks_fn(
                    input1.as_ptr(),
                    input1.len(),
                    &IV_BYTES,
                    initial_counter,
                    0,
                    test_left.ptr,
                );
                hash_chunks_fn(
                    input2.as_ptr(),
                    input2.len(),
                    &TEST_KEY,
                    initial_counter + degree as u64,
                    KEYED_HASH,
                    test_right.ptr,
                );
            }

            check_transposed_eq(&portable_output, &test_output);
        }
    }
}

fn painted_transposed_input() -> TransposedVectors {
    let mut vectors = TransposedVectors::default();
    let mut val = 0;
    for col in 0..2 * MAX_SIMD_DEGREE {
        for row in 0..8 {
            vectors.0[row][col] = val;
            val += 1;
        }
    }
    vectors
}

pub fn test_hash_parents_vs_portable(hash_parents_fn: HashParentsFn, degree: usize) {
    assert!(degree <= MAX_SIMD_DEGREE);
    let input = painted_transposed_input();
    for num_parents in 2..=(degree / 2) {
        dbg!(num_parents);
        let mut portable_output = TransposedVectors(input.0);
        let (portable_left, portable_right) = portable_output.split(degree);
        Implementation::portable().hash_parents(
            &input,
            2 * num_parents, // num_cvs
            &IV_BYTES,
            0,
            portable_left,
        );
        Implementation::portable().hash_parents(
            &input,
            2 * num_parents, // num_cvs
            &TEST_KEY,
            KEYED_HASH,
            portable_right,
        );

        let mut test_output = input.clone();
        let (test_left, test_right) = test_output.split(degree);
        unsafe {
            hash_parents_fn(
                input.as_ptr(),
                num_parents,
                &IV_BYTES,
                PARENT,
                test_left.ptr,
            );
            hash_parents_fn(
                input.as_ptr(),
                num_parents,
                &TEST_KEY,
                PARENT | KEYED_HASH,
                test_right.ptr,
            );
        }

        check_transposed_eq(&portable_output, &test_output);
    }
}
