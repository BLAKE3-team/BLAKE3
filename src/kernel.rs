use crate::CHUNK_LEN;
use std::arch::{asm, global_asm};

global_asm!(include_str!("../asm/out.S"), options(raw));

extern "C" {
    pub fn blake3_sse2_compress(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
    );
    pub fn blake3_sse41_compress(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
    );
    pub fn blake3_avx512_compress(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
    );
    pub fn blake3_sse2_xof_stream_1(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64],
    );
    pub fn blake3_sse41_xof_stream_1(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64],
    );
    pub fn blake3_avx512_xof_stream_1(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64],
    );
    pub fn blake3_avx2_xof_stream_2(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64 * 2],
    );
    pub fn blake3_avx512_xof_stream_2(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64 * 2],
    );
    pub fn blake3_avx512_xof_stream_4(
        cv: &[u32; 8],
        block: &[u8; 64],
        counter: u64,
        block_len: u32,
        flags: u32,
        out: *mut [u8; 64 * 4],
    );
}

pub type CompressionFn =
    unsafe extern "C" fn(cv: &[u32; 8], block: &[u8; 64], counter: u64, block_len: u32, flags: u32);

pub type XofStreamFn<const N: usize> = unsafe extern "C" fn(
    cv: &[u32; 8],
    block: &[u8; 64],
    counter: u64,
    block_len: u32,
    flags: u32,
    out: *mut [u8; N],
);

#[cfg(test)]
mod test {
    use super::*;

    fn test_compression_function(f: CompressionFn) {
        let mut block = [0; 64];
        let block_len = 53;
        crate::test::paint_test_input(&mut block[..block_len]);
        let counter = u64::MAX - 42;
        let flags = crate::CHUNK_START | crate::CHUNK_END | crate::ROOT;

        let mut expected = *crate::IV;
        crate::platform::Platform::Portable.compress_in_place(
            &mut expected,
            &block,
            block_len as u8,
            counter,
            flags,
        );

        let mut found = *crate::IV;
        unsafe {
            f(&mut found, &block, counter, block_len as u32, flags as u32);
        }

        assert_eq!(expected, found);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_sse2_compress() {
        if !is_x86_feature_detected!("sse2") {
            return;
        }
        test_compression_function(blake3_sse2_compress);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_sse41_compress() {
        if !is_x86_feature_detected!("sse4.1") {
            return;
        }
        test_compression_function(blake3_sse41_compress);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_compress() {
        if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
            return;
        }
        test_compression_function(blake3_avx512_compress);
    }

    fn test_xof_function<const N: usize>(f: XofStreamFn<N>) {
        let mut block = [0; 64];
        let block_len = 53;
        crate::test::paint_test_input(&mut block[..block_len]);
        let counter = u32::MAX as u64;
        let flags = crate::CHUNK_START | crate::CHUNK_END | crate::ROOT;

        let mut expected = [0; N];
        let mut incrementing_counter = counter;
        let mut i = 0;
        assert_eq!(0, N % 64);
        while i < N {
            let out_block: &mut [u8; 64] = (&mut expected[i..][..64]).try_into().unwrap();
            *out_block = crate::platform::Platform::Portable.compress_xof(
                crate::IV,
                &block,
                block_len as u8,
                incrementing_counter,
                flags,
            );
            i += 64;
            incrementing_counter += 1;
        }
        assert_eq!(incrementing_counter, counter + N as u64 / 64);

        let mut found = [0; N];
        unsafe {
            f(
                crate::IV,
                &block,
                counter,
                block_len as u32,
                flags as u32,
                &mut found,
            );
        }

        assert_eq!(expected, found);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_sse2_xof_1() {
        if !is_x86_feature_detected!("sse2") {
            return;
        }
        test_xof_function(blake3_sse2_xof_stream_1);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_sse41_xof_1() {
        if !is_x86_feature_detected!("sse4.1") {
            return;
        }
        test_xof_function(blake3_sse41_xof_stream_1);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_xof_1() {
        if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
            return;
        }
        test_xof_function(blake3_avx512_xof_stream_1);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_xof_2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }
        test_xof_function(blake3_avx2_xof_stream_2);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_xof_2() {
        if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
            return;
        }
        test_xof_function(blake3_avx512_xof_stream_2);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_xof_4() {
        if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vl") {
            return;
        }
        test_xof_function(blake3_avx512_xof_stream_4);
    }
}

global_asm!(
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_kernel_16
    //
    //         ecx: block length
    //         r8d: domain flags
    //   zmm0-zmm7: transposed input CV (which may be the key or the IV)
    //       zmm12: transposed lower order counter words
    //       zmm13: transposed higher order counter words
    //       zmm14: transposed block sizes (all 64)
    //       zmm15: transposed flag words
    // zmm16-zmm31: transposed message vectors
    //
    // This routine overwrites zmm8-zmm11 (the third row of the state) with IV bytes, broadcasts
    // the block length into zmm14, and broadcasts the domain flags into zmm15. This completes the
    // transposed state in zmm0-zmm15. It then executes all 7 rounds of compression, and performs
    // the XOR of the upper half of the state into the lower half (but not the feed-forward). The
    // result is left in zmm0-zmm7.
    // --------------------------------------------------------------------------------------------
    ".p2align 6",
    "BLAKE3_IV0_16:",
    ".long 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667",
    ".long 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667, 0x6A09E667",
    "BLAKE3_IV1_16:",
    ".long 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85",
    ".long 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85, 0xBB67AE85",
    "BLAKE3_IV2_16:",
    ".long 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372",
    ".long 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372, 0x3C6EF372",
    "BLAKE3_IV3_16:",
    ".long 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A",
    ".long 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A, 0xA54FF53A",
    "blake3_avx512_kernel_16:",
    // load IV constants into the third row
    "vmovdqa32  zmm8, zmmword ptr [BLAKE3_IV0_16 + rip]",
    "vmovdqa32  zmm9, zmmword ptr [BLAKE3_IV1_16 + rip]",
    "vmovdqa32 zmm10, zmmword ptr [BLAKE3_IV2_16 + rip]",
    "vmovdqa32 zmm11, zmmword ptr [BLAKE3_IV3_16 + rip]",
    // broadcast the block length
    "vpbroadcastd zmm14, ecx",
    // broadcast the domain flags
    "vpbroadcastd zmm15, r8d",
    // round 1
    "vpaddd  zmm0, zmm0, zmm16",
    "vpaddd  zmm1, zmm1, zmm18",
    "vpaddd  zmm2, zmm2, zmm20",
    "vpaddd  zmm3, zmm3, zmm22",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm17",
    "vpaddd  zmm1, zmm1, zmm19",
    "vpaddd  zmm2, zmm2, zmm21",
    "vpaddd  zmm3, zmm3, zmm23",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm24",
    "vpaddd  zmm1, zmm1, zmm26",
    "vpaddd  zmm2, zmm2, zmm28",
    "vpaddd  zmm3, zmm3, zmm30",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm25",
    "vpaddd  zmm1, zmm1, zmm27",
    "vpaddd  zmm2, zmm2, zmm29",
    "vpaddd  zmm3, zmm3, zmm31",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 2
    "vpaddd  zmm0, zmm0, zmm18",
    "vpaddd  zmm1, zmm1, zmm19",
    "vpaddd  zmm2, zmm2, zmm23",
    "vpaddd  zmm3, zmm3, zmm20",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm22",
    "vpaddd  zmm1, zmm1, zmm26",
    "vpaddd  zmm2, zmm2, zmm16",
    "vpaddd  zmm3, zmm3, zmm29",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm17",
    "vpaddd  zmm1, zmm1, zmm28",
    "vpaddd  zmm2, zmm2, zmm25",
    "vpaddd  zmm3, zmm3, zmm31",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm27",
    "vpaddd  zmm1, zmm1, zmm21",
    "vpaddd  zmm2, zmm2, zmm30",
    "vpaddd  zmm3, zmm3, zmm24",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 3
    "vpaddd  zmm0, zmm0, zmm19",
    "vpaddd  zmm1, zmm1, zmm26",
    "vpaddd  zmm2, zmm2, zmm29",
    "vpaddd  zmm3, zmm3, zmm23",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm20",
    "vpaddd  zmm1, zmm1, zmm28",
    "vpaddd  zmm2, zmm2, zmm18",
    "vpaddd  zmm3, zmm3, zmm30",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm22",
    "vpaddd  zmm1, zmm1, zmm25",
    "vpaddd  zmm2, zmm2, zmm27",
    "vpaddd  zmm3, zmm3, zmm24",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm21",
    "vpaddd  zmm1, zmm1, zmm16",
    "vpaddd  zmm2, zmm2, zmm31",
    "vpaddd  zmm3, zmm3, zmm17",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 4
    "vpaddd  zmm0, zmm0, zmm26",
    "vpaddd  zmm1, zmm1, zmm28",
    "vpaddd  zmm2, zmm2, zmm30",
    "vpaddd  zmm3, zmm3, zmm29",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm23",
    "vpaddd  zmm1, zmm1, zmm25",
    "vpaddd  zmm2, zmm2, zmm19",
    "vpaddd  zmm3, zmm3, zmm31",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm20",
    "vpaddd  zmm1, zmm1, zmm27",
    "vpaddd  zmm2, zmm2, zmm21",
    "vpaddd  zmm3, zmm3, zmm17",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm16",
    "vpaddd  zmm1, zmm1, zmm18",
    "vpaddd  zmm2, zmm2, zmm24",
    "vpaddd  zmm3, zmm3, zmm22",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 5
    "vpaddd  zmm0, zmm0, zmm28",
    "vpaddd  zmm1, zmm1, zmm25",
    "vpaddd  zmm2, zmm2, zmm31",
    "vpaddd  zmm3, zmm3, zmm30",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm29",
    "vpaddd  zmm1, zmm1, zmm27",
    "vpaddd  zmm2, zmm2, zmm26",
    "vpaddd  zmm3, zmm3, zmm24",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm23",
    "vpaddd  zmm1, zmm1, zmm21",
    "vpaddd  zmm2, zmm2, zmm16",
    "vpaddd  zmm3, zmm3, zmm22",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm18",
    "vpaddd  zmm1, zmm1, zmm19",
    "vpaddd  zmm2, zmm2, zmm17",
    "vpaddd  zmm3, zmm3, zmm20",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 6
    "vpaddd  zmm0, zmm0, zmm25",
    "vpaddd  zmm1, zmm1, zmm27",
    "vpaddd  zmm2, zmm2, zmm24",
    "vpaddd  zmm3, zmm3, zmm31",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm30",
    "vpaddd  zmm1, zmm1, zmm21",
    "vpaddd  zmm2, zmm2, zmm28",
    "vpaddd  zmm3, zmm3, zmm17",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm29",
    "vpaddd  zmm1, zmm1, zmm16",
    "vpaddd  zmm2, zmm2, zmm18",
    "vpaddd  zmm3, zmm3, zmm20",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm19",
    "vpaddd  zmm1, zmm1, zmm26",
    "vpaddd  zmm2, zmm2, zmm22",
    "vpaddd  zmm3, zmm3, zmm23",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // round 7
    "vpaddd  zmm0, zmm0, zmm27",
    "vpaddd  zmm1, zmm1, zmm21",
    "vpaddd  zmm2, zmm2, zmm17",
    "vpaddd  zmm3, zmm3, zmm24",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vprord  zmm15, zmm15, 16",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 12",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vpaddd  zmm0, zmm0, zmm31",
    "vpaddd  zmm1, zmm1, zmm16",
    "vpaddd  zmm2, zmm2, zmm25",
    "vpaddd  zmm3, zmm3, zmm22",
    "vpaddd  zmm0, zmm0, zmm4",
    "vpaddd  zmm1, zmm1, zmm5",
    "vpaddd  zmm2, zmm2, zmm6",
    "vpaddd  zmm3, zmm3, zmm7",
    "vpxord  zmm12, zmm12, zmm0",
    "vpxord  zmm13, zmm13, zmm1",
    "vpxord  zmm14, zmm14, zmm2",
    "vpxord  zmm15, zmm15, zmm3",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vprord  zmm15, zmm15, 8",
    "vpaddd  zmm8, zmm8, zmm12",
    "vpaddd  zmm9, zmm9, zmm13",
    "vpaddd  zmm10, zmm10, zmm14",
    "vpaddd  zmm11, zmm11, zmm15",
    "vpxord  zmm4, zmm4, zmm8",
    "vpxord  zmm5, zmm5, zmm9",
    "vpxord  zmm6, zmm6, zmm10",
    "vpxord  zmm7, zmm7, zmm11",
    "vprord  zmm4, zmm4, 7",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vpaddd  zmm0, zmm0, zmm30",
    "vpaddd  zmm1, zmm1, zmm18",
    "vpaddd  zmm2, zmm2, zmm19",
    "vpaddd  zmm3, zmm3, zmm23",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 16",
    "vprord  zmm12, zmm12, 16",
    "vprord  zmm13, zmm13, 16",
    "vprord  zmm14, zmm14, 16",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 12",
    "vprord  zmm6, zmm6, 12",
    "vprord  zmm7, zmm7, 12",
    "vprord  zmm4, zmm4, 12",
    "vpaddd  zmm0, zmm0, zmm26",
    "vpaddd  zmm1, zmm1, zmm28",
    "vpaddd  zmm2, zmm2, zmm20",
    "vpaddd  zmm3, zmm3, zmm29",
    "vpaddd  zmm0, zmm0, zmm5",
    "vpaddd  zmm1, zmm1, zmm6",
    "vpaddd  zmm2, zmm2, zmm7",
    "vpaddd  zmm3, zmm3, zmm4",
    "vpxord  zmm15, zmm15, zmm0",
    "vpxord  zmm12, zmm12, zmm1",
    "vpxord  zmm13, zmm13, zmm2",
    "vpxord  zmm14, zmm14, zmm3",
    "vprord  zmm15, zmm15, 8",
    "vprord  zmm12, zmm12, 8",
    "vprord  zmm13, zmm13, 8",
    "vprord  zmm14, zmm14, 8",
    "vpaddd  zmm10, zmm10, zmm15",
    "vpaddd  zmm11, zmm11, zmm12",
    "vpaddd  zmm8, zmm8, zmm13",
    "vpaddd  zmm9, zmm9, zmm14",
    "vpxord  zmm5, zmm5, zmm10",
    "vpxord  zmm6, zmm6, zmm11",
    "vpxord  zmm7, zmm7, zmm8",
    "vpxord  zmm4, zmm4, zmm9",
    "vprord  zmm5, zmm5, 7",
    "vprord  zmm6, zmm6, 7",
    "vprord  zmm7, zmm7, 7",
    "vprord  zmm4, zmm4, 7",
    // final xors
    "vpxord  zmm0, zmm0, zmm8",
    "vpxord  zmm1, zmm1, zmm9",
    "vpxord  zmm2, zmm2, zmm10",
    "vpxord  zmm3, zmm3, zmm11",
    "vpxord  zmm4, zmm4, zmm12",
    "vpxord  zmm5, zmm5, zmm13",
    "vpxord  zmm6, zmm6, zmm14",
    "vpxord  zmm7, zmm7, zmm15",
    "ret",
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_kernel_8
    //
    //         ecx: block length
    //         r8d: domain flags
    //   ymm0-ymm7: transposed input CV (which may be the key or the IV)
    //       ymm12: transposed lower order counter words
    //       ymm13: transposed higher order counter words
    //       ymm14: transposed block sizes (all 64)
    //       ymm15: transposed flag words
    // ymm16-ymm31: transposed message vectors
    //
    // This routine overwrites ymm8-ymm11 (the third row of the state) with IV bytes, broadcasts
    // the block length into ymm14, and broadcasts the domain flags into ymm15. This completes the
    // transposed state in ymm0-ymm15. It then executes all 7 rounds of compression, and performs
    // the XOR of the upper half of the state into the lower half (but not the feed-forward). The
    // result is left in ymm0-ymm7.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_kernel_8:",
    // load IV constants into the third row
    "vmovdqa32  ymm8, ymmword ptr [BLAKE3_IV0_16 + rip]",
    "vmovdqa32  ymm9, ymmword ptr [BLAKE3_IV1_16 + rip]",
    "vmovdqa32 ymm10, ymmword ptr [BLAKE3_IV2_16 + rip]",
    "vmovdqa32 ymm11, ymmword ptr [BLAKE3_IV3_16 + rip]",
    // broadcast the block length
    "vpbroadcastd ymm14, ecx",
    // broadcast the domain flags
    "vpbroadcastd ymm15, r8d",
    // round 1
    "vpaddd  ymm0, ymm0, ymm16",
    "vpaddd  ymm1, ymm1, ymm18",
    "vpaddd  ymm2, ymm2, ymm20",
    "vpaddd  ymm3, ymm3, ymm22",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm17",
    "vpaddd  ymm1, ymm1, ymm19",
    "vpaddd  ymm2, ymm2, ymm21",
    "vpaddd  ymm3, ymm3, ymm23",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm24",
    "vpaddd  ymm1, ymm1, ymm26",
    "vpaddd  ymm2, ymm2, ymm28",
    "vpaddd  ymm3, ymm3, ymm30",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm25",
    "vpaddd  ymm1, ymm1, ymm27",
    "vpaddd  ymm2, ymm2, ymm29",
    "vpaddd  ymm3, ymm3, ymm31",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 2
    "vpaddd  ymm0, ymm0, ymm18",
    "vpaddd  ymm1, ymm1, ymm19",
    "vpaddd  ymm2, ymm2, ymm23",
    "vpaddd  ymm3, ymm3, ymm20",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm22",
    "vpaddd  ymm1, ymm1, ymm26",
    "vpaddd  ymm2, ymm2, ymm16",
    "vpaddd  ymm3, ymm3, ymm29",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm17",
    "vpaddd  ymm1, ymm1, ymm28",
    "vpaddd  ymm2, ymm2, ymm25",
    "vpaddd  ymm3, ymm3, ymm31",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm27",
    "vpaddd  ymm1, ymm1, ymm21",
    "vpaddd  ymm2, ymm2, ymm30",
    "vpaddd  ymm3, ymm3, ymm24",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 3
    "vpaddd  ymm0, ymm0, ymm19",
    "vpaddd  ymm1, ymm1, ymm26",
    "vpaddd  ymm2, ymm2, ymm29",
    "vpaddd  ymm3, ymm3, ymm23",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm20",
    "vpaddd  ymm1, ymm1, ymm28",
    "vpaddd  ymm2, ymm2, ymm18",
    "vpaddd  ymm3, ymm3, ymm30",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm22",
    "vpaddd  ymm1, ymm1, ymm25",
    "vpaddd  ymm2, ymm2, ymm27",
    "vpaddd  ymm3, ymm3, ymm24",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm21",
    "vpaddd  ymm1, ymm1, ymm16",
    "vpaddd  ymm2, ymm2, ymm31",
    "vpaddd  ymm3, ymm3, ymm17",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 4
    "vpaddd  ymm0, ymm0, ymm26",
    "vpaddd  ymm1, ymm1, ymm28",
    "vpaddd  ymm2, ymm2, ymm30",
    "vpaddd  ymm3, ymm3, ymm29",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm23",
    "vpaddd  ymm1, ymm1, ymm25",
    "vpaddd  ymm2, ymm2, ymm19",
    "vpaddd  ymm3, ymm3, ymm31",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm20",
    "vpaddd  ymm1, ymm1, ymm27",
    "vpaddd  ymm2, ymm2, ymm21",
    "vpaddd  ymm3, ymm3, ymm17",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm16",
    "vpaddd  ymm1, ymm1, ymm18",
    "vpaddd  ymm2, ymm2, ymm24",
    "vpaddd  ymm3, ymm3, ymm22",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 5
    "vpaddd  ymm0, ymm0, ymm28",
    "vpaddd  ymm1, ymm1, ymm25",
    "vpaddd  ymm2, ymm2, ymm31",
    "vpaddd  ymm3, ymm3, ymm30",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm29",
    "vpaddd  ymm1, ymm1, ymm27",
    "vpaddd  ymm2, ymm2, ymm26",
    "vpaddd  ymm3, ymm3, ymm24",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm23",
    "vpaddd  ymm1, ymm1, ymm21",
    "vpaddd  ymm2, ymm2, ymm16",
    "vpaddd  ymm3, ymm3, ymm22",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm18",
    "vpaddd  ymm1, ymm1, ymm19",
    "vpaddd  ymm2, ymm2, ymm17",
    "vpaddd  ymm3, ymm3, ymm20",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 6
    "vpaddd  ymm0, ymm0, ymm25",
    "vpaddd  ymm1, ymm1, ymm27",
    "vpaddd  ymm2, ymm2, ymm24",
    "vpaddd  ymm3, ymm3, ymm31",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm30",
    "vpaddd  ymm1, ymm1, ymm21",
    "vpaddd  ymm2, ymm2, ymm28",
    "vpaddd  ymm3, ymm3, ymm17",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm29",
    "vpaddd  ymm1, ymm1, ymm16",
    "vpaddd  ymm2, ymm2, ymm18",
    "vpaddd  ymm3, ymm3, ymm20",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm19",
    "vpaddd  ymm1, ymm1, ymm26",
    "vpaddd  ymm2, ymm2, ymm22",
    "vpaddd  ymm3, ymm3, ymm23",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // round 7
    "vpaddd  ymm0, ymm0, ymm27",
    "vpaddd  ymm1, ymm1, ymm21",
    "vpaddd  ymm2, ymm2, ymm17",
    "vpaddd  ymm3, ymm3, ymm24",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vprord  ymm15, ymm15, 16",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 12",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vpaddd  ymm0, ymm0, ymm31",
    "vpaddd  ymm1, ymm1, ymm16",
    "vpaddd  ymm2, ymm2, ymm25",
    "vpaddd  ymm3, ymm3, ymm22",
    "vpaddd  ymm0, ymm0, ymm4",
    "vpaddd  ymm1, ymm1, ymm5",
    "vpaddd  ymm2, ymm2, ymm6",
    "vpaddd  ymm3, ymm3, ymm7",
    "vpxord  ymm12, ymm12, ymm0",
    "vpxord  ymm13, ymm13, ymm1",
    "vpxord  ymm14, ymm14, ymm2",
    "vpxord  ymm15, ymm15, ymm3",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vprord  ymm15, ymm15, 8",
    "vpaddd  ymm8, ymm8, ymm12",
    "vpaddd  ymm9, ymm9, ymm13",
    "vpaddd  ymm10, ymm10, ymm14",
    "vpaddd  ymm11, ymm11, ymm15",
    "vpxord  ymm4, ymm4, ymm8",
    "vpxord  ymm5, ymm5, ymm9",
    "vpxord  ymm6, ymm6, ymm10",
    "vpxord  ymm7, ymm7, ymm11",
    "vprord  ymm4, ymm4, 7",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vpaddd  ymm0, ymm0, ymm30",
    "vpaddd  ymm1, ymm1, ymm18",
    "vpaddd  ymm2, ymm2, ymm19",
    "vpaddd  ymm3, ymm3, ymm23",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 16",
    "vprord  ymm12, ymm12, 16",
    "vprord  ymm13, ymm13, 16",
    "vprord  ymm14, ymm14, 16",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 12",
    "vprord  ymm6, ymm6, 12",
    "vprord  ymm7, ymm7, 12",
    "vprord  ymm4, ymm4, 12",
    "vpaddd  ymm0, ymm0, ymm26",
    "vpaddd  ymm1, ymm1, ymm28",
    "vpaddd  ymm2, ymm2, ymm20",
    "vpaddd  ymm3, ymm3, ymm29",
    "vpaddd  ymm0, ymm0, ymm5",
    "vpaddd  ymm1, ymm1, ymm6",
    "vpaddd  ymm2, ymm2, ymm7",
    "vpaddd  ymm3, ymm3, ymm4",
    "vpxord  ymm15, ymm15, ymm0",
    "vpxord  ymm12, ymm12, ymm1",
    "vpxord  ymm13, ymm13, ymm2",
    "vpxord  ymm14, ymm14, ymm3",
    "vprord  ymm15, ymm15, 8",
    "vprord  ymm12, ymm12, 8",
    "vprord  ymm13, ymm13, 8",
    "vprord  ymm14, ymm14, 8",
    "vpaddd  ymm10, ymm10, ymm15",
    "vpaddd  ymm11, ymm11, ymm12",
    "vpaddd  ymm8, ymm8, ymm13",
    "vpaddd  ymm9, ymm9, ymm14",
    "vpxord  ymm5, ymm5, ymm10",
    "vpxord  ymm6, ymm6, ymm11",
    "vpxord  ymm7, ymm7, ymm8",
    "vpxord  ymm4, ymm4, ymm9",
    "vprord  ymm5, ymm5, 7",
    "vprord  ymm6, ymm6, 7",
    "vprord  ymm7, ymm7, 7",
    "vprord  ymm4, ymm4, 7",
    // final xors
    "vpxord  ymm0, ymm0, ymm8",
    "vpxord  ymm1, ymm1, ymm9",
    "vpxord  ymm2, ymm2, ymm10",
    "vpxord  ymm3, ymm3, ymm11",
    "vpxord  ymm4, ymm4, ymm12",
    "vpxord  ymm5, ymm5, ymm13",
    "vpxord  ymm6, ymm6, ymm14",
    "vpxord  ymm7, ymm7, ymm15",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_blocks_16
    //
    // zmm0-zmm7: incoming CV
    // rdi: pointer to first message block in rdi, subsequent blocks offset by 1024 bytes each
    // rsi: [unused]
    // rdx: pointer to two 64-byte aligned vectors, counter-low followed by counter-high
    // ecx: block len
    // r8d: flags (other than CHUNK_START and CHUNK_END)
    //
    // This routine loads and transposes message words, populates the rest of the state registers,
    // and invokes blake3_avx512_kernel_16.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_blocks_16:",
    // Load and transpose the message words. Because operations that cross 128-bit lanes are
    // relatively expensive, we split each 512-bit load into four 128-bit loads. This results in
    // vectors like:
    //
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    //
    // Here a, b, c and so on are the 1024-byte-strided blocks provided by the caller,
    // and *0, *1, *2, and so on represent the consecutive 32-bit words of each block. Our goal in
    // transposition is to produce the vectors (a0, b0, c0, ...), (a1, b1, c1, ...), and so on.
    //
    // After the loads, we need to do two interleaving passes. First we interleave 32-bit words.
    // This produces vectors like:
    //
    // a0, b0, a1, b1, e0, f0, e1, f1, i0, j0, i1, j1, m0, n0, m1, n1
    //
    // Finally we interleave 64-bit words. This gives us our goal, which is vectors like:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
    //
    // The interleavings can be done mostly in place, but the first interleaving requires a single
    // scratch vector, and the second interleaving requires two scratch vectors, for a total of
    // three scratch vectors needed. Thus we load each of the message vectors three register
    // positions "higher" than its final destination. We want the transposed results to reside in
    // zmm16-zmm31, so we initially load into zmm19-"zmm34" (except zmm32-zmm34 don't exist, so we
    // substitute zmm13-zmm15 for this range).
    "vmovdqu32    xmm19,        xmmword ptr [rdi +  0 * 16 +  0 * 1024]",
    "vinserti32x4 zmm19, zmm19, xmmword ptr [rdi +  0 * 16 +  4 * 1024], 1",
    "vinserti32x4 zmm19, zmm19, xmmword ptr [rdi +  0 * 16 +  8 * 1024], 2",
    "vinserti32x4 zmm19, zmm19, xmmword ptr [rdi +  0 * 16 + 12 * 1024], 3",
    "vmovdqu32    xmm20,        xmmword ptr [rdi +  0 * 16 +  1 * 1024]",
    "vinserti32x4 zmm20, zmm20, xmmword ptr [rdi +  0 * 16 +  5 * 1024], 1",
    "vinserti32x4 zmm20, zmm20, xmmword ptr [rdi +  0 * 16 +  9 * 1024], 2",
    "vinserti32x4 zmm20, zmm20, xmmword ptr [rdi +  0 * 16 + 13 * 1024], 3",
    "vpunpckldq   zmm18, zmm19, zmm20",
    "vpunpckhdq   zmm19, zmm19, zmm20",
    "vmovdqu32    xmm21,        xmmword ptr [rdi +  0 * 16 +  2 * 1024]",
    "vinserti32x4 zmm21, zmm21, xmmword ptr [rdi +  0 * 16 +  6 * 1024], 1",
    "vinserti32x4 zmm21, zmm21, xmmword ptr [rdi +  0 * 16 + 10 * 1024], 2",
    "vinserti32x4 zmm21, zmm21, xmmword ptr [rdi +  0 * 16 + 14 * 1024], 3",
    "vmovdqu32    xmm22,        xmmword ptr [rdi +  0 * 16 +  3 * 1024]",
    "vinserti32x4 zmm22, zmm22, xmmword ptr [rdi +  0 * 16 +  7 * 1024], 1",
    "vinserti32x4 zmm22, zmm22, xmmword ptr [rdi +  0 * 16 + 11 * 1024], 2",
    "vinserti32x4 zmm22, zmm22, xmmword ptr [rdi +  0 * 16 + 15 * 1024], 3",
    "vpunpckldq   zmm20, zmm21, zmm22",
    "vpunpckhdq   zmm21, zmm21, zmm22",
    "vpunpcklqdq  zmm16, zmm18, zmm20",
    "vpunpckhqdq  zmm17, zmm18, zmm20",
    "vpunpcklqdq  zmm18, zmm19, zmm21",
    "vpunpckhqdq  zmm19, zmm19, zmm21",
    "vmovdqu32    xmm23,        xmmword ptr [rdi +  1 * 16 +  0 * 1024]",
    "vinserti32x4 zmm23, zmm23, xmmword ptr [rdi +  1 * 16 +  4 * 1024], 1",
    "vinserti32x4 zmm23, zmm23, xmmword ptr [rdi +  1 * 16 +  8 * 1024], 2",
    "vinserti32x4 zmm23, zmm23, xmmword ptr [rdi +  1 * 16 + 12 * 1024], 3",
    "vmovdqu32    xmm24,        xmmword ptr [rdi +  1 * 16 +  1 * 1024]",
    "vinserti32x4 zmm24, zmm24, xmmword ptr [rdi +  1 * 16 +  5 * 1024], 1",
    "vinserti32x4 zmm24, zmm24, xmmword ptr [rdi +  1 * 16 +  9 * 1024], 2",
    "vinserti32x4 zmm24, zmm24, xmmword ptr [rdi +  1 * 16 + 13 * 1024], 3",
    "vpunpckldq   zmm22, zmm23, zmm24",
    "vpunpckhdq   zmm23, zmm23, zmm24",
    "vmovdqu32    xmm25,        xmmword ptr [rdi +  1 * 16 +  2 * 1024]",
    "vinserti32x4 zmm25, zmm25, xmmword ptr [rdi +  1 * 16 +  6 * 1024], 1",
    "vinserti32x4 zmm25, zmm25, xmmword ptr [rdi +  1 * 16 + 10 * 1024], 2",
    "vinserti32x4 zmm25, zmm25, xmmword ptr [rdi +  1 * 16 + 14 * 1024], 3",
    "vmovdqu32    xmm26,        xmmword ptr [rdi +  1 * 16 +  3 * 1024]",
    "vinserti32x4 zmm26, zmm26, xmmword ptr [rdi +  1 * 16 +  7 * 1024], 1",
    "vinserti32x4 zmm26, zmm26, xmmword ptr [rdi +  1 * 16 + 11 * 1024], 2",
    "vinserti32x4 zmm26, zmm26, xmmword ptr [rdi +  1 * 16 + 15 * 1024], 3",
    "vpunpckldq   zmm24, zmm25, zmm26",
    "vpunpckhdq   zmm25, zmm25, zmm26",
    "vpunpcklqdq  zmm20, zmm22, zmm24",
    "vpunpckhqdq  zmm21, zmm22, zmm24",
    "vpunpcklqdq  zmm22, zmm23, zmm25",
    "vpunpckhqdq  zmm23, zmm23, zmm25",
    "vmovdqu32    xmm27,        xmmword ptr [rdi +  2 * 16 +  0 * 1024]",
    "vinserti32x4 zmm27, zmm27, xmmword ptr [rdi +  2 * 16 +  4 * 1024], 1",
    "vinserti32x4 zmm27, zmm27, xmmword ptr [rdi +  2 * 16 +  8 * 1024], 2",
    "vinserti32x4 zmm27, zmm27, xmmword ptr [rdi +  2 * 16 + 12 * 1024], 3",
    "vmovdqu32    xmm28,        xmmword ptr [rdi +  2 * 16 +  1 * 1024]",
    "vinserti32x4 zmm28, zmm28, xmmword ptr [rdi +  2 * 16 +  5 * 1024], 1",
    "vinserti32x4 zmm28, zmm28, xmmword ptr [rdi +  2 * 16 +  9 * 1024], 2",
    "vinserti32x4 zmm28, zmm28, xmmword ptr [rdi +  2 * 16 + 13 * 1024], 3",
    "vpunpckldq   zmm26, zmm27, zmm28",
    "vpunpckhdq   zmm27, zmm27, zmm28",
    "vmovdqu32    xmm29,        xmmword ptr [rdi +  2 * 16 +  2 * 1024]",
    "vinserti32x4 zmm29, zmm29, xmmword ptr [rdi +  2 * 16 +  6 * 1024], 1",
    "vinserti32x4 zmm29, zmm29, xmmword ptr [rdi +  2 * 16 + 10 * 1024], 2",
    "vinserti32x4 zmm29, zmm29, xmmword ptr [rdi +  2 * 16 + 14 * 1024], 3",
    "vmovdqu32    xmm30,        xmmword ptr [rdi +  2 * 16 +  3 * 1024]",
    "vinserti32x4 zmm30, zmm30, xmmword ptr [rdi +  2 * 16 +  7 * 1024], 1",
    "vinserti32x4 zmm30, zmm30, xmmword ptr [rdi +  2 * 16 + 11 * 1024], 2",
    "vinserti32x4 zmm30, zmm30, xmmword ptr [rdi +  2 * 16 + 15 * 1024], 3",
    "vpunpckldq   zmm28, zmm29, zmm30",
    "vpunpckhdq   zmm29, zmm29, zmm30",
    "vpunpcklqdq  zmm24, zmm26, zmm28",
    "vpunpckhqdq  zmm25, zmm26, zmm28",
    "vpunpcklqdq  zmm26, zmm27, zmm29",
    "vpunpckhqdq  zmm27, zmm27, zmm29",
    "vmovdqu32    xmm31,        xmmword ptr [rdi +  3 * 16 +  0 * 1024]",
    "vinserti32x4 zmm31, zmm31, xmmword ptr [rdi +  3 * 16 +  4 * 1024], 1",
    "vinserti32x4 zmm31, zmm31, xmmword ptr [rdi +  3 * 16 +  8 * 1024], 2",
    "vinserti32x4 zmm31, zmm31, xmmword ptr [rdi +  3 * 16 + 12 * 1024], 3",
    // There are no registers "above" zmm31, so for the next twenty operations we use zmm13-zmm15
    // to stand in for zmm32-34, but otherwise the pattern is the same.
    "vmovdqu32    xmm13,        xmmword ptr [rdi +  3 * 16 +  1 * 1024]",
    "vinserti32x4 zmm13, zmm13, xmmword ptr [rdi +  3 * 16 +  5 * 1024], 1",
    "vinserti32x4 zmm13, zmm13, xmmword ptr [rdi +  3 * 16 +  9 * 1024], 2",
    "vinserti32x4 zmm13, zmm13, xmmword ptr [rdi +  3 * 16 + 13 * 1024], 3",
    "vpunpckldq   zmm30, zmm31, zmm13",
    "vpunpckhdq   zmm31, zmm31, zmm13",
    "vmovdqu32    xmm14,        xmmword ptr [rdi +  3 * 16 +  2 * 1024]",
    "vinserti32x4 zmm14, zmm14, xmmword ptr [rdi +  3 * 16 +  6 * 1024], 1",
    "vinserti32x4 zmm14, zmm14, xmmword ptr [rdi +  3 * 16 + 10 * 1024], 2",
    "vinserti32x4 zmm14, zmm14, xmmword ptr [rdi +  3 * 16 + 14 * 1024], 3",
    "vmovdqu32    xmm15,        xmmword ptr [rdi +  3 * 16 +  3 * 1024]",
    "vinserti32x4 zmm15, zmm15, xmmword ptr [rdi +  3 * 16 +  7 * 1024], 1",
    "vinserti32x4 zmm15, zmm15, xmmword ptr [rdi +  3 * 16 + 11 * 1024], 2",
    "vinserti32x4 zmm15, zmm15, xmmword ptr [rdi +  3 * 16 + 15 * 1024], 3",
    "vpunpckldq   zmm13, zmm14, zmm15",
    "vpunpckhdq   zmm14, zmm14, zmm15",
    "vpunpcklqdq  zmm28, zmm30, zmm13",
    "vpunpckhqdq  zmm29, zmm30, zmm13",
    "vpunpcklqdq  zmm30, zmm31, zmm14",
    "vpunpckhqdq  zmm31, zmm31, zmm14",
    // Load the low and high counter words.
    "vmovdqa32 zmm12, zmmword ptr [rdx + 64 * 0]", // counter low
    "vmovdqa32 zmm13, zmmword ptr [rdx + 64 * 1]", // counter high
    // Run the kernel and then exit.
    "jmp blake3_avx512_kernel_16",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_blocks_8
    //
    // ymm0-ymm7: incoming CV
    // rdi: pointer to first message block in rdi, subsequent blocks offset by 1024 bytes each
    // rsi: [unused]
    // rdx: pointer to two 32-byte aligned vectors, counter-low followed by counter-high
    // ecx: block len
    // r8d: flags (other than CHUNK_START and CHUNK_END)
    //
    // This routine loads and transposes message words, populates the rest of the state registers,
    // and invokes blake3_avx512_kernel_8.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_blocks_8:",
    // Load and transpose the message words. Because operations that cross 128-bit lanes are
    // relatively expensive, we split each 256-bit load into two 128-bit loads. This results in
    // vectors like:
    //
    // a0, a1, a2, a3, e0, e1, e2, e3
    //
    // Here a, b, c and so on are the 1024-byte-strided blocks provided by the caller,
    // and *0, *1, *2, and so on represent the consecutive 32-bit words of each block. Our goal in
    // transposition is to produce the vectors (a0, b0, c0, ...), (a1, b1, c1, ...), and so on.
    //
    // After the loads, we need to do two interleaving passes. First we interleave 32-bit words.
    // This produces vectors like:
    //
    // a0, b0, a1, b1, e0, f0, e1, f1
    //
    // Finally we interleave 64-bit words. This gives us our goal, which is vectors like:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0
    //
    // The interleavings can be done mostly in place, but the first interleaving requires a single
    // scratch vector, and the second interleaving requires two scratch vectors, for a total of
    // three scratch vectors needed. Thus we load each of the message vectors three register
    // positions "higher" than its final destination. We want the transposed results to reside in
    // ymm16-ymm31, so we initially load into ymm19-"ymm34" (except ymm32-ymm34 don't exist, so we
    // substitute ymm13-ymm15 for this range).
    "vmovdqu32    xmm19,        xmmword ptr [rdi +  0 * 16 +  0 * 1024]",
    "vinserti32x4 ymm19, ymm19, xmmword ptr [rdi +  0 * 16 +  4 * 1024], 1",
    "vmovdqu32    xmm20,        xmmword ptr [rdi +  0 * 16 +  1 * 1024]",
    "vinserti32x4 ymm20, ymm20, xmmword ptr [rdi +  0 * 16 +  5 * 1024], 1",
    "vpunpckldq   ymm18, ymm19, ymm20",
    "vpunpckhdq   ymm19, ymm19, ymm20",
    "vmovdqu32    xmm21,        xmmword ptr [rdi +  0 * 16 +  2 * 1024]",
    "vinserti32x4 ymm21, ymm21, xmmword ptr [rdi +  0 * 16 +  6 * 1024], 1",
    "vmovdqu32    xmm22,        xmmword ptr [rdi +  0 * 16 +  3 * 1024]",
    "vinserti32x4 ymm22, ymm22, xmmword ptr [rdi +  0 * 16 +  7 * 1024], 1",
    "vpunpckldq   ymm20, ymm21, ymm22",
    "vpunpckhdq   ymm21, ymm21, ymm22",
    "vpunpcklqdq  ymm16, ymm18, ymm20",
    "vpunpckhqdq  ymm17, ymm18, ymm20",
    "vpunpcklqdq  ymm18, ymm19, ymm21",
    "vpunpckhqdq  ymm19, ymm19, ymm21",
    "vmovdqu32    xmm23,        xmmword ptr [rdi +  1 * 16 +  0 * 1024]",
    "vinserti32x4 ymm23, ymm23, xmmword ptr [rdi +  1 * 16 +  4 * 1024], 1",
    "vmovdqu32    xmm24,        xmmword ptr [rdi +  1 * 16 +  1 * 1024]",
    "vinserti32x4 ymm24, ymm24, xmmword ptr [rdi +  1 * 16 +  5 * 1024], 1",
    "vpunpckldq   ymm22, ymm23, ymm24",
    "vpunpckhdq   ymm23, ymm23, ymm24",
    "vmovdqu32    xmm25,        xmmword ptr [rdi +  1 * 16 +  2 * 1024]",
    "vinserti32x4 ymm25, ymm25, xmmword ptr [rdi +  1 * 16 +  6 * 1024], 1",
    "vmovdqu32    xmm26,        xmmword ptr [rdi +  1 * 16 +  3 * 1024]",
    "vinserti32x4 ymm26, ymm26, xmmword ptr [rdi +  1 * 16 +  7 * 1024], 1",
    "vpunpckldq   ymm24, ymm25, ymm26",
    "vpunpckhdq   ymm25, ymm25, ymm26",
    "vpunpcklqdq  ymm20, ymm22, ymm24",
    "vpunpckhqdq  ymm21, ymm22, ymm24",
    "vpunpcklqdq  ymm22, ymm23, ymm25",
    "vpunpckhqdq  ymm23, ymm23, ymm25",
    "vmovdqu32    xmm27,        xmmword ptr [rdi +  2 * 16 +  0 * 1024]",
    "vinserti32x4 ymm27, ymm27, xmmword ptr [rdi +  2 * 16 +  4 * 1024], 1",
    "vmovdqu32    xmm28,        xmmword ptr [rdi +  2 * 16 +  1 * 1024]",
    "vinserti32x4 ymm28, ymm28, xmmword ptr [rdi +  2 * 16 +  5 * 1024], 1",
    "vpunpckldq   ymm26, ymm27, ymm28",
    "vpunpckhdq   ymm27, ymm27, ymm28",
    "vmovdqu32    xmm29,        xmmword ptr [rdi +  2 * 16 +  2 * 1024]",
    "vinserti32x4 ymm29, ymm29, xmmword ptr [rdi +  2 * 16 +  6 * 1024], 1",
    "vmovdqu32    xmm30,        xmmword ptr [rdi +  2 * 16 +  3 * 1024]",
    "vinserti32x4 ymm30, ymm30, xmmword ptr [rdi +  2 * 16 +  7 * 1024], 1",
    "vpunpckldq   ymm28, ymm29, ymm30",
    "vpunpckhdq   ymm29, ymm29, ymm30",
    "vpunpcklqdq  ymm24, ymm26, ymm28",
    "vpunpckhqdq  ymm25, ymm26, ymm28",
    "vpunpcklqdq  ymm26, ymm27, ymm29",
    "vpunpckhqdq  ymm27, ymm27, ymm29",
    "vmovdqu32    xmm31,        xmmword ptr [rdi +  3 * 16 +  0 * 1024]",
    "vinserti32x4 ymm31, ymm31, xmmword ptr [rdi +  3 * 16 +  4 * 1024], 1",
    // There are no registers "above" ymm31, so for the next twenty operations we use ymm13-ymm15
    // to stand in for ymm32-34, but otherwise the pattern is the same.
    "vmovdqu32    xmm13,        xmmword ptr [rdi +  3 * 16 +  1 * 1024]",
    "vinserti32x4 ymm13, ymm13, xmmword ptr [rdi +  3 * 16 +  5 * 1024], 1",
    "vpunpckldq   ymm30, ymm31, ymm13",
    "vpunpckhdq   ymm31, ymm31, ymm13",
    "vmovdqu32    xmm14,        xmmword ptr [rdi +  3 * 16 +  2 * 1024]",
    "vinserti32x4 ymm14, ymm14, xmmword ptr [rdi +  3 * 16 +  6 * 1024], 1",
    "vmovdqu32    xmm15,        xmmword ptr [rdi +  3 * 16 +  3 * 1024]",
    "vinserti32x4 ymm15, ymm15, xmmword ptr [rdi +  3 * 16 +  7 * 1024], 1",
    "vpunpckldq   ymm13, ymm14, ymm15",
    "vpunpckhdq   ymm14, ymm14, ymm15",
    "vpunpcklqdq  ymm28, ymm30, ymm13",
    "vpunpckhqdq  ymm29, ymm30, ymm13",
    "vpunpcklqdq  ymm30, ymm31, ymm14",
    "vpunpckhqdq  ymm31, ymm31, ymm14",
    // Load the low and high counter words.
    "vmovdqa32 ymm12, ymmword ptr [rdx]",      // counter low
    "vmovdqa32 ymm13, ymmword ptr [rdx + 32]", // counter high
    // Run the kernel and then exit.
    "jmp blake3_avx512_kernel_8",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_chunks_16
    //
    // zmm0-zmm31: [clobbered]
    // rdi: pointer to 16 contiguous chunks of 1024 bytes each, unaligned
    // rsi: pointer to the 8-word key, 4-byte aligned
    // rdx: pointer to two 64-byte aligned vectors, counter-low followed by counter-high
    // ecx: [clobbered]
    // r8d: flags (other than CHUNK_START and CHUNK_END)
    //  r9: out pointer to 8x64 bytes, 64-byte aligned
    //
    // This routine broadcasts the key and calls blake3_avx512_blocks_16 for each block, setting
    // CHUNK_START and CHUNK_END for the first and last blocks respectively. The final transposed
    // CVs in zmm0-zmm7 are written to the out pointer.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_chunks_16:",
    // TODO: Prefetches
    // Broadcast the key into zmm0-zmm7.
    "vpbroadcastd zmm0, dword ptr [rsi + 0 * 4]",
    "vpbroadcastd zmm1, dword ptr [rsi + 1 * 4]",
    "vpbroadcastd zmm2, dword ptr [rsi + 2 * 4]",
    "vpbroadcastd zmm3, dword ptr [rsi + 3 * 4]",
    "vpbroadcastd zmm4, dword ptr [rsi + 4 * 4]",
    "vpbroadcastd zmm5, dword ptr [rsi + 5 * 4]",
    "vpbroadcastd zmm6, dword ptr [rsi + 6 * 4]",
    "vpbroadcastd zmm7, dword ptr [rsi + 7 * 4]",
    // The block length is always 64 here.
    "mov ecx, 64",
    // Set the CHUNK_START flag.
    "or r8d, 1",
    // Compress the first block.
    "call blake3_avx512_blocks_16",
    // Clear the CHUNK_START flag.
    "and r8d, 0xFFFFFFFE",
    // Compress the middle fourteen blocks.
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    // Set the CHUNK_END flag.
    "or r8d, 2",
    // Compress the last block.
    "add rdi, 64",
    "call blake3_avx512_blocks_16",
    // Write the output in transposed form and exit.
    "vmovdqa32 zmmword ptr [r9 + 0 * 64], zmm0",
    "vmovdqa32 zmmword ptr [r9 + 1 * 64], zmm1",
    "vmovdqa32 zmmword ptr [r9 + 2 * 64], zmm2",
    "vmovdqa32 zmmword ptr [r9 + 3 * 64], zmm3",
    "vmovdqa32 zmmword ptr [r9 + 4 * 64], zmm4",
    "vmovdqa32 zmmword ptr [r9 + 5 * 64], zmm5",
    "vmovdqa32 zmmword ptr [r9 + 6 * 64], zmm6",
    "vmovdqa32 zmmword ptr [r9 + 7 * 64], zmm7",
    "vzeroupper",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_chunks_8
    //
    // ymm0-ymm31: [clobbered]
    // rdi: pointer to 8 contiguous chunks of 1024 bytes each, unaligned
    // rsi: pointer to the 8-word key, 4-byte aligned
    // rdx: pointer to two 32-byte aligned vectors, counter-low followed by counter-high
    // ecx: [clobbered]
    // r8d: flags (other than CHUNK_START and CHUNK_END)
    //  r9: out pointer to 8x32 bytes, 32-byte aligned
    //
    // This routine broadcasts the key and calls blake3_avx512_blocks_8 for each block, setting
    // CHUNK_START and CHUNK_END for the first and last blocks respectively. The final transposed
    // CVs in ymm0-ymm7 are written to the out pointer.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_chunks_8:",
    // TODO: Prefetches
    // Broadcast the key into ymm0-ymm7.
    "vpbroadcastd ymm0, dword ptr [rsi + 0 * 4]",
    "vpbroadcastd ymm1, dword ptr [rsi + 1 * 4]",
    "vpbroadcastd ymm2, dword ptr [rsi + 2 * 4]",
    "vpbroadcastd ymm3, dword ptr [rsi + 3 * 4]",
    "vpbroadcastd ymm4, dword ptr [rsi + 4 * 4]",
    "vpbroadcastd ymm5, dword ptr [rsi + 5 * 4]",
    "vpbroadcastd ymm6, dword ptr [rsi + 6 * 4]",
    "vpbroadcastd ymm7, dword ptr [rsi + 7 * 4]",
    // The block length is always 64 here.
    "mov ecx, 64",
    // Set the CHUNK_START flag.
    "or r8d, 1",
    // Compress the first block.
    "call blake3_avx512_blocks_8",
    // Clear the CHUNK_START flag.
    "and r8d, 0xFFFFFFFE",
    // Compress the middle fourteen blocks.
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    // Set the CHUNK_END flag.
    "or r8d, 2",
    // Compress the last block.
    "add rdi, 64",
    "call blake3_avx512_blocks_8",
    // Write the output in transposed form and exit.
    "vmovdqa32 ymmword ptr [r9 + 0 * 32], ymm0",
    "vmovdqa32 ymmword ptr [r9 + 1 * 32], ymm1",
    "vmovdqa32 ymmword ptr [r9 + 2 * 32], ymm2",
    "vmovdqa32 ymmword ptr [r9 + 3 * 32], ymm3",
    "vmovdqa32 ymmword ptr [r9 + 4 * 32], ymm4",
    "vmovdqa32 ymmword ptr [r9 + 5 * 32], ymm5",
    "vmovdqa32 ymmword ptr [r9 + 6 * 32], ymm6",
    "vmovdqa32 ymmword ptr [r9 + 7 * 32], ymm7",
    "vzeroupper",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_parents_16
    //
    // zmm0-zmm31: [clobbered]
    // rdi: pointer to the left child CVs, 8 transposed state vectors, 64-byte aligned
    // rsi: pointer to the right child CVs, 8 transposed state vectors, 64-byte aligned
    // rdx: pointer to the 8-word key, 4-byte aligned
    // ecx: [clobbered]
    // r8d: flags (other than PARENT)
    //  r9: out pointer to 8x64 bytes, 64-byte aligned
    //
    // This routine interleaves the input state vectors into message block vectors for a parent
    // compression, broadcasts the key, and calls blake3_avx512_kernel_16 with the PARENT flag set.
    // Note that the input state vectors are in exactly the format produced by two calls to
    // blake3_avx512_chunks_16, and the transposed output written to the out pointer is also in the
    // same format. This keeps transposition overhead to a minimum as we work our way up the tree.
    // --------------------------------------------------------------------------------------------
    ".p2align 6",
    "BLAKE3_AVX512_EVEN_INDEXES:",
    ".long 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30",
    "BLAKE3_AVX512_ODD_INDEXES:",
    ".long 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31",
    "blake3_avx512_parents_16:",
    // The first 8 out of 16 input message vectors, which are the transposed CVs of the first 8
    // children, come in looking like this:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
    //
    // Here, a and b are the chaining values of the leftmost two children. In this parent
    // compression we're about to do, we're going to compress them together, and that means that we
    // need to get a and b into different vector registers. In particular, all of a's words need to
    // wind up in zmm16-zmm23 (the transposed left half of each message block) and all of b's words
    // need to wind up in zmm24-zmm31 (the transposed right half of each message block). So for
    // example we need zmm16 to look like this (where ' indicates the last 8 children):
    //
    // a0, c0, e0, g0, i0, k0, m0, o0, a'0, c'0, e'0, g'0, i'0, k'0, m'0, o'0
    //
    // Use zmm0 and zmm1 to hold the even and odd index values for vpermt2d, and use zmm2 as a
    // scratch register.
    "vmovdqa32 zmm0, zmmword ptr [rip + BLAKE3_AVX512_EVEN_INDEXES]",
    "vmovdqa32 zmm1, zmmword ptr [rip + BLAKE3_AVX512_ODD_INDEXES]",
    "vmovdqa32 zmm16, zmmword ptr [rdi + 0 * 64]",
    "vmovdqa32 zmm24, zmm16",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 0 * 64]",
    "vpermt2d  zmm16, zmm0, zmm2",
    "vpermt2d  zmm24, zmm1, zmm2",
    "vmovdqa32 zmm17, zmmword ptr [rdi + 1 * 64]",
    "vmovdqa32 zmm25, zmm17",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 1 * 64]",
    "vpermt2d  zmm17, zmm0, zmm2",
    "vpermt2d  zmm25, zmm1, zmm2",
    "vmovdqa32 zmm18, zmmword ptr [rdi + 2 * 64]",
    "vmovdqa32 zmm26, zmm18",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 2 * 64]",
    "vpermt2d  zmm18, zmm0, zmm2",
    "vpermt2d  zmm26, zmm1, zmm2",
    "vmovdqa32 zmm19, zmmword ptr [rdi + 3 * 64]",
    "vmovdqa32 zmm27, zmm19",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 3 * 64]",
    "vpermt2d  zmm19, zmm0, zmm2",
    "vpermt2d  zmm27, zmm1, zmm2",
    "vmovdqa32 zmm20, zmmword ptr [rdi + 4 * 64]",
    "vmovdqa32 zmm28, zmm20",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 4 * 64]",
    "vpermt2d  zmm20, zmm0, zmm2",
    "vpermt2d  zmm28, zmm1, zmm2",
    "vmovdqa32 zmm21, zmmword ptr [rdi + 5 * 64]",
    "vmovdqa32 zmm29, zmm21",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 5 * 64]",
    "vpermt2d  zmm21, zmm0, zmm2",
    "vpermt2d  zmm29, zmm1, zmm2",
    "vmovdqa32 zmm22, zmmword ptr [rdi + 6 * 64]",
    "vmovdqa32 zmm30, zmm22",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 6 * 64]",
    "vpermt2d  zmm22, zmm0, zmm2",
    "vpermt2d  zmm30, zmm1, zmm2",
    "vmovdqa32 zmm23, zmmword ptr [rdi + 7 * 64]",
    "vmovdqa32 zmm31, zmm23",
    "vmovdqa32  zmm2, zmmword ptr [rsi + 7 * 64]",
    "vpermt2d  zmm23, zmm0, zmm2",
    "vpermt2d  zmm31, zmm1, zmm2",
    // Broadcast the key into zmm0-zmm7.
    "vpbroadcastd zmm0, dword ptr [rdx + 0 * 4]",
    "vpbroadcastd zmm1, dword ptr [rdx + 1 * 4]",
    "vpbroadcastd zmm2, dword ptr [rdx + 2 * 4]",
    "vpbroadcastd zmm3, dword ptr [rdx + 3 * 4]",
    "vpbroadcastd zmm4, dword ptr [rdx + 4 * 4]",
    "vpbroadcastd zmm5, dword ptr [rdx + 5 * 4]",
    "vpbroadcastd zmm6, dword ptr [rdx + 6 * 4]",
    "vpbroadcastd zmm7, dword ptr [rdx + 7 * 4]",
    // Initialize the low and high counter words.
    "vpxorq zmm12, zmm12, zmm12", // counter low, always zero for parents
    "vpxorq zmm13, zmm13, zmm13", // counter high, always zero for parents
    // The block length is always 64 for parents.
    "mov ecx, 64",
    // Set the PARENT flag.
    "or r8d, 4",
    // Run the kernel.
    "call blake3_avx512_kernel_16",
    // Write the output and exit.
    "vmovdqa32 zmmword ptr [r9 + 0 * 64], zmm0",
    "vmovdqa32 zmmword ptr [r9 + 1 * 64], zmm1",
    "vmovdqa32 zmmword ptr [r9 + 2 * 64], zmm2",
    "vmovdqa32 zmmword ptr [r9 + 3 * 64], zmm3",
    "vmovdqa32 zmmword ptr [r9 + 4 * 64], zmm4",
    "vmovdqa32 zmmword ptr [r9 + 5 * 64], zmm5",
    "vmovdqa32 zmmword ptr [r9 + 6 * 64], zmm6",
    "vmovdqa32 zmmword ptr [r9 + 7 * 64], zmm7",
    "vzeroupper",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_parents_8
    //
    // ymm0-ymm31: [clobbered]
    // rdi: pointer to the left child CVs, 8 transposed state vectors, 32-byte aligned
    // rsi: pointer to the right child CVs, 8 transposed state vectors, 32-byte aligned
    // rdx: pointer to the 8-word key, 4-byte aligned
    // ecx: [clobbered]
    // r8d: flags (other than PARENT)
    //  r9: out pointer to 8x32 bytes, 32-byte aligned
    //
    // This routine interleaves the input state vectors into message block vectors for a parent
    // compression, broadcasts the key, and calls blake3_avx512_kernel_8 with the PARENT flag set.
    // Note that the input state vectors are in exactly the format produced by two calls to
    // blake3_avx512_chunks_8, and the transposed output written to the out pointer is also in the
    // same format. This keeps transposition overhead to a minimum as we work our way up the tree.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_parents_8:",
    // The first 8 out of 16 input message vectors, which are the transposed CVs of the first 8
    // children, come in looking like this:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0
    //
    // Here, a and b are the chaining values of the leftmost two children. In this parent
    // compression we're about to do, we're going to compress them together, and that means that we
    // need to get a and b into different vector registers. In particular, all of a's words need to
    // wind up in ymm16-ymm23 (the transposed left half of each message block) and all of b's words
    // need to wind up in ymm24-ymm31 (the transposed right half of each message block). So for
    // example we need ymm16 to look like this (where ' indicates the last 8 children):
    //
    // a0, c0, e0, g0, a'0, c'0, e'0, g'0
    //
    // Use ymm0 and ymm1 to hold the even and odd index values for vpermt2d, and use ymm2 as a
    // scratch register.
    "vmovdqa32 ymm0, ymmword ptr [rip + BLAKE3_AVX512_EVEN_INDEXES]",
    "vmovdqa32 ymm1, ymmword ptr [rip + BLAKE3_AVX512_ODD_INDEXES]",
    "vmovdqa32 ymm16, ymmword ptr [rdi + 0 * 32]",
    "vmovdqa32 ymm24, ymm16",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 0 * 32]",
    "vpermt2d  ymm16, ymm0, ymm2",
    "vpermt2d  ymm24, ymm1, ymm2",
    "vmovdqa32 ymm17, ymmword ptr [rdi + 1 * 32]",
    "vmovdqa32 ymm25, ymm17",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 1 * 32]",
    "vpermt2d  ymm17, ymm0, ymm2",
    "vpermt2d  ymm25, ymm1, ymm2",
    "vmovdqa32 ymm18, ymmword ptr [rdi + 2 * 32]",
    "vmovdqa32 ymm26, ymm18",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 2 * 32]",
    "vpermt2d  ymm18, ymm0, ymm2",
    "vpermt2d  ymm26, ymm1, ymm2",
    "vmovdqa32 ymm19, ymmword ptr [rdi + 3 * 32]",
    "vmovdqa32 ymm27, ymm19",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 3 * 32]",
    "vpermt2d  ymm19, ymm0, ymm2",
    "vpermt2d  ymm27, ymm1, ymm2",
    "vmovdqa32 ymm20, ymmword ptr [rdi + 4 * 32]",
    "vmovdqa32 ymm28, ymm20",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 4 * 32]",
    "vpermt2d  ymm20, ymm0, ymm2",
    "vpermt2d  ymm28, ymm1, ymm2",
    "vmovdqa32 ymm21, ymmword ptr [rdi + 5 * 32]",
    "vmovdqa32 ymm29, ymm21",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 5 * 32]",
    "vpermt2d  ymm21, ymm0, ymm2",
    "vpermt2d  ymm29, ymm1, ymm2",
    "vmovdqa32 ymm22, ymmword ptr [rdi + 6 * 32]",
    "vmovdqa32 ymm30, ymm22",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 6 * 32]",
    "vpermt2d  ymm22, ymm0, ymm2",
    "vpermt2d  ymm30, ymm1, ymm2",
    "vmovdqa32 ymm23, ymmword ptr [rdi + 7 * 32]",
    "vmovdqa32 ymm31, ymm23",
    "vmovdqa32  ymm2, ymmword ptr [rsi + 7 * 32]",
    "vpermt2d  ymm23, ymm0, ymm2",
    "vpermt2d  ymm31, ymm1, ymm2",
    // Broadcast the key into ymm0-ymm7.
    "vpbroadcastd ymm0, dword ptr [rdx + 0 * 4]",
    "vpbroadcastd ymm1, dword ptr [rdx + 1 * 4]",
    "vpbroadcastd ymm2, dword ptr [rdx + 2 * 4]",
    "vpbroadcastd ymm3, dword ptr [rdx + 3 * 4]",
    "vpbroadcastd ymm4, dword ptr [rdx + 4 * 4]",
    "vpbroadcastd ymm5, dword ptr [rdx + 5 * 4]",
    "vpbroadcastd ymm6, dword ptr [rdx + 6 * 4]",
    "vpbroadcastd ymm7, dword ptr [rdx + 7 * 4]",
    // Initialize the low and high counter words.
    "vpxorq ymm12, ymm12, ymm12", // counter low, always zero for parents
    "vpxorq ymm13, ymm13, ymm13", // counter high, always zero for parents
    // The block length is always 64 for parents.
    "mov ecx, 64",
    // Set the PARENT flag.
    "or r8d, 4",
    // Run the kernel.
    "call blake3_avx512_kernel_8",
    // Write the output and exit.
    "vmovdqa32 ymmword ptr [r9 + 0 * 32], ymm0",
    "vmovdqa32 ymmword ptr [r9 + 1 * 32], ymm1",
    "vmovdqa32 ymmword ptr [r9 + 2 * 32], ymm2",
    "vmovdqa32 ymmword ptr [r9 + 3 * 32], ymm3",
    "vmovdqa32 ymmword ptr [r9 + 4 * 32], ymm4",
    "vmovdqa32 ymmword ptr [r9 + 5 * 32], ymm5",
    "vmovdqa32 ymmword ptr [r9 + 6 * 32], ymm6",
    "vmovdqa32 ymmword ptr [r9 + 7 * 32], ymm7",
    "vzeroupper",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_xof_stream_16
    //
    // zmm0-zmm31: [clobbered]
    // rdi: pointer to the 16-word message block, 4-byte aligned
    // rsi: pointer to the 8-word input CV, 4-byte aligned
    // rdx: pointer to two 64-byte aligned vectors, counter-low followed by counter-high
    // ecx: block length
    // r8d: flags (other than ROOT)
    //  r9: out pointer to 16x64=1024 bytes, unaligned
    //
    // This routine performs the root compression for 16 consecutive output blocks and writes 1024
    // bytes of output to the out pointer.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_xof_stream_16:",
    // Broadcast the input CV into zmm0-zmm7, the first two rows of the state.
    "vpbroadcastd zmm0, dword ptr [rsi + 0 * 4]",
    "vpbroadcastd zmm1, dword ptr [rsi + 1 * 4]",
    "vpbroadcastd zmm2, dword ptr [rsi + 2 * 4]",
    "vpbroadcastd zmm3, dword ptr [rsi + 3 * 4]",
    "vpbroadcastd zmm4, dword ptr [rsi + 4 * 4]",
    "vpbroadcastd zmm5, dword ptr [rsi + 5 * 4]",
    "vpbroadcastd zmm6, dword ptr [rsi + 6 * 4]",
    "vpbroadcastd zmm7, dword ptr [rsi + 7 * 4]",
    // Load the low and high counter words.
    "vmovdqa32 zmm12, zmmword ptr [rdx + 64 * 0]", // counter low
    "vmovdqa32 zmm13, zmmword ptr [rdx + 64 * 1]", // counter high
    // Set the ROOT flag.
    "or r8d, 8",
    // Broadcast the message block into zmm16-zmm31
    "vpbroadcastd zmm16, dword ptr [rdi + 0 * 4]",
    "vpbroadcastd zmm17, dword ptr [rdi + 1 * 4]",
    "vpbroadcastd zmm18, dword ptr [rdi + 2 * 4]",
    "vpbroadcastd zmm19, dword ptr [rdi + 3 * 4]",
    "vpbroadcastd zmm20, dword ptr [rdi + 4 * 4]",
    "vpbroadcastd zmm21, dword ptr [rdi + 5 * 4]",
    "vpbroadcastd zmm22, dword ptr [rdi + 6 * 4]",
    "vpbroadcastd zmm23, dword ptr [rdi + 7 * 4]",
    "vpbroadcastd zmm24, dword ptr [rdi + 8 * 4]",
    "vpbroadcastd zmm25, dword ptr [rdi + 9 * 4]",
    "vpbroadcastd zmm26, dword ptr [rdi + 10 * 4]",
    "vpbroadcastd zmm27, dword ptr [rdi + 11 * 4]",
    "vpbroadcastd zmm28, dword ptr [rdi + 12 * 4]",
    "vpbroadcastd zmm29, dword ptr [rdi + 13 * 4]",
    "vpbroadcastd zmm30, dword ptr [rdi + 14 * 4]",
    "vpbroadcastd zmm31, dword ptr [rdi + 15 * 4]",
    // Run the kernel.
    "call blake3_avx512_kernel_16",
    // Re-broadcast the input CV and feed it forward into the second half of the state.
    "vpbroadcastd zmm16, dword ptr [rsi + 0 * 4]",
    "vpxord zmm8, zmm8, zmm16",
    "vpbroadcastd zmm17, dword ptr [rsi + 1 * 4]",
    "vpxord zmm9, zmm9, zmm17",
    "vpbroadcastd zmm18, dword ptr [rsi + 2 * 4]",
    "vpxord zmm10, zmm10, zmm18",
    "vpbroadcastd zmm19, dword ptr [rsi + 3 * 4]",
    "vpxord zmm11, zmm11, zmm19",
    "vpbroadcastd zmm20, dword ptr [rsi + 4 * 4]",
    "vpxord zmm12, zmm12, zmm20",
    "vpbroadcastd zmm21, dword ptr [rsi + 5 * 4]",
    "vpxord zmm13, zmm13, zmm21",
    "vpbroadcastd zmm22, dword ptr [rsi + 6 * 4]",
    "vpxord zmm14, zmm14, zmm22",
    "vpbroadcastd zmm23, dword ptr [rsi + 7 * 4]",
    "vpxord zmm15, zmm15, zmm23",
    // zmm0-zmm15 now contain the final extended state vectors, transposed. We need to un-transpose
    // them before we write them out. As with blake3_avx512_blocks_16, we prefer to avoid expensive
    // operations across 128-bit lanes, so we do a couple of interleaving passes and then write out
    // 128 bits at a time.
    //
    // First, interleave 32-bit words. Use zmm16-zmm31 to hold the intermediate results. This
    // takes the input vectors like:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
    //
    // And produces vectors like:
    //
    // a0, a1, b0, b1, e0, e1, g0, g1, i0, i1, k0, k1, m0, m1, o0, o1
    //
    // Then interleave 64-bit words back into zmm0-zmm15, producing vectors like:
    //
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    //
    // Finally, write out each 128-bit group, unaligned.
    "vpunpckldq zmm16, zmm0, zmm1",
    "vpunpckhdq zmm17, zmm0, zmm1",
    "vpunpckldq zmm18, zmm2, zmm3",
    "vpunpckhdq zmm19, zmm2, zmm3",
    "vpunpcklqdq zmm0, zmm16, zmm18",
    "vmovdqu32 xmmword ptr [r9 + 0 * 16], xmm0",
    "vextracti32x4 xmmword ptr [r9 + 16 * 16], zmm0, 1",
    "vextracti32x4 xmmword ptr [r9 + 32 * 16], zmm0, 2",
    "vextracti32x4 xmmword ptr [r9 + 48 * 16], zmm0, 3",
    "vpunpckhqdq zmm1, zmm16, zmm18",
    "vmovdqu32 xmmword ptr [r9 + 4 * 16], xmm1",
    "vextracti32x4 xmmword ptr [r9 + 20 * 16], zmm1, 1",
    "vextracti32x4 xmmword ptr [r9 + 36 * 16], zmm1, 2",
    "vextracti32x4 xmmword ptr [r9 + 52 * 16], zmm1, 3",
    "vpunpcklqdq zmm2, zmm17, zmm19",
    "vmovdqu32 xmmword ptr [r9 + 8 * 16], xmm2",
    "vextracti32x4 xmmword ptr [r9 + 24 * 16], zmm2, 1",
    "vextracti32x4 xmmword ptr [r9 + 40 * 16], zmm2, 2",
    "vextracti32x4 xmmword ptr [r9 + 56 * 16], zmm2, 3",
    "vpunpckhqdq zmm3, zmm17, zmm19",
    "vmovdqu32 xmmword ptr [r9 + 12 * 16], xmm3",
    "vextracti32x4 xmmword ptr [r9 + 28 * 16], zmm3, 1",
    "vextracti32x4 xmmword ptr [r9 + 44 * 16], zmm3, 2",
    "vextracti32x4 xmmword ptr [r9 + 60 * 16], zmm3, 3",
    "vpunpckldq zmm20, zmm4, zmm5",
    "vpunpckhdq zmm21, zmm4, zmm5",
    "vpunpckldq zmm22, zmm6, zmm7",
    "vpunpckhdq zmm23, zmm6, zmm7",
    "vpunpcklqdq zmm4, zmm20, zmm22",
    "vmovdqu32 xmmword ptr [r9 + 1 * 16], xmm4",
    "vextracti32x4 xmmword ptr [r9 + 17 * 16], zmm4, 1",
    "vextracti32x4 xmmword ptr [r9 + 33 * 16], zmm4, 2",
    "vextracti32x4 xmmword ptr [r9 + 49 * 16], zmm4, 3",
    "vpunpckhqdq zmm5, zmm20, zmm22",
    "vmovdqu32 xmmword ptr [r9 + 5 * 16], xmm5",
    "vextracti32x4 xmmword ptr [r9 + 21 * 16], zmm5, 1",
    "vextracti32x4 xmmword ptr [r9 + 37 * 16], zmm5, 2",
    "vextracti32x4 xmmword ptr [r9 + 53 * 16], zmm5, 3",
    "vpunpcklqdq zmm6, zmm21, zmm23",
    "vmovdqu32 xmmword ptr [r9 + 9 * 16], xmm6",
    "vextracti32x4 xmmword ptr [r9 + 25 * 16], zmm6, 1",
    "vextracti32x4 xmmword ptr [r9 + 41 * 16], zmm6, 2",
    "vextracti32x4 xmmword ptr [r9 + 57 * 16], zmm6, 3",
    "vpunpckhqdq zmm7, zmm21, zmm23",
    "vmovdqu32 xmmword ptr [r9 + 13 * 16], xmm7",
    "vextracti32x4 xmmword ptr [r9 + 29 * 16], zmm7, 1",
    "vextracti32x4 xmmword ptr [r9 + 45 * 16], zmm7, 2",
    "vextracti32x4 xmmword ptr [r9 + 61 * 16], zmm7, 3",
    "vpunpckldq zmm24, zmm8, zmm9",
    "vpunpckhdq zmm25, zmm8, zmm9",
    "vpunpckldq zmm26, zmm10, zmm11",
    "vpunpckhdq zmm27, zmm10, zmm11",
    "vpunpcklqdq zmm8, zmm24, zmm26",
    "vmovdqu32 xmmword ptr [r9 + 2 * 16], xmm8",
    "vextracti32x4 xmmword ptr [r9 + 18 * 16], zmm8, 1",
    "vextracti32x4 xmmword ptr [r9 + 34 * 16], zmm8, 2",
    "vextracti32x4 xmmword ptr [r9 + 50 * 16], zmm8, 3",
    "vpunpckhqdq zmm9, zmm24, zmm26",
    "vmovdqu32 xmmword ptr [r9 + 6 * 16], xmm9",
    "vextracti32x4 xmmword ptr [r9 + 22 * 16], zmm9, 1",
    "vextracti32x4 xmmword ptr [r9 + 38 * 16], zmm9, 2",
    "vextracti32x4 xmmword ptr [r9 + 54 * 16], zmm9, 3",
    "vpunpcklqdq zmm10, zmm25, zmm27",
    "vmovdqu32 xmmword ptr [r9 + 10 * 16], xmm10",
    "vextracti32x4 xmmword ptr [r9 + 26 * 16], zmm10, 1",
    "vextracti32x4 xmmword ptr [r9 + 42 * 16], zmm10, 2",
    "vextracti32x4 xmmword ptr [r9 + 58 * 16], zmm10, 3",
    "vpunpckhqdq zmm11, zmm25, zmm27",
    "vmovdqu32 xmmword ptr [r9 + 14 * 16], xmm11",
    "vextracti32x4 xmmword ptr [r9 + 30 * 16], zmm11, 1",
    "vextracti32x4 xmmword ptr [r9 + 46 * 16], zmm11, 2",
    "vextracti32x4 xmmword ptr [r9 + 62 * 16], zmm11, 3",
    "vpunpckldq zmm28, zmm12, zmm13",
    "vpunpckhdq zmm29, zmm12, zmm13",
    "vpunpckldq zmm30, zmm14, zmm15",
    "vpunpckhdq zmm31, zmm14, zmm15",
    "vpunpcklqdq zmm12, zmm28, zmm30",
    "vmovdqu32 xmmword ptr [r9 + 3 * 16], xmm12",
    "vextracti32x4 xmmword ptr [r9 + 19 * 16], zmm12, 1",
    "vextracti32x4 xmmword ptr [r9 + 35 * 16], zmm12, 2",
    "vextracti32x4 xmmword ptr [r9 + 51 * 16], zmm12, 3",
    "vpunpckhqdq zmm13, zmm28, zmm30",
    "vmovdqu32 xmmword ptr [r9 + 7 * 16], xmm13",
    "vextracti32x4 xmmword ptr [r9 + 23 * 16], zmm13, 1",
    "vextracti32x4 xmmword ptr [r9 + 39 * 16], zmm13, 2",
    "vextracti32x4 xmmword ptr [r9 + 55 * 16], zmm13, 3",
    "vpunpcklqdq zmm14, zmm29, zmm31",
    "vmovdqu32 xmmword ptr [r9 + 11 * 16], xmm14",
    "vextracti32x4 xmmword ptr [r9 + 27 * 16], zmm14, 1",
    "vextracti32x4 xmmword ptr [r9 + 43 * 16], zmm14, 2",
    "vextracti32x4 xmmword ptr [r9 + 59 * 16], zmm14, 3",
    "vpunpckhqdq zmm15, zmm29, zmm31",
    "vmovdqu32 xmmword ptr [r9 + 15 * 16], xmm15",
    "vextracti32x4 xmmword ptr [r9 + 31 * 16], zmm15, 1",
    "vextracti32x4 xmmword ptr [r9 + 47 * 16], zmm15, 2",
    "vextracti32x4 xmmword ptr [r9 + 63 * 16], zmm15, 3",
    "vzeroupper",
    "ret",
    //
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_xof_xor_16
    //
    // zmm0-zmm31: [clobbered]
    // rdi: pointer to the 16-word message block, 4-byte aligned
    // rsi: pointer to the 8-word input CV, 4-byte aligned
    // rdx: pointer to two 64-byte aligned vectors, counter-low followed by counter-high
    // ecx: block length
    // r8d: flags (other than ROOT)
    //  r9: inout pointer to 16x64=1024 bytes, unaligned
    //
    // This routine performs the root compression for 16 consecutive output blocks and xor's 1024
    // bytes of output into the inout pointer.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_xof_xor_16:",
    // Broadcast the input CV into zmm0-zmm7, the first two rows of the state.
    "vpbroadcastd zmm0, dword ptr [rsi + 0 * 4]",
    "vpbroadcastd zmm1, dword ptr [rsi + 1 * 4]",
    "vpbroadcastd zmm2, dword ptr [rsi + 2 * 4]",
    "vpbroadcastd zmm3, dword ptr [rsi + 3 * 4]",
    "vpbroadcastd zmm4, dword ptr [rsi + 4 * 4]",
    "vpbroadcastd zmm5, dword ptr [rsi + 5 * 4]",
    "vpbroadcastd zmm6, dword ptr [rsi + 6 * 4]",
    "vpbroadcastd zmm7, dword ptr [rsi + 7 * 4]",
    // Load the low and high counter words.
    "vmovdqa32 zmm12, zmmword ptr [rdx + 64 * 0]", // counter low
    "vmovdqa32 zmm13, zmmword ptr [rdx + 64 * 1]", // counter high
    // Set the ROOT flag.
    "or r8d, 8",
    // Broadcast the message block into zmm16-zmm31
    "vpbroadcastd zmm16, dword ptr [rdi + 0 * 4]",
    "vpbroadcastd zmm17, dword ptr [rdi + 1 * 4]",
    "vpbroadcastd zmm18, dword ptr [rdi + 2 * 4]",
    "vpbroadcastd zmm19, dword ptr [rdi + 3 * 4]",
    "vpbroadcastd zmm20, dword ptr [rdi + 4 * 4]",
    "vpbroadcastd zmm21, dword ptr [rdi + 5 * 4]",
    "vpbroadcastd zmm22, dword ptr [rdi + 6 * 4]",
    "vpbroadcastd zmm23, dword ptr [rdi + 7 * 4]",
    "vpbroadcastd zmm24, dword ptr [rdi + 8 * 4]",
    "vpbroadcastd zmm25, dword ptr [rdi + 9 * 4]",
    "vpbroadcastd zmm26, dword ptr [rdi + 10 * 4]",
    "vpbroadcastd zmm27, dword ptr [rdi + 11 * 4]",
    "vpbroadcastd zmm28, dword ptr [rdi + 12 * 4]",
    "vpbroadcastd zmm29, dword ptr [rdi + 13 * 4]",
    "vpbroadcastd zmm30, dword ptr [rdi + 14 * 4]",
    "vpbroadcastd zmm31, dword ptr [rdi + 15 * 4]",
    // Run the kernel.
    "call blake3_avx512_kernel_16",
    // Re-broadcast the input CV and feed it forward into the second half of the state.
    "vpbroadcastd zmm16, dword ptr [rsi + 0 * 4]",
    "vpxord zmm8, zmm8, zmm16",
    "vpbroadcastd zmm17, dword ptr [rsi + 1 * 4]",
    "vpxord zmm9, zmm9, zmm17",
    "vpbroadcastd zmm18, dword ptr [rsi + 2 * 4]",
    "vpxord zmm10, zmm10, zmm18",
    "vpbroadcastd zmm19, dword ptr [rsi + 3 * 4]",
    "vpxord zmm11, zmm11, zmm19",
    "vpbroadcastd zmm20, dword ptr [rsi + 4 * 4]",
    "vpxord zmm12, zmm12, zmm20",
    "vpbroadcastd zmm21, dword ptr [rsi + 5 * 4]",
    "vpxord zmm13, zmm13, zmm21",
    "vpbroadcastd zmm22, dword ptr [rsi + 6 * 4]",
    "vpxord zmm14, zmm14, zmm22",
    "vpbroadcastd zmm23, dword ptr [rsi + 7 * 4]",
    "vpxord zmm15, zmm15, zmm23",
    // zmm0-zmm15 now contain the final extended state vectors, transposed. We need to un-transpose
    // them before we write them out. Unlike blake3_avx512_xof_stream_16, we do a complete
    // un-transpose here, to make the xor step easier.
    //
    // First interleave 32-bit words. This takes vectors like:
    //
    // a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
    //
    // And produces vectors like:
    //
    // a0, a1, b0, b1, e0, e1, g0, g1, i0, i1, k0, k1, m0, m1, o0, o1
    "vpunpckldq zmm16, zmm0, zmm1",
    "vpunpckhdq zmm17, zmm0, zmm1",
    "vpunpckldq zmm18, zmm2, zmm3",
    "vpunpckhdq zmm19, zmm2, zmm3",
    "vpunpckldq zmm20, zmm4, zmm5",
    "vpunpckhdq zmm21, zmm4, zmm5",
    "vpunpckldq zmm22, zmm6, zmm7",
    "vpunpckhdq zmm23, zmm6, zmm7",
    "vpunpckldq zmm24, zmm8, zmm9",
    "vpunpckhdq zmm25, zmm8, zmm9",
    "vpunpckldq zmm26, zmm10, zmm11",
    "vpunpckhdq zmm27, zmm10, zmm11",
    "vpunpckldq zmm28, zmm12, zmm13",
    "vpunpckhdq zmm29, zmm12, zmm13",
    "vpunpckldq zmm30, zmm14, zmm15",
    "vpunpckhdq zmm31, zmm14, zmm15",
    // Then interleave 64-bit words, producing vectors like:
    //
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    "vpunpcklqdq zmm0, zmm16, zmm18",
    "vpunpckhqdq zmm1, zmm16, zmm18",
    "vpunpcklqdq zmm2, zmm17, zmm19",
    "vpunpckhqdq zmm3, zmm17, zmm19",
    "vpunpcklqdq zmm4, zmm20, zmm22",
    "vpunpckhqdq zmm5, zmm20, zmm22",
    "vpunpcklqdq zmm6, zmm21, zmm23",
    "vpunpckhqdq zmm7, zmm21, zmm23",
    "vpunpcklqdq zmm8, zmm24, zmm26",
    "vpunpckhqdq zmm9, zmm24, zmm26",
    "vpunpcklqdq zmm10, zmm25, zmm27",
    "vpunpckhqdq zmm11, zmm25, zmm27",
    "vpunpcklqdq zmm12, zmm28, zmm30",
    "vpunpckhqdq zmm13, zmm28, zmm30",
    "vpunpcklqdq zmm14, zmm29, zmm31",
    "vpunpckhqdq zmm15, zmm29, zmm31",
    // Then interleave 128-bit lanes, producing vectors like:
    //
    // a0, a1, a2, a3, i0, i1, i2, i3, a4, a5, a6, a7, i4, i5, i6, i7
    "vshufi32x4 zmm16, zmm0, zmm4, 0x88", // lo lanes: 0x88 = 0b10001000 = (0, 2, 0, 2)
    "vshufi32x4 zmm17, zmm1, zmm5, 0x88",
    "vshufi32x4 zmm18, zmm2, zmm6, 0x88",
    "vshufi32x4 zmm19, zmm3, zmm7, 0x88",
    "vshufi32x4 zmm20, zmm0, zmm4, 0xdd", // hi lanes: 0xdd = 0b11011101 = (1, 3, 1, 3)
    "vshufi32x4 zmm21, zmm1, zmm5, 0xdd",
    "vshufi32x4 zmm22, zmm2, zmm6, 0xdd",
    "vshufi32x4 zmm23, zmm3, zmm7, 0xdd",
    "vshufi32x4 zmm24, zmm8, zmm12, 0x88", // lo lanes
    "vshufi32x4 zmm25, zmm9, zmm13, 0x88",
    "vshufi32x4 zmm26, zmm10, zmm14, 0x88",
    "vshufi32x4 zmm27, zmm11, zmm15, 0x88",
    "vshufi32x4 zmm28, zmm8, zmm12, 0xdd", // hi lanes
    "vshufi32x4 zmm29, zmm9, zmm13, 0xdd",
    "vshufi32x4 zmm30, zmm10, zmm14, 0xdd",
    "vshufi32x4 zmm31, zmm11, zmm15, 0xdd",
    // Finally interleave 128-bit lanes again (the same permutation as the previous pass, but
    // different inputs), producing vectors like:
    //
    // a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15
    "vshufi32x4 zmm0, zmm16, zmm24, 0x88", // lo lanes
    "vshufi32x4 zmm1, zmm17, zmm25, 0x88",
    "vshufi32x4 zmm2, zmm18, zmm26, 0x88",
    "vshufi32x4 zmm3, zmm19, zmm27, 0x88",
    "vshufi32x4 zmm4, zmm20, zmm28, 0x88",
    "vshufi32x4 zmm5, zmm21, zmm29, 0x88",
    "vshufi32x4 zmm6, zmm22, zmm30, 0x88",
    "vshufi32x4 zmm7, zmm23, zmm31, 0x88",
    "vshufi32x4 zmm8, zmm16, zmm24, 0xdd", // hi lanes
    "vshufi32x4 zmm9, zmm17, zmm25, 0xdd",
    "vshufi32x4 zmm10, zmm18, zmm26, 0xdd",
    "vshufi32x4 zmm11, zmm19, zmm27, 0xdd",
    "vshufi32x4 zmm12, zmm20, zmm28, 0xdd",
    "vshufi32x4 zmm13, zmm21, zmm29, 0xdd",
    "vshufi32x4 zmm14, zmm22, zmm30, 0xdd",
    "vshufi32x4 zmm15, zmm23, zmm31, 0xdd",
    // zmm0-zmm15 now contain the fully un-transposed state words. Load each 64 block on input
    // (unaligned), perform the xor, and write out the result (again unaligned).
    "vmovdqu32 zmm16, zmmword ptr [r9 + 0 * 64]",
    "vpxord zmm0, zmm0, zmm16",
    "vmovdqu32 zmmword ptr [r9 + 0 * 64], zmm0",
    "vmovdqu32 zmm17, zmmword ptr [r9 + 1 * 64]",
    "vpxord zmm1, zmm1, zmm17",
    "vmovdqu32 zmmword ptr [r9 + 1 * 64], zmm1",
    "vmovdqu32 zmm18, zmmword ptr [r9 + 2 * 64]",
    "vpxord zmm2, zmm2, zmm18",
    "vmovdqu32 zmmword ptr [r9 + 2 * 64], zmm2",
    "vmovdqu32 zmm19, zmmword ptr [r9 + 3 * 64]",
    "vpxord zmm3, zmm3, zmm19",
    "vmovdqu32 zmmword ptr [r9 + 3 * 64], zmm3",
    "vmovdqu32 zmm20, zmmword ptr [r9 + 4 * 64]",
    "vpxord zmm4, zmm4, zmm20",
    "vmovdqu32 zmmword ptr [r9 + 4 * 64], zmm4",
    "vmovdqu32 zmm21, zmmword ptr [r9 + 5 * 64]",
    "vpxord zmm5, zmm5, zmm21",
    "vmovdqu32 zmmword ptr [r9 + 5 * 64], zmm5",
    "vmovdqu32 zmm22, zmmword ptr [r9 + 6 * 64]",
    "vpxord zmm6, zmm6, zmm22",
    "vmovdqu32 zmmword ptr [r9 + 6 * 64], zmm6",
    "vmovdqu32 zmm23, zmmword ptr [r9 + 7 * 64]",
    "vpxord zmm7, zmm7, zmm23",
    "vmovdqu32 zmmword ptr [r9 + 7 * 64], zmm7",
    "vmovdqu32 zmm24, zmmword ptr [r9 + 8 * 64]",
    "vpxord zmm8, zmm8, zmm24",
    "vmovdqu32 zmmword ptr [r9 + 8 * 64], zmm8",
    "vmovdqu32 zmm25, zmmword ptr [r9 + 9 * 64]",
    "vpxord zmm9, zmm9, zmm25",
    "vmovdqu32 zmmword ptr [r9 + 9 * 64], zmm9",
    "vmovdqu32 zmm26, zmmword ptr [r9 + 10 * 64]",
    "vpxord zmm10, zmm10, zmm26",
    "vmovdqu32 zmmword ptr [r9 + 10 * 64], zmm10",
    "vmovdqu32 zmm27, zmmword ptr [r9 + 11 * 64]",
    "vpxord zmm11, zmm11, zmm27",
    "vmovdqu32 zmmword ptr [r9 + 11 * 64], zmm11",
    "vmovdqu32 zmm28, zmmword ptr [r9 + 12 * 64]",
    "vpxord zmm12, zmm12, zmm28",
    "vmovdqu32 zmmword ptr [r9 + 12 * 64], zmm12",
    "vmovdqu32 zmm29, zmmword ptr [r9 + 13 * 64]",
    "vpxord zmm13, zmm13, zmm29",
    "vmovdqu32 zmmword ptr [r9 + 13 * 64], zmm13",
    "vmovdqu32 zmm30, zmmword ptr [r9 + 14 * 64]",
    "vpxord zmm14, zmm14, zmm30",
    "vmovdqu32 zmmword ptr [r9 + 14 * 64], zmm14",
    "vmovdqu32 zmm31, zmmword ptr [r9 + 15 * 64]",
    "vpxord zmm15, zmm15, zmm31",
    "vmovdqu32 zmmword ptr [r9 + 15 * 64], zmm15",
    "vzeroupper",
    "ret",
);

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug)]
pub struct Words16(pub [u32; 16]);

#[repr(C, align(32))]
#[derive(Copy, Clone, Debug)]
pub struct Words8(pub [u32; 8]);

pub unsafe fn chunks16(
    message: &[u8; 16 * CHUNK_LEN],
    key: &[u32; 8],
    counter: u64,
    flags: u32,
    out_ptr: *mut [Words16; 8],
) {
    // Prepare the counter vectors, the low words and high words.
    let mut counter_vectors = [Words16([0; 16]); 2];
    for i in 0..16 {
        counter_vectors[0].0[i] = (counter + i as u64) as u32;
        counter_vectors[1].0[i] = ((counter + i as u64) >> 32) as u32;
    }
    asm!(
        "call blake3_avx512_chunks_16",
        inout("rdi") message => _,
        inout("rsi") key => _,
        inout("rdx") &counter_vectors => _,
        out("ecx") _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("zmm0") _, out("zmm1") _, out("zmm2") _, out("zmm3") _,
        out("zmm4") _, out("zmm5") _, out("zmm6") _, out("zmm7") _,
        out("zmm8") _, out("zmm9") _, out("zmm10") _, out("zmm11") _,
        out("zmm12") _, out("zmm13") _, out("zmm14") _, out("zmm15") _,
        out("zmm16") _, out("zmm17") _, out("zmm18") _, out("zmm19") _,
        out("zmm20") _, out("zmm21") _, out("zmm22") _, out("zmm23") _,
        out("zmm24") _, out("zmm25") _, out("zmm26") _, out("zmm27") _,
        out("zmm28") _, out("zmm29") _, out("zmm30") _, out("zmm31") _,
    );
}

pub unsafe fn chunks8(
    message: &[u8; 8 * CHUNK_LEN],
    key: &[u32; 8],
    counter: u64,
    flags: u32,
    out_ptr: *mut [Words8; 8],
) {
    // Prepare the counter vectors, the low words and high words.
    let mut counter_vectors = [Words8([0; 8]); 2];
    for i in 0..8 {
        counter_vectors[0].0[i] = (counter + i as u64) as u32;
        counter_vectors[1].0[i] = ((counter + i as u64) >> 32) as u32;
    }
    asm!(
        "call blake3_avx512_chunks_8",
        inout("rdi") message => _,
        inout("rsi") key => _,
        inout("rdx") &counter_vectors => _,
        out("ecx") _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("ymm0") _, out("ymm1") _, out("ymm2") _, out("ymm3") _,
        out("ymm4") _, out("ymm5") _, out("ymm6") _, out("ymm7") _,
        out("ymm8") _, out("ymm9") _, out("ymm10") _, out("ymm11") _,
        out("ymm12") _, out("ymm13") _, out("ymm14") _, out("ymm15") _,
        out("ymm16") _, out("ymm17") _, out("ymm18") _, out("ymm19") _,
        out("ymm20") _, out("ymm21") _, out("ymm22") _, out("ymm23") _,
        out("ymm24") _, out("ymm25") _, out("ymm26") _, out("ymm27") _,
        out("ymm28") _, out("ymm29") _, out("ymm30") _, out("ymm31") _,
    );
}

pub unsafe fn parents16(
    left_child_cvs: &[Words16; 8],
    right_child_cvs: &[Words16; 8],
    key: &[u32; 8],
    flags: u32,
    out_ptr: *mut [Words16; 8],
) {
    asm!(
        "call blake3_avx512_parents_16",
        inout("rdi") left_child_cvs => _,
        inout("rsi") right_child_cvs => _,
        inout("rdx") key => _,
        out("ecx") _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("zmm0") _, out("zmm1") _, out("zmm2") _, out("zmm3") _,
        out("zmm4") _, out("zmm5") _, out("zmm6") _, out("zmm7") _,
        out("zmm8") _, out("zmm9") _, out("zmm10") _, out("zmm11") _,
        out("zmm12") _, out("zmm13") _, out("zmm14") _, out("zmm15") _,
        out("zmm16") _, out("zmm17") _, out("zmm18") _, out("zmm19") _,
        out("zmm20") _, out("zmm21") _, out("zmm22") _, out("zmm23") _,
        out("zmm24") _, out("zmm25") _, out("zmm26") _, out("zmm27") _,
        out("zmm28") _, out("zmm29") _, out("zmm30") _, out("zmm31") _,
    );
}

pub unsafe fn parents8(
    left_child_cvs: &[Words8; 8],
    right_child_cvs: &[Words8; 8],
    key: &[u32; 8],
    flags: u32,
    out_ptr: *mut [Words8; 8],
) {
    asm!(
        "call blake3_avx512_parents_8",
        inout("rdi") left_child_cvs => _,
        inout("rsi") right_child_cvs => _,
        inout("rdx") key => _,
        out("ecx") _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("ymm0") _, out("ymm1") _, out("ymm2") _, out("ymm3") _,
        out("ymm4") _, out("ymm5") _, out("ymm6") _, out("ymm7") _,
        out("ymm8") _, out("ymm9") _, out("ymm10") _, out("ymm11") _,
        out("ymm12") _, out("ymm13") _, out("ymm14") _, out("ymm15") _,
        out("ymm16") _, out("ymm17") _, out("ymm18") _, out("ymm19") _,
        out("ymm20") _, out("ymm21") _, out("ymm22") _, out("ymm23") _,
        out("ymm24") _, out("ymm25") _, out("ymm26") _, out("ymm27") _,
        out("ymm28") _, out("ymm29") _, out("ymm30") _, out("ymm31") _,
    );
}

pub unsafe fn xof_stream16(
    message_words: &[u32; 16],
    cv_words: &[u32; 8],
    counter: u64,
    block_len: u32,
    flags: u32,
    out_ptr: *mut [u8; 16 * 64],
) {
    // Prepare the counter vectors, the low words and high words.
    let mut counter_vectors = [Words16([0; 16]); 2];
    for i in 0..16 {
        counter_vectors[0].0[i] = (counter + i as u64) as u32;
        counter_vectors[1].0[i] = ((counter + i as u64) >> 32) as u32;
    }
    asm!(
        "call blake3_avx512_xof_stream_16",
        inout("rdi") message_words => _,
        inout("rsi") cv_words => _,
        inout("rdx") &counter_vectors => _,
        inout("ecx") block_len => _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("zmm0") _, out("zmm1") _, out("zmm2") _, out("zmm3") _,
        out("zmm4") _, out("zmm5") _, out("zmm6") _, out("zmm7") _,
        out("zmm8") _, out("zmm9") _, out("zmm10") _, out("zmm11") _,
        out("zmm12") _, out("zmm13") _, out("zmm14") _, out("zmm15") _,
        out("zmm16") _, out("zmm17") _, out("zmm18") _, out("zmm19") _,
        out("zmm20") _, out("zmm21") _, out("zmm22") _, out("zmm23") _,
        out("zmm24") _, out("zmm25") _, out("zmm26") _, out("zmm27") _,
        out("zmm28") _, out("zmm29") _, out("zmm30") _, out("zmm31") _,
    );
}

pub unsafe fn xof_xor16(
    message_words: &[u32; 16],
    cv_words: &[u32; 8],
    counter: u64,
    block_len: u32,
    flags: u32,
    out_ptr: &mut [u8; 16 * 64],
) {
    // Prepare the counter vectors, the low words and high words.
    let mut counter_vectors = [Words16([0; 16]); 2];
    for i in 0..16 {
        counter_vectors[0].0[i] = (counter + i as u64) as u32;
        counter_vectors[1].0[i] = ((counter + i as u64) >> 32) as u32;
    }
    asm!(
        "call blake3_avx512_xof_xor_16",
        inout("rdi") message_words => _,
        inout("rsi") cv_words => _,
        inout("rdx") &counter_vectors => _,
        inout("ecx") block_len => _,
        inout("r8d") flags => _,
        inout("r9") out_ptr => _,
        out("zmm0") _, out("zmm1") _, out("zmm2") _, out("zmm3") _,
        out("zmm4") _, out("zmm5") _, out("zmm6") _, out("zmm7") _,
        out("zmm8") _, out("zmm9") _, out("zmm10") _, out("zmm11") _,
        out("zmm12") _, out("zmm13") _, out("zmm14") _, out("zmm15") _,
        out("zmm16") _, out("zmm17") _, out("zmm18") _, out("zmm19") _,
        out("zmm20") _, out("zmm21") _, out("zmm22") _, out("zmm23") _,
        out("zmm24") _, out("zmm25") _, out("zmm26") _, out("zmm27") _,
        out("zmm28") _, out("zmm29") _, out("zmm30") _, out("zmm31") _,
    );
}

#[test]
fn test_chunks16() {
    let mut message = [0u8; 16 * CHUNK_LEN];
    crate::test::paint_test_input(&mut message);

    let mut chunk_refs: Vec<&[u8; CHUNK_LEN]> = Vec::new();
    for i in 0..16 {
        chunk_refs.push(message[i * CHUNK_LEN..][..CHUNK_LEN].try_into().unwrap());
    }
    let counter = u32::MAX as u64; // a counter value that will overflow 32 bits
    let flags = crate::KEYED_HASH;
    let mut expected_out = [0u8; 32 * 16];
    unsafe {
        crate::avx512::hash_many(
            chunk_refs[..].try_into().unwrap(),
            crate::IV,
            counter,
            crate::IncrementCounter::Yes,
            flags,
            crate::CHUNK_START,
            crate::CHUNK_END,
            &mut expected_out,
        );
    }

    let mut found_out = [Words16([0; 16]); 8];
    unsafe {
        chunks16(&message, crate::IV, counter, flags as u32, &mut found_out);
    }
    let mut found_out_transposed = [0; 8 * 16 * 4];
    for vector_i in 0..8 {
        for element_i in 0..16 {
            let word = found_out[vector_i].0[element_i];
            let word_start = 32 * element_i + 4 * vector_i;
            found_out_transposed[word_start..][..4].copy_from_slice(&word.to_le_bytes());
        }
    }
    assert_eq!(expected_out, found_out_transposed);
}

#[test]
fn test_chunks8() {
    let mut message = [0u8; 8 * CHUNK_LEN];
    crate::test::paint_test_input(&mut message);

    let mut chunk_refs: Vec<&[u8; CHUNK_LEN]> = Vec::new();
    for i in 0..8 {
        chunk_refs.push(message[i * CHUNK_LEN..][..CHUNK_LEN].try_into().unwrap());
    }
    let counter = u32::MAX as u64; // a counter value that will overflow 32 bits
    let flags = crate::KEYED_HASH;
    let mut expected_out = [0u8; 32 * 8];
    unsafe {
        crate::avx512::hash_many(
            chunk_refs[..].try_into().unwrap(),
            crate::IV,
            counter,
            crate::IncrementCounter::Yes,
            flags,
            crate::CHUNK_START,
            crate::CHUNK_END,
            &mut expected_out,
        );
    }

    let mut found_out = [Words8([0; 8]); 8];
    unsafe {
        chunks8(&message, crate::IV, counter, flags as u32, &mut found_out);
    }
    let mut found_out_transposed = [0; 8 * 8 * 4];
    for vector_i in 0..8 {
        for element_i in 0..8 {
            let word = found_out[vector_i].0[element_i];
            let word_start = 32 * element_i + 4 * vector_i;
            found_out_transposed[word_start..][..4].copy_from_slice(&word.to_le_bytes());
        }
    }
    assert_eq!(expected_out, found_out_transposed);
}

#[test]
fn test_parents16() {
    // 16 left child CVs and 16 right child CVs, each 32 bytes long
    let mut child_cvs = [0u8; 2 * 16 * 32];
    crate::test::paint_test_input(&mut child_cvs);
    let mut child_cv_refs = [&[0; 64]; 16]; // references to parent nodes
    for i in 0..16 {
        child_cv_refs[i] = (&child_cvs[i * 64..][..64]).try_into().unwrap();
    }
    // 16 output CVs of 32 bytes each.
    let flags = crate::KEYED_HASH;
    let mut expected_out = [0; 32 * 16];
    unsafe {
        crate::avx512::hash_many(
            &child_cv_refs,
            crate::IV,
            0,
            crate::IncrementCounter::No,
            flags | crate::PARENT,
            0,
            0,
            &mut expected_out,
        );
    }

    // 8 transposed left child CVs and 8 transposed right child CVs
    let mut transposed_left_child_cvs = [Words16([0; 16]); 8];
    let mut transposed_right_child_cvs = [Words16([0; 16]); 8];
    for child_i in 0..16 {
        for word_i in 0..8 {
            let word = u32::from_le_bytes(
                child_cvs[child_i * 32 + word_i * 4..][..4]
                    .try_into()
                    .unwrap(),
            );
            transposed_left_child_cvs[word_i].0[child_i] = word;
        }
    }
    for child_i in 16..32 {
        for word_i in 0..8 {
            let word = u32::from_le_bytes(
                child_cvs[child_i * 32 + word_i * 4..][..4]
                    .try_into()
                    .unwrap(),
            );
            transposed_right_child_cvs[word_i].0[child_i - 16] = word;
        }
    }
    let mut found_out = [Words16([0; 16]); 8];
    unsafe {
        parents16(
            &transposed_left_child_cvs,
            &transposed_right_child_cvs,
            crate::IV,
            flags as u32,
            &mut found_out,
        );
    }
    let mut found_out_transposed = [0; 8 * 16 * 4];
    for vector_i in 0..8 {
        for element_i in 0..16 {
            let word = found_out[vector_i].0[element_i];
            let word_start = 32 * element_i + 4 * vector_i;
            found_out_transposed[word_start..][..4].copy_from_slice(&word.to_le_bytes());
        }
    }
    assert_eq!(expected_out, found_out_transposed);
}

#[test]
fn test_parents8() {
    // 8 left child CVs and 8 right child CVs, each 32 bytes long
    let mut child_cvs = [0u8; 2 * 8 * 32];
    crate::test::paint_test_input(&mut child_cvs);
    let mut child_cv_refs = [&[0; 64]; 8]; // references to parent nodes
    for i in 0..8 {
        child_cv_refs[i] = (&child_cvs[i * 64..][..64]).try_into().unwrap();
    }
    // 8 output CVs of 32 bytes each.
    let flags = crate::KEYED_HASH;
    let mut expected_out = [0; 32 * 8];
    unsafe {
        crate::avx512::hash_many(
            &child_cv_refs,
            crate::IV,
            0,
            crate::IncrementCounter::No,
            flags | crate::PARENT,
            0,
            0,
            &mut expected_out,
        );
    }

    // 8 transposed left child CVs and 8 transposed right child CVs
    let mut transposed_left_child_cvs = [Words8([0; 8]); 8];
    let mut transposed_right_child_cvs = [Words8([0; 8]); 8];
    for child_i in 0..8 {
        for word_i in 0..8 {
            let word = u32::from_le_bytes(
                child_cvs[child_i * 32 + word_i * 4..][..4]
                    .try_into()
                    .unwrap(),
            );
            transposed_left_child_cvs[word_i].0[child_i] = word;
        }
    }
    for child_i in 8..16 {
        for word_i in 0..8 {
            let word = u32::from_le_bytes(
                child_cvs[child_i * 32 + word_i * 4..][..4]
                    .try_into()
                    .unwrap(),
            );
            transposed_right_child_cvs[word_i].0[child_i - 8] = word;
        }
    }
    let mut found_out = [Words8([0; 8]); 8];
    unsafe {
        parents8(
            &transposed_left_child_cvs,
            &transposed_right_child_cvs,
            crate::IV,
            flags as u32,
            &mut found_out,
        );
    }
    let mut found_out_transposed = [0; 8 * 8 * 4];
    for vector_i in 0..8 {
        for element_i in 0..8 {
            let word = found_out[vector_i].0[element_i];
            let word_start = 32 * element_i + 4 * vector_i;
            found_out_transposed[word_start..][..4].copy_from_slice(&word.to_le_bytes());
        }
    }
    assert_eq!(expected_out, found_out_transposed);
}

#[test]
fn test_xof_stream16() {
    let mut padded_block = [0; 64];
    let block_len = 53;
    let block = &mut padded_block[..block_len];
    let mut key = [0; 32];
    crate::test::paint_test_input(block);
    crate::test::paint_test_input(&mut key);
    let mut expected = [0; 1024];
    crate::Hasher::new_keyed(&key)
        .update(block)
        .finalize_xof()
        .fill(&mut expected);

    let block_words = crate::platform::words_from_le_bytes_64(&padded_block);
    let key_words = crate::platform::words_from_le_bytes_32(&key);
    let flags = crate::KEYED_HASH | crate::CHUNK_START | crate::CHUNK_END;
    let mut found_aligned = [0u32; 16 * 16 + 1];
    assert!(std::mem::size_of_val(&found_aligned) > 1024);
    let found: &mut [u8; 1024] =
        unsafe { &mut *((found_aligned.as_mut_ptr() as *mut u8).add(1) as *mut _) };
    assert_eq!(1, found.as_ptr() as usize % 4);
    unsafe {
        xof_stream16(
            &block_words,
            &key_words,
            0,
            block_len as u32,
            flags as u32,
            found,
        );
    }
    assert_eq!(expected, *found);
}

#[test]
fn test_xof_xor16() {
    let mut padded_block = [0; 64];
    let block_len = 53;
    let block = &mut padded_block[..block_len];
    let mut key = [0; 32];
    crate::test::paint_test_input(block);
    crate::test::paint_test_input(&mut key);
    let mut expected = [0; 1024];
    crate::Hasher::new_keyed(&key)
        .update(block)
        .finalize_xof()
        .fill(&mut expected);
    // xor in our regular input pattern manually.
    for i in 0..expected.len() {
        expected[i] ^= (i % 251) as u8;
    }

    let block_words = crate::platform::words_from_le_bytes_64(&padded_block);
    let key_words = crate::platform::words_from_le_bytes_32(&key);
    let flags = crate::KEYED_HASH | crate::CHUNK_START | crate::CHUNK_END;
    let mut found_aligned = [0u32; 16 * 16 + 1];
    assert!(std::mem::size_of_val(&found_aligned) > 1024);
    let found: &mut [u8; 1024] =
        unsafe { &mut *((found_aligned.as_mut_ptr() as *mut u8).add(1) as *mut _) };
    assert_eq!(1, found.as_ptr() as usize % 4);
    // Apply the regular input pattern to the output, so we'll xor on top of that.
    crate::test::paint_test_input(found);
    unsafe {
        xof_xor16(
            &block_words,
            &key_words,
            0,
            block_len as u32,
            flags as u32,
            found,
        );
    }
    assert_eq!(expected, *found);
}
