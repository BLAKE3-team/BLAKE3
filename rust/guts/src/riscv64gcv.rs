//! This implementation currently assumes riscv64gcv_zbb_zvbb. Zvbb in particular ("Vector
//! Bit-manipulation used in Cryptography") is a bleeding-edge extension that was only frozen a few
//! weeks ago at the time I'm writing this comment. Compiling and testing this code currently
//! requires quite a lot of effort, including building Clang from master and building QEMU from a
//! custom branch. Please don't expect this code to be usable on real hardware for some time.

use crate::{CVBytes, Implementation, CHUNK_LEN};

// NOTE: Keep this in sync with the same constant in assembly.
pub(crate) const MAX_SIMD_DEGREE: usize = 16;

extern "C" {
    fn blake3_guts_riscv64gcv_degree() -> usize;
    fn blake3_guts_riscv64gcv_hash_chunks(
        input: *const u8,
        input_len: usize,
        key: *const CVBytes,
        counter: u64,
        flags: u32,
        transposed_output: *mut u32,
    );
    fn blake3_guts_riscv64gcv_hash_parents(
        transposed_input: *const u32,
        num_parents: usize,
        key: *const CVBytes,
        flags: u32,
        transposed_output: *mut u32,
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
    if input_len % CHUNK_LEN == 0 {
        blake3_guts_riscv64gcv_hash_chunks(
            input,
            input_len,
            key,
            counter,
            flags,
            transposed_output,
        );
    } else {
        crate::portable::hash_chunks(input, input_len, key, counter, flags, transposed_output);
    }
}

pub fn implementation() -> Implementation {
    Implementation::new(
        blake3_guts_riscv64gcv_degree,
        crate::portable::compress,
        hash_chunks,
        blake3_guts_riscv64gcv_hash_parents,
        crate::portable::xof,
        crate::portable::xof_xor,
        crate::portable::universal_hash,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress_vs_portable() {
        crate::test::test_compress_vs_portable(&implementation());
    }

    #[test]
    fn test_compress_vs_reference() {
        crate::test::test_compress_vs_reference(&implementation());
    }

    #[test]
    fn test_hash_chunks_vs_portable() {
        crate::test::test_hash_chunks_vs_portable(&implementation());
    }

    #[test]
    fn test_hash_parents_vs_portable() {
        crate::test::test_hash_parents_vs_portable(&implementation());
    }

    #[test]
    fn test_chunks_and_parents_vs_reference() {
        crate::test::test_chunks_and_parents_vs_reference(&implementation());
    }

    #[test]
    fn test_xof_vs_portable() {
        crate::test::test_xof_vs_portable(&implementation());
    }

    #[test]
    fn test_xof_vs_reference() {
        crate::test::test_xof_vs_reference(&implementation());
    }

    #[test]
    fn test_universal_hash_vs_portable() {
        crate::test::test_universal_hash_vs_portable(&implementation());
    }

    #[test]
    fn test_universal_hash_vs_reference() {
        crate::test::test_universal_hash_vs_reference(&implementation());
    }
}
