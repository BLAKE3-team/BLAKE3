use crate::CHUNK_LEN;
use arrayref::array_ref;
use core::usize;

// Interesting input lengths to run tests on.
pub const TEST_CASES: &[usize] = &[
    0,
    1,
    CHUNK_LEN - 1,
    CHUNK_LEN,
    CHUNK_LEN + 1,
    2 * CHUNK_LEN,
    2 * CHUNK_LEN + 1,
    3 * CHUNK_LEN,
    3 * CHUNK_LEN + 1,
    4 * CHUNK_LEN,
    4 * CHUNK_LEN + 1,
    5 * CHUNK_LEN,
    5 * CHUNK_LEN + 1,
    6 * CHUNK_LEN,
    6 * CHUNK_LEN + 1,
    7 * CHUNK_LEN,
    7 * CHUNK_LEN + 1,
    8 * CHUNK_LEN,
    8 * CHUNK_LEN + 1,
    16 * CHUNK_LEN, // AVX512's bandwidth
    31 * CHUNK_LEN, // 16 + 8 + 4 + 2 + 1
];

pub const TEST_CASES_MAX: usize = 31 * CHUNK_LEN;

pub const TEST_KEY: [u8; crate::KEY_LEN] = *b"whats the Elvish word for friend";

// Paint the input with a repeating byte pattern. We use a cycle length of 251,
// because that's the largets prime number less than 256. This makes it
// unlikely to swapping any two adjacent input blocks or chunks will give the
// same answer.
pub fn paint_test_input(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
}

#[test]
fn test_reference_impl_size() {
    // Because the Rust compiler optimizes struct layout, it's possible that
    // some future version of the compiler will produce a different size. If
    // that happens, we can either disable this test, or test for multiple
    // expected values. For now, the purpose of this test is to make sure we
    // notice if that happens.
    assert_eq!(1848, core::mem::size_of::<reference_impl::Hasher>());
}

#[test]
fn test_offset_words() {
    let offset: u64 = (1 << 32) + 2;
    assert_eq!(crate::offset_low(offset), 2);
    assert_eq!(crate::offset_high(offset), 1);
}

#[test]
fn test_largest_power_of_two_leq() {
    let input_output = &[
        // The zero case is nonsensical, but it does work.
        (0, 1),
        (1, 1),
        (2, 2),
        (3, 2),
        (4, 4),
        (5, 4),
        (6, 4),
        (7, 4),
        (8, 8),
        // the largest possible usize
        (usize::MAX, (usize::MAX >> 1) + 1),
    ];
    for &(input, output) in input_output {
        assert_eq!(
            output,
            crate::largest_power_of_two_leq(input),
            "wrong output for n={}",
            input
        );
    }
}

#[test]
fn test_left_len() {
    let input_output = &[
        (CHUNK_LEN + 1, CHUNK_LEN),
        (2 * CHUNK_LEN - 1, CHUNK_LEN),
        (2 * CHUNK_LEN, CHUNK_LEN),
        (2 * CHUNK_LEN + 1, 2 * CHUNK_LEN),
        (4 * CHUNK_LEN - 1, 2 * CHUNK_LEN),
        (4 * CHUNK_LEN, 2 * CHUNK_LEN),
        (4 * CHUNK_LEN + 1, 4 * CHUNK_LEN),
    ];
    for &(input, output) in input_output {
        assert_eq!(crate::left_len(input), output);
    }
}

#[test]
fn test_compare_reference_impl() {
    const OUT: usize = 303; // more than 64, not a multiple of 4
    let mut input_buf = [0; TEST_CASES_MAX];
    paint_test_input(&mut input_buf);
    for &case in TEST_CASES {
        let input = &input_buf[..case];
        #[cfg(feature = "std")]
        dbg!(case);

        // regular
        let mut reference_hasher = reference_impl::Hasher::new();
        reference_hasher.update(input);
        let mut expected_out = [0; OUT];
        reference_hasher.finalize(&mut expected_out);

        let test_out = crate::hash(input);
        assert_eq!(&test_out, array_ref!(expected_out, 0, 32));

        // keyed
        let mut reference_hasher = reference_impl::Hasher::new_keyed(&TEST_KEY);
        reference_hasher.update(input);
        let mut expected_out = [0; OUT];
        reference_hasher.finalize(&mut expected_out);

        let test_out = crate::hash_keyed(&TEST_KEY, input);
        assert_eq!(&test_out, array_ref!(expected_out, 0, 32));

        // derive_key
        let mut reference_hasher = reference_impl::Hasher::new_derive_key(&TEST_KEY);
        reference_hasher.update(input);
        let mut expected_out = [0; OUT];
        reference_hasher.finalize(&mut expected_out);

        let test_out = crate::derive_key(&TEST_KEY, input);
        assert_eq!(&test_out, array_ref!(expected_out, 0, 32));
    }
}
