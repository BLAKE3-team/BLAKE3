// A non-multiple of 4 is important, since one possible bug is to fail to emit
// partial words.
const OUTPUT_LEN: usize = 2 * blake3::BLOCK_LEN + 3;

fn main() {
    let mut cases = Vec::new();
    for &input_len in test_vectors::TEST_CASES {
        let mut input = vec![0; input_len];
        test_vectors::paint_test_input(&mut input);

        let mut hash_out = [0; OUTPUT_LEN];
        blake3::Hasher::new()
            .update(&input)
            .finalize_xof(&mut hash_out);

        let mut keyed_hash_out = [0; OUTPUT_LEN];
        blake3::Hasher::new_keyed(test_vectors::TEST_KEY)
            .update(&input)
            .finalize_xof(&mut keyed_hash_out);

        let mut derive_key_out = [0; OUTPUT_LEN];
        blake3::Hasher::new_derive_key(test_vectors::TEST_KEY)
            .update(&input)
            .finalize_xof(&mut derive_key_out);

        cases.push(test_vectors::Case {
            input_len,
            hash: hex::encode(&hash_out[..]),
            keyed_hash: hex::encode(&keyed_hash_out[..]),
            derive_key: hex::encode(&derive_key_out[..]),
        });
    }

    let output = serde_json::to_string_pretty(&test_vectors::Cases {
        _comment: "Each test is an input length and three outputs, one for each of the hash, keyed_hash, and derive_key modes. The input in each case is filled with a 251-byte-long repeating pattern: 0, 1, 2, ..., 249, 250, 0, 1, ... The key used with keyed_hash and derive_key is the 32-byte ASCII string given below. Outputs are encoded as hexadecimal. Each case is an extended output, and implementations should also check that the first 32 bytes match their default-length output.".to_string(),
        key: std::str::from_utf8(test_vectors::TEST_KEY).unwrap().to_string(),
        cases,
    }).unwrap();

    println!("{}", &output);
}
