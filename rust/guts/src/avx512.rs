use crate::{BlockBytes, CVBytes, Implementation, BLOCK_LEN, CHUNK_LEN};

const DEGREE: usize = 16;

extern "C" {
    fn blake3_guts_avx512_compress(
        block: *const BlockBytes,
        block_len: u32,
        cv: *const CVBytes,
        counter: u64,
        flags: u32,
        out: *mut CVBytes,
    );
    fn blake3_guts_avx512_compress_xof(
        block: *const BlockBytes,
        block_len: u32,
        cv: *const CVBytes,
        counter: u64,
        flags: u32,
        out: *mut BlockBytes,
    );
    fn blake3_guts_avx512_hash_chunks_16_exact(
        input: *const u8,
        input_len: usize,
        key: *const CVBytes,
        counter: u64,
        flags: u32,
        transposed_output: *mut u32,
    );
    fn blake3_guts_avx512_hash_parents_16_exact(
        transposed_input: *const u32,
        num_parents: usize,
        key: *const CVBytes,
        flags: u32,
        transposed_output: *mut u32,
    );
    fn blake3_guts_avx512_hash_parents_8_exact(
        transposed_input: *const u32,
        num_parents: usize,
        key: *const CVBytes,
        flags: u32,
        transposed_output: *mut u32,
    );
    fn blake3_guts_avx512_xof_16_exact(
        block: *const BlockBytes,
        block_len: u32,
        cv: *const CVBytes,
        counter: u64,
        flags: u32,
        out: *mut u8,
    );
    fn blake3_guts_avx512_xof_xor_16_exact(
        block: *const BlockBytes,
        block_len: u32,
        cv: *const CVBytes,
        counter: u64,
        flags: u32,
        out: *mut u8,
    );
    fn blake3_guts_avx512_universal_hash_16_exact(
        input: *const u8,
        input_len: usize,
        key: *const CVBytes,
        counter: u64,
        out: *mut [u8; 16],
    );
}

unsafe extern "C" fn hash_chunks(
    input: *const u8,
    input_len: usize,
    key: *const CVBytes,
    counter: u64,
    flags: u32,
    transposed_output: *mut u32,
) {
    debug_assert!(input_len <= 16 * CHUNK_LEN);
    if input_len == 16 * CHUNK_LEN {
        blake3_guts_avx512_hash_chunks_16_exact(
            input,
            0, // unused
            key,
            counter,
            flags,
            transposed_output,
        );
        return;
    }
    crate::hash_chunks_using_compress(
        blake3_guts_avx512_compress,
        input,
        input_len,
        key,
        counter,
        flags,
        transposed_output,
    )
}

unsafe extern "C" fn hash_parents(
    transposed_input: *const u32,
    num_parents: usize,
    key: *const CVBytes,
    flags: u32,
    transposed_output: *mut u32, // may overlap the input
) {
    debug_assert!(num_parents <= 16);
    if num_parents == 16 {
        blake3_guts_avx512_hash_parents_16_exact(
            transposed_input,
            num_parents,
            key,
            flags,
            transposed_output,
        );
        return;
    }
    if num_parents == 8 {
        blake3_guts_avx512_hash_parents_8_exact(
            transposed_input,
            num_parents,
            key,
            flags,
            transposed_output,
        );
        return;
    }
    crate::hash_parents_using_compress(
        blake3_guts_avx512_compress,
        transposed_input,
        num_parents,
        key,
        flags,
        transposed_output,
    )
}

unsafe extern "C" fn xof(
    block: *const BlockBytes,
    block_len: u32,
    cv: *const CVBytes,
    counter: u64,
    flags: u32,
    out: *mut u8,
    out_len: usize,
) {
    debug_assert!(out_len <= 16 * BLOCK_LEN);
    if out_len == 16 * BLOCK_LEN {
        blake3_guts_avx512_xof_16_exact(block, block_len, cv, counter, flags, out);
        return;
    }
    crate::xof_using_compress_xof(
        blake3_guts_avx512_compress_xof,
        block,
        block_len,
        cv,
        counter,
        flags,
        out,
        out_len,
    )
}

unsafe extern "C" fn xof_xor(
    block: *const BlockBytes,
    block_len: u32,
    cv: *const CVBytes,
    counter: u64,
    flags: u32,
    out: *mut u8,
    out_len: usize,
) {
    debug_assert!(out_len <= 16 * BLOCK_LEN);
    if out_len == 16 * BLOCK_LEN {
        blake3_guts_avx512_xof_xor_16_exact(block, block_len, cv, counter, flags, out);
        return;
    }
    crate::xof_xor_using_compress_xof(
        blake3_guts_avx512_compress_xof,
        block,
        block_len,
        cv,
        counter,
        flags,
        out,
        out_len,
    )
}

unsafe extern "C" fn universal_hash(
    input: *const u8,
    input_len: usize,
    key: *const CVBytes,
    counter: u64,
    out: *mut [u8; 16],
) {
    debug_assert!(input_len <= 16 * BLOCK_LEN);
    if input_len == 16 * BLOCK_LEN {
        blake3_guts_avx512_universal_hash_16_exact(input, input_len, key, counter, out);
        return;
    }
    crate::universal_hash_using_compress(
        blake3_guts_avx512_compress,
        input,
        input_len,
        key,
        counter,
        out,
    )
}

fn supported() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_avx512") {
        return false;
    }
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(all(target_feature = "avx512f", target_feature = "avx512vl"))]
    {
        return true;
    }
    // Dynamic check, if std is enabled.
    #[cfg(feature = "std")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512vl") {
            return true;
        }
    }
    false
}

pub fn implementation() -> Option<Implementation> {
    if supported() {
        Some(Implementation::new(
            || DEGREE,
            blake3_guts_avx512_compress,
            hash_chunks,
            hash_parents,
            xof,
            xof_xor,
            universal_hash,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress_vs_portable() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_compress_vs_portable(&implementation);
    }

    #[test]
    fn test_compress_vs_reference() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_compress_vs_reference(&implementation);
    }

    #[test]
    fn test_hash_chunks_vs_portable() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_hash_chunks_vs_portable(&implementation);
    }

    #[test]
    fn test_hash_parents_vs_portable() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_hash_parents_vs_portable(&implementation);
    }

    #[test]
    fn test_chunks_and_parents_vs_reference() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_chunks_and_parents_vs_reference(&implementation);
    }

    #[test]
    fn test_xof_vs_portable() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_xof_vs_portable(&implementation);
    }

    #[test]
    fn test_xof_vs_reference() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_xof_vs_reference(&implementation);
    }

    #[test]
    fn test_universal_hash_vs_portable() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_universal_hash_vs_portable(&implementation);
    }

    #[test]
    fn test_universal_hash_vs_reference() {
        let Some(implementation) = implementation() else { return };
        crate::test::test_universal_hash_vs_reference(&implementation);
    }
}
