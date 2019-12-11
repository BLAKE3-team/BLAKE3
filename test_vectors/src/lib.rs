use blake3::CHUNK_LEN;
use serde::{Deserialize, Serialize};

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

pub const TEST_KEY: &[u8; blake3::KEY_LEN] = b"whats the Elvish word for friend";

// Paint the input with a repeating byte pattern. We use a cycle length of 251,
// because that's the largets prime number less than 256. This makes it
// unlikely to swapping any two adjacent input blocks or chunks will give the
// same answer.
pub fn paint_test_input(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cases {
    pub _comment: String,
    pub key: String,
    pub cases: Vec<Case>,
}

#[derive(Serialize, Deserialize)]
pub struct Case {
    pub input_len: usize,
    pub hash: String,
    pub keyed_hash: String,
    pub derive_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    fn test_reference_impl_all_at_once(
        key: &[u8; blake3::KEY_LEN],
        input: &[u8],
        expected_hash: &[u8],
        expected_keyed_hash: &[u8],
        expected_derive_key: &[u8],
    ) {
        let mut out = vec![0; expected_hash.len()];
        let mut hasher = reference_impl::Hasher::new();
        hasher.update(input);
        hasher.finalize(&mut out);
        assert_eq!(expected_hash, &out[..]);

        let mut out = vec![0; expected_keyed_hash.len()];
        let mut hasher = reference_impl::Hasher::new_keyed(key);
        hasher.update(input);
        hasher.finalize(&mut out);
        assert_eq!(expected_keyed_hash, &out[..]);

        let mut out = vec![0; expected_derive_key.len()];
        let mut hasher = reference_impl::Hasher::new_derive_key(key);
        hasher.update(input);
        hasher.finalize(&mut out);
        assert_eq!(expected_derive_key, &out[..]);
    }

    fn test_reference_impl_one_at_a_time(
        key: &[u8; blake3::KEY_LEN],
        input: &[u8],
        expected_hash: &[u8],
        expected_keyed_hash: &[u8],
        expected_derive_key: &[u8],
    ) {
        let mut out = vec![0; expected_hash.len()];
        let mut hasher = reference_impl::Hasher::new();
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize(&mut out);
        assert_eq!(expected_hash, &out[..]);

        let mut out = vec![0; expected_keyed_hash.len()];
        let mut hasher = reference_impl::Hasher::new_keyed(key);
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize(&mut out);
        assert_eq!(expected_keyed_hash, &out[..]);

        let mut out = vec![0; expected_derive_key.len()];
        let mut hasher = reference_impl::Hasher::new_derive_key(key);
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize(&mut out);
        assert_eq!(expected_derive_key, &out[..]);
    }

    fn test_incremental_all_at_once(
        key: &[u8; blake3::KEY_LEN],
        input: &[u8],
        expected_hash: &[u8],
        expected_keyed_hash: &[u8],
        expected_derive_key: &[u8],
    ) {
        let mut out = vec![0; expected_hash.len()];
        let mut hasher = blake3::Hasher::new();
        hasher.update(input);
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_hash, &out[..]);
        assert_eq!(&expected_hash[..32], hasher.finalize().as_bytes());

        let mut out = vec![0; expected_keyed_hash.len()];
        let mut hasher = blake3::Hasher::new_keyed(key);
        hasher.update(input);
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_keyed_hash, &out[..]);
        assert_eq!(&expected_keyed_hash[..32], hasher.finalize().as_bytes());

        let mut out = vec![0; expected_derive_key.len()];
        let mut hasher = blake3::Hasher::new_derive_key(key);
        hasher.update(input);
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_derive_key, &out[..]);
        assert_eq!(&expected_derive_key[..32], hasher.finalize().as_bytes());
    }

    fn test_incremental_one_at_a_time(
        key: &[u8; blake3::KEY_LEN],
        input: &[u8],
        expected_hash: &[u8],
        expected_keyed_hash: &[u8],
        expected_derive_key: &[u8],
    ) {
        let mut out = vec![0; expected_hash.len()];
        let mut hasher = blake3::Hasher::new();
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_hash, &out[..]);
        assert_eq!(&expected_hash[..32], hasher.finalize().as_bytes());

        let mut out = vec![0; expected_keyed_hash.len()];
        let mut hasher = blake3::Hasher::new_keyed(key);
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_keyed_hash, &out[..]);
        assert_eq!(&expected_keyed_hash[..32], hasher.finalize().as_bytes());

        let mut out = vec![0; expected_derive_key.len()];
        let mut hasher = blake3::Hasher::new_derive_key(key);
        for &b in input {
            hasher.update(&[b]);
        }
        hasher.finalize_xof(&mut out);
        assert_eq!(expected_derive_key, &out[..]);
        assert_eq!(&expected_derive_key[..32], hasher.finalize().as_bytes());
    }

    fn test_recursive(
        key: &[u8; blake3::KEY_LEN],
        input: &[u8],
        expected_hash: &[u8],
        expected_keyed_hash: &[u8],
        expected_derive_key: &[u8],
    ) {
        assert_eq!(&expected_hash[..32], blake3::hash(input).as_bytes());
        assert_eq!(
            &expected_keyed_hash[..32],
            blake3::keyed_hash(key, input).as_bytes()
        );
        assert_eq!(
            &expected_derive_key[..32],
            blake3::derive_key(key, input).as_bytes()
        );
    }

    #[test]
    fn run_test_vectors() -> Result<(), Box<dyn std::error::Error>> {
        let test_vectors_file_path = "./test_vectors.json";
        let test_vectors_json = std::fs::read_to_string(test_vectors_file_path)?;
        let cases: Cases = serde_json::from_str(&test_vectors_json)?;
        let key: &[u8; blake3::KEY_LEN] = cases.key.as_bytes().try_into()?;
        for case in &cases.cases {
            let mut input = vec![0; case.input_len];
            paint_test_input(&mut input);
            let expected_hash = hex::decode(&case.hash)?;
            let expected_keyed_hash = hex::decode(&case.keyed_hash)?;
            let expected_derive_key = hex::decode(&case.derive_key)?;

            test_reference_impl_all_at_once(
                key,
                &input,
                &expected_hash,
                &expected_keyed_hash,
                &expected_derive_key,
            );

            test_reference_impl_one_at_a_time(
                key,
                &input,
                &expected_hash,
                &expected_keyed_hash,
                &expected_derive_key,
            );

            test_incremental_all_at_once(
                key,
                &input,
                &expected_hash,
                &expected_keyed_hash,
                &expected_derive_key,
            );

            test_incremental_one_at_a_time(
                key,
                &input,
                &expected_hash,
                &expected_keyed_hash,
                &expected_derive_key,
            );

            test_recursive(
                key,
                &input,
                &expected_hash,
                &expected_keyed_hash,
                &expected_derive_key,
            );
        }
        Ok(())
    }
}
