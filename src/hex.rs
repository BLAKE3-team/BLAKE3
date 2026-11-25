/// These constant-time-ish algorithms (nothing is really guaranteed to be constant time without
/// dropping down to assembly) are adapted from the `key_to_hex` and `key_from_hex` functions here:
/// https://git.zx2c4.com/wireguard-tools/tree/src/encoding.c?h=v1.0.20260223

const HEX_LETTER_OFFSET: u8 = b'a' - 0xa; // 87
const HEX_NUMBER_ADJUSTMENT: u8 = b'0'.wrapping_sub(HEX_LETTER_OFFSET); // -39 or !38, but unsigned

pub fn nibble_to_hex_constant_time_ish(nibble: u8) -> u8 {
    debug_assert!(nibble < 16);
    //  0 <= nibble <=  9: 0xff
    // 10 <= nibble <= 15: 0x00
    let is_number_mask = ((nibble as u32).wrapping_sub(10) >> 8) as u8;
    (nibble + HEX_LETTER_OFFSET).wrapping_add(is_number_mask & HEX_NUMBER_ADJUSTMENT)
}

#[test]
fn test_nibble_to_hex_constant_time_ish() {
    for i in 0..=9 {
        assert_eq!(b'0' + i, nibble_to_hex_constant_time_ish(i));
    }
    for i in 10..=15 {
        assert_eq!(b'a' + (i - 10), nibble_to_hex_constant_time_ish(i));
    }
}

const ASCII_CASE_BIT: u8 = b'A' ^ b'a'; // 0x20

// The zero-or-positive nibble if the hex digit is valid, otherwise -1.
pub fn hex_to_nibble_constant_time_ish(hex_digit: u8) -> i32 {
    let number = hex_digit.wrapping_sub(b'0') as i32; // 0-9 if a number, otherwise larger
    let is_number_mask = (number - 10) >> 8; // 0xffffffff if a number, otherwise 0x00000000
    // Setting the case bit normalizes 'A' to 'a' and leaves 'a' unchanged.
    let letter = (hex_digit | ASCII_CASE_BIT).wrapping_sub(b'a') as i32; // 0-5 if a letter, otherwise larger
    let is_letter_mask = (letter - 6) >> 8; // 0xffffffff if a letter, otherwise 0x00000000
    let is_invalid_mask = !(is_number_mask | is_letter_mask);
    (number & is_number_mask) | ((letter + 10) & is_letter_mask) | is_invalid_mask
}

#[test]
fn test_hex_to_nibble_constant_time_ish() {
    for i in 0x00..=0xff {
        let expected = match i {
            b'0'..=b'9' => (i - b'0') as i32,
            b'A'..=b'F' => (i - b'A' + 10) as i32,
            b'a'..=b'f' => (i - b'a' + 10) as i32,
            _ => -1,
        };
        assert_eq!(hex_to_nibble_constant_time_ish(i), expected);
    }
}
