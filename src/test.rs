use core::cmp;

pub const TEST_KEY: [u8; crate::KEY_LEN] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

// Paint a byte pattern that won't repeat, so that we don't accidentally
// miss buffer offset bugs.
pub fn paint_test_input(buf: &mut [u8]) {
    let mut offset = 0;
    let mut counter: u32 = 1;
    while offset < buf.len() {
        let bytes = counter.to_le_bytes();
        let take = cmp::min(bytes.len(), buf.len() - offset);
        buf[offset..][..take].copy_from_slice(&bytes[..take]);
        counter += 1;
        offset += take;
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
