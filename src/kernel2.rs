use crate::{BLOCK_LEN, CHUNK_LEN, IV};
use core::arch::x86_64::*;
use core::arch::{asm, global_asm};
use core::mem;

global_asm!(
    // --------------------------------------------------------------------------------------------
    // blake3_avx512_kernel2_16
    //
    // Inputs:
    //     zmm0-zmm7: transposed input CV
    //    zmm8-zmm11: broadcasted IV0-IV3 words
    //         zmm12: transposed lower order counter words
    //         zmm13: transposed higher order counter words
    //         zmm14: broadcasted block sizes (always 64)
    //         zmm15: broadcasted flag words
    //   zmm16-zmm31: transposed message vectors
    //
    // Outputs:
    //   zmm0-zmm16: transposed output state minus the feed-forward
    //
    // "Minus the feed-forward" means that the final xors have been done in the lower half of the
    // state but not the upper half. XOF callers need to do the upper half xors / CV-feed-forward
    // themselves.
    // --------------------------------------------------------------------------------------------
    "blake3_avx512_kernel2_16:",
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
);

#[inline]
#[target_feature(enable = "avx512f,avx512vl")]
unsafe fn load_transposed_16(input: *const u8) -> [__m512i; 16] {
    // We're going to load 16 vectors, each containing 16 words (64 bytes). We assume that these
    // vectors are coming from contiguous chunks, so each is offset by CHUNK_LEN (1024 bytes) from
    // the last. We'll name the input vectors a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, and p.
    // Well denote the words of the input vectors:
    //
    //     a_0, a_1, a_2, a_3, a_4, a_5, a_6, a_7, a_8, a_9, a_a, a_b, a_c, a_d, a_e, a_f
    //     b_0, b_1, b_2, b_3, b_4, b_5, b_6, b_7, b_8, b_9, b_a, b_b, b_c, b_d, b_e, b_f
    //     etc.
    //
    // Our goal is to load and transpose these into output vectors that look like:
    //
    //     a_0, b_0, c_0, d_0, e_0, f_0, g_0, h_0, i_0, j_0, k_0, l_0, m_0, n_0, o_0, p_0
    //     a_1, b_1, c_1, d_1, e_1, f_1, g_1, h_1, i_1, j_1, k_1, l_1, m_1, n_1, o_1, p_1
    //     etc.

    // Because operations that cross 128-bit lanes are relatively expensive, we split each 512-bit
    // load into four 128-bit loads. This results in vectors like:
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    #[inline(always)]
    unsafe fn load_4_lanes(input: *const u8) -> __m512i {
        let lane0 = _mm_loadu_epi32(input.add(0 * CHUNK_LEN) as *const i32);
        let lane1 = _mm_loadu_epi32(input.add(4 * CHUNK_LEN) as *const i32);
        let lane2 = _mm_loadu_epi32(input.add(8 * CHUNK_LEN) as *const i32);
        let lane3 = _mm_loadu_epi32(input.add(12 * CHUNK_LEN) as *const i32);
        let ret = _mm512_castsi128_si512(lane0);
        let ret = _mm512_inserti32x4::<1>(ret, lane1);
        let ret = _mm512_inserti32x4::<2>(ret, lane2);
        let ret = _mm512_inserti32x4::<3>(ret, lane3);
        ret
    }
    let aeim_0123 = load_4_lanes(input.add(0 * CHUNK_LEN + 0 * 16));
    let aeim_4567 = load_4_lanes(input.add(0 * CHUNK_LEN + 1 * 16));
    let aeim_89ab = load_4_lanes(input.add(0 * CHUNK_LEN + 2 * 16));
    let aeim_cdef = load_4_lanes(input.add(0 * CHUNK_LEN + 3 * 16));
    let bfjn_0123 = load_4_lanes(input.add(1 * CHUNK_LEN + 0 * 16));
    let bfjn_4567 = load_4_lanes(input.add(1 * CHUNK_LEN + 1 * 16));
    let bfjn_89ab = load_4_lanes(input.add(1 * CHUNK_LEN + 2 * 16));
    let bfjn_cdef = load_4_lanes(input.add(1 * CHUNK_LEN + 3 * 16));
    let cgko_0123 = load_4_lanes(input.add(2 * CHUNK_LEN + 0 * 16));
    let cgko_4567 = load_4_lanes(input.add(2 * CHUNK_LEN + 1 * 16));
    let cgko_89ab = load_4_lanes(input.add(2 * CHUNK_LEN + 2 * 16));
    let cgko_cdef = load_4_lanes(input.add(2 * CHUNK_LEN + 3 * 16));
    let dhlp_0123 = load_4_lanes(input.add(3 * CHUNK_LEN + 0 * 16));
    let dhlp_4567 = load_4_lanes(input.add(3 * CHUNK_LEN + 1 * 16));
    let dhlp_89ab = load_4_lanes(input.add(3 * CHUNK_LEN + 2 * 16));
    let dhlp_cdef = load_4_lanes(input.add(3 * CHUNK_LEN + 3 * 16));

    // Interleave 32-bit words. This results in vectors like:
    // a0, b0, a1, b1, e0, f0, e1, f1, i0, j0, i1, j1, m0, n0, m1, n1
    let abefijmn_01 = _mm512_unpacklo_epi32(aeim_0123, bfjn_0123);
    let abefijmn_23 = _mm512_unpackhi_epi32(aeim_0123, bfjn_0123);
    let abefijmn_45 = _mm512_unpacklo_epi32(aeim_4567, bfjn_4567);
    let abefijmn_67 = _mm512_unpackhi_epi32(aeim_4567, bfjn_4567);
    let abefijmn_89 = _mm512_unpacklo_epi32(aeim_89ab, bfjn_89ab);
    let abefijmn_ab = _mm512_unpackhi_epi32(aeim_89ab, bfjn_89ab);
    let abefijmn_cd = _mm512_unpacklo_epi32(aeim_cdef, bfjn_cdef);
    let abefijmn_ef = _mm512_unpackhi_epi32(aeim_cdef, bfjn_cdef);
    let cdghklop_01 = _mm512_unpacklo_epi32(cgko_0123, dhlp_0123);
    let cdghklop_23 = _mm512_unpackhi_epi32(cgko_0123, dhlp_0123);
    let cdghklop_45 = _mm512_unpacklo_epi32(cgko_4567, dhlp_4567);
    let cdghklop_67 = _mm512_unpackhi_epi32(cgko_4567, dhlp_4567);
    let cdghklop_89 = _mm512_unpacklo_epi32(cgko_89ab, dhlp_89ab);
    let cdghklop_ab = _mm512_unpackhi_epi32(cgko_89ab, dhlp_89ab);
    let cdghklop_cd = _mm512_unpacklo_epi32(cgko_cdef, dhlp_cdef);
    let cdghklop_ef = _mm512_unpackhi_epi32(cgko_cdef, dhlp_cdef);

    // Finally, interleave 64-bit words. This gives us our goal, which is vectors like:
    // a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
    [
        _mm512_unpacklo_epi64(abefijmn_01, cdghklop_01),
        _mm512_unpackhi_epi64(abefijmn_01, cdghklop_01),
        _mm512_unpacklo_epi64(abefijmn_23, cdghklop_23),
        _mm512_unpackhi_epi64(abefijmn_23, cdghklop_23),
        _mm512_unpacklo_epi64(abefijmn_45, cdghklop_45),
        _mm512_unpackhi_epi64(abefijmn_45, cdghklop_45),
        _mm512_unpacklo_epi64(abefijmn_67, cdghklop_67),
        _mm512_unpackhi_epi64(abefijmn_67, cdghklop_67),
        _mm512_unpacklo_epi64(abefijmn_89, cdghklop_89),
        _mm512_unpackhi_epi64(abefijmn_89, cdghklop_89),
        _mm512_unpacklo_epi64(abefijmn_ab, cdghklop_ab),
        _mm512_unpackhi_epi64(abefijmn_ab, cdghklop_ab),
        _mm512_unpacklo_epi64(abefijmn_cd, cdghklop_cd),
        _mm512_unpackhi_epi64(abefijmn_cd, cdghklop_cd),
        _mm512_unpacklo_epi64(abefijmn_ef, cdghklop_ef),
        _mm512_unpackhi_epi64(abefijmn_ef, cdghklop_ef),
    ]
}

#[test]
fn test_load_transpose_16() {
    if !crate::platform::avx512_detected() {
        return;
    }
    // 16 chunks, as 32-bit words rather than bytes
    let mut input = [0u32; (CHUNK_LEN / 4) * 16];
    // Populate the first 16 words of each chunk with an incrementing counter.
    for chunk in 0..16 {
        for word in 0..16 {
            let chunk_start = (CHUNK_LEN / 4) * chunk;
            let val = 16 * chunk as u32 + word as u32;
            input[chunk_start + word] = val;
        }
    }
    // Run the load-transpose and cast the vectors back to words.
    let transposed: [u32; 16 * 16] = unsafe {
        let vecs = load_transposed_16(input.as_ptr() as *const u8);
        mem::transmute(vecs)
    };
    // Now check for the same incrementing counter as above, except we've swapped the inner and
    // outer loops to account for the transposition.
    for word in 0..16 {
        for vec in 0..16 {
            let vec_start = 16 * vec;
            let val = 16 * word as u32 + vec as u32;
            assert_eq!(transposed[vec_start + word], val, "word {word} vec {vec}");
        }
    }
}

// returns (low_words, high_words)
#[inline(always)]
unsafe fn incrementing_counter(initial_value: u64) -> (__m512i, __m512i) {
    let increments = _mm512_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let initial_low_words = _mm512_set1_epi32(initial_value as u32 as i32);
    let low_words = _mm512_add_epi32(initial_low_words, increments);
    let less_than_mask = _mm512_cmplt_epu32_mask(low_words, initial_low_words);
    let initial_high_words = _mm512_set1_epi32((initial_value >> 32) as u32 as i32);
    let high_words = _mm512_mask_add_epi32(
        initial_high_words,
        less_than_mask,
        initial_high_words,
        _mm512_set1_epi32(1),
    );
    (low_words, high_words)
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn chunks_16(
    message: &[u8; 16 * CHUNK_LEN],
    key: &[u32; 8],
    counter: u64,
    flags: u32,
) -> [__m512i; 8] {
    let (counters_low, counters_high) = incrementing_counter(counter);
    let mut state_regs = [
        _mm512_set1_epi32(key[0] as i32),
        _mm512_set1_epi32(key[1] as i32),
        _mm512_set1_epi32(key[2] as i32),
        _mm512_set1_epi32(key[3] as i32),
        _mm512_set1_epi32(key[4] as i32),
        _mm512_set1_epi32(key[5] as i32),
        _mm512_set1_epi32(key[6] as i32),
        _mm512_set1_epi32(key[7] as i32),
    ];
    for block in 0..(CHUNK_LEN / BLOCK_LEN) {
        let message_regs = load_transposed_16(message.as_ptr().add(BLOCK_LEN * block));
        let mut block_flags = flags;
        if block == 0 {
            block_flags |= crate::CHUNK_START as u32;
        }
        if block == (CHUNK_LEN / BLOCK_LEN) - 1 {
            block_flags |= crate::CHUNK_END as u32;
        }
        asm!(
            "call blake3_avx512_kernel2_16",
            inout("zmm0") state_regs[0],
            inout("zmm1") state_regs[1],
            inout("zmm2") state_regs[2],
            inout("zmm3") state_regs[3],
            inout("zmm4") state_regs[4],
            inout("zmm5") state_regs[5],
            inout("zmm6") state_regs[6],
            inout("zmm7") state_regs[7],
            in("zmm8") _mm512_set1_epi32(IV[0] as i32),
            in("zmm9") _mm512_set1_epi32(IV[1] as i32),
            in("zmm10") _mm512_set1_epi32(IV[2] as i32),
            in("zmm11") _mm512_set1_epi32(IV[3] as i32),
            in("zmm12") counters_low,
            in("zmm13") counters_high,
            in("zmm14") _mm512_set1_epi32(BLOCK_LEN as i32),
            in("zmm15") _mm512_set1_epi32(block_flags as i32),
            in("zmm16") message_regs[0],
            in("zmm17") message_regs[1],
            in("zmm18") message_regs[2],
            in("zmm19") message_regs[3],
            in("zmm20") message_regs[4],
            in("zmm21") message_regs[5],
            in("zmm22") message_regs[6],
            in("zmm23") message_regs[7],
            in("zmm24") message_regs[8],
            in("zmm25") message_regs[9],
            in("zmm26") message_regs[10],
            in("zmm27") message_regs[11],
            in("zmm28") message_regs[12],
            in("zmm29") message_regs[13],
            in("zmm30") message_regs[14],
            in("zmm31") message_regs[15],
        );
    }
    state_regs
}

pub fn portable_transpose<T, const N: usize>(words: &mut [[T; N]; N]) {
    if N == 0 {
        return;
    }
    for row in 0..(N - 1) {
        for col in (row + 1)..N {
            let (upper_rows, lower_rows) = words.split_at_mut(row + 1);
            mem::swap(
                &mut upper_rows[row][col],
                &mut lower_rows[col - row - 1][row],
            );
        }
    }
}

#[test]
fn test_portable_transpose() {
    let mut empty: [[u32; 0]; 0] = [];
    portable_transpose(&mut empty);

    let mut one = [[42u32]];
    portable_transpose(&mut one);
    assert_eq!(one[0][0], 42);

    let mut two = [[0u32, 1], [2, 3]];
    portable_transpose(&mut two);
    assert_eq!(two, [[0, 2], [1, 3]]);

    let mut three = [[0u32, 1, 2], [3, 4, 5], [6, 7, 8]];
    portable_transpose(&mut three);
    assert_eq!(three, [[0, 3, 6], [1, 4, 7], [2, 5, 8]]);

    let mut painted_bytes = [0; 100];
    crate::test::paint_test_input(&mut painted_bytes);
    let painted_rows: [[u8; 10]; 10] = unsafe { mem::transmute(painted_bytes) };
    let mut transposed_rows = painted_rows;
    portable_transpose(&mut transposed_rows);
    for row in 0..10 {
        for col in 0..10 {
            assert_eq!(painted_rows[row][col], transposed_rows[col][row]);
        }
    }
}

#[test]
fn test_chunks_16() {
    if !crate::platform::avx512_detected() {
        return;
    }
    let mut chunks = [0; 16 * CHUNK_LEN];
    crate::test::paint_test_input(&mut chunks);
    let key = [42, 43, 44, 45, 46, 47, 48, 49];
    let counter = (1 << 32) - 1;
    let outputs: [__m512i; 8] =
        unsafe { chunks_16(&chunks, &key, counter, crate::KEYED_HASH as u32) };

    let separate_chunks: [&[u8; CHUNK_LEN]; 16] =
        core::array::from_fn(|i| chunks[i * CHUNK_LEN..][..CHUNK_LEN].try_into().unwrap());
    let mut expected = [0u8; 16 * 32];
    crate::portable::hash_many(
        &separate_chunks,
        &key,
        counter,
        crate::IncrementCounter::Yes,
        crate::KEYED_HASH,
        crate::CHUNK_START,
        crate::CHUNK_END,
        &mut expected,
    );

    let outputs_u32: [[u32; 16]; 8] = unsafe { mem::transmute(outputs) };
    let mut compare = [0u8; 16 * 32];
    for vec in 0..8 {
        for word in 0..16 {
            compare[word * 32 + vec * 4..][..4]
                .copy_from_slice(&outputs_u32[vec][word].to_le_bytes());
        }
    }
    assert_eq!(expected, compare);
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn parents_16(
    _left_children: &[__m512i; 8],
    _right_children: &[__m512i; 8],
    _key: &[u32; 8],
    _flags: u32,
) -> [__m512i; 8] {
    todo!();
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn just_kernel2() {
    asm!(
        "call blake3_avx512_kernel2_16",
        in("zmm0") _mm512_set1_epi32(0),
        in("zmm1") _mm512_set1_epi32(0),
        in("zmm2") _mm512_set1_epi32(0),
        in("zmm3") _mm512_set1_epi32(0),
        in("zmm4") _mm512_set1_epi32(0),
        in("zmm5") _mm512_set1_epi32(0),
        in("zmm6") _mm512_set1_epi32(0),
        in("zmm7") _mm512_set1_epi32(0),
        in("zmm8") _mm512_set1_epi32(0),
        in("zmm9") _mm512_set1_epi32(0),
        in("zmm10") _mm512_set1_epi32(0),
        in("zmm11") _mm512_set1_epi32(0),
        in("zmm12") _mm512_set1_epi32(0),
        in("zmm13") _mm512_set1_epi32(0),
        in("zmm14") _mm512_set1_epi32(0),
        in("zmm15") _mm512_set1_epi32(0),
        in("zmm16") _mm512_set1_epi32(0),
        in("zmm17") _mm512_set1_epi32(0),
        in("zmm18") _mm512_set1_epi32(0),
        in("zmm19") _mm512_set1_epi32(0),
        in("zmm20") _mm512_set1_epi32(0),
        in("zmm21") _mm512_set1_epi32(0),
        in("zmm22") _mm512_set1_epi32(0),
        in("zmm23") _mm512_set1_epi32(0),
        in("zmm24") _mm512_set1_epi32(0),
        in("zmm25") _mm512_set1_epi32(0),
        in("zmm26") _mm512_set1_epi32(0),
        in("zmm27") _mm512_set1_epi32(0),
        in("zmm28") _mm512_set1_epi32(0),
        in("zmm29") _mm512_set1_epi32(0),
        in("zmm30") _mm512_set1_epi32(0),
        in("zmm31") _mm512_set1_epi32(0),
    );
}

#[inline]
#[target_feature(enable = "avx512f,avx512vl")]
unsafe fn xof_inner_16(
    block: &[u8; BLOCK_LEN],
    cv: &[u32; 8],
    counter: u64,
    block_len: u32,
    flags: u32,
) -> [__m512i; 16] {
    let (counters_low, counters_high) = incrementing_counter(counter);
    let mut state = [
        _mm512_set1_epi32(cv[0] as i32),
        _mm512_set1_epi32(cv[1] as i32),
        _mm512_set1_epi32(cv[2] as i32),
        _mm512_set1_epi32(cv[3] as i32),
        _mm512_set1_epi32(cv[4] as i32),
        _mm512_set1_epi32(cv[5] as i32),
        _mm512_set1_epi32(cv[6] as i32),
        _mm512_set1_epi32(cv[7] as i32),
        _mm512_set1_epi32(IV[0] as i32),
        _mm512_set1_epi32(IV[1] as i32),
        _mm512_set1_epi32(IV[2] as i32),
        _mm512_set1_epi32(IV[3] as i32),
        counters_low,
        counters_high,
        _mm512_set1_epi32(block_len as i32),
        _mm512_set1_epi32(flags as i32),
        _mm512_set1_epi32(i32::from_le_bytes(block[0..4].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[4..8].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[8..12].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[12..16].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[16..20].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[20..24].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[24..28].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[28..32].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[32..36].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[36..40].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[40..44].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[44..48].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[48..52].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[52..56].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[56..60].try_into().unwrap())),
        _mm512_set1_epi32(i32::from_le_bytes(block[60..64].try_into().unwrap())),
    ];
    asm!(
        "call blake3_avx512_kernel2_16",
        inout("zmm0") state[0],
        inout("zmm1") state[1],
        inout("zmm2") state[2],
        inout("zmm3") state[3],
        inout("zmm4") state[4],
        inout("zmm5") state[5],
        inout("zmm6") state[6],
        inout("zmm7") state[7],
        inout("zmm8") state[8],
        inout("zmm9") state[9],
        inout("zmm10") state[10],
        inout("zmm11") state[11],
        inout("zmm12") state[12],
        inout("zmm13") state[13],
        inout("zmm14") state[14],
        inout("zmm15") state[15],
        in("zmm16") state[16],
        in("zmm17") state[17],
        in("zmm18") state[18],
        in("zmm19") state[19],
        in("zmm20") state[20],
        in("zmm21") state[21],
        in("zmm22") state[22],
        in("zmm23") state[23],
        in("zmm24") state[24],
        in("zmm25") state[25],
        in("zmm26") state[26],
        in("zmm27") state[27],
        in("zmm28") state[28],
        in("zmm29") state[29],
        in("zmm30") state[30],
        in("zmm31") state[31],
    );

    // Perform the feed-forward xors into the upper half of the state. (The xors in the lower half
    // of the state are done in the kernel function.)
    for i in 0..8 {
        state[i + 8] = _mm512_xor_si512(state[i + 8], _mm512_set1_epi32(cv[i] as i32));
    }

    // Partially untranspose the state vectors. We'll use the same trick here as with message
    // loading, where we avoid doing any relatively expensive cross-128-bit-lane operations, and
    // instead we delay reordering 128-bit lanes until the store step.

    // Interleave 32-bit words, producing vectors like:
    // a0, a1, b0, b1, e0, e1, f0, f1, i0, i1, j0, j1, m0, m1, n0, n1
    let abefijmn_01 = _mm512_unpacklo_epi32(state[0], state[1]);
    let cdghklop_01 = _mm512_unpackhi_epi32(state[0], state[1]);
    let abefijmn_23 = _mm512_unpacklo_epi32(state[2], state[3]);
    let cdghklop_23 = _mm512_unpackhi_epi32(state[2], state[3]);
    let abefijmn_45 = _mm512_unpacklo_epi32(state[4], state[5]);
    let cdghklop_45 = _mm512_unpackhi_epi32(state[4], state[5]);
    let abefijmn_67 = _mm512_unpacklo_epi32(state[6], state[7]);
    let cdghklop_67 = _mm512_unpackhi_epi32(state[6], state[7]);
    let abefijmn_89 = _mm512_unpacklo_epi32(state[8], state[9]);
    let cdghklop_89 = _mm512_unpackhi_epi32(state[8], state[9]);
    let abefijmn_ab = _mm512_unpacklo_epi32(state[10], state[11]);
    let cdghklop_ab = _mm512_unpackhi_epi32(state[10], state[11]);
    let abefijmn_cd = _mm512_unpacklo_epi32(state[12], state[13]);
    let cdghklop_cd = _mm512_unpackhi_epi32(state[12], state[13]);
    let abefijmn_ef = _mm512_unpacklo_epi32(state[14], state[15]);
    let cdghklop_ef = _mm512_unpackhi_epi32(state[14], state[15]);

    // Interleave 64-bit words, producing vectors like:
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    let aeim_0123 = _mm512_unpacklo_epi64(abefijmn_01, abefijmn_23);
    let bfjn_0123 = _mm512_unpackhi_epi64(abefijmn_01, abefijmn_23);
    let cgko_0123 = _mm512_unpacklo_epi64(cdghklop_01, cdghklop_23);
    let dhlp_0123 = _mm512_unpackhi_epi64(cdghklop_01, cdghklop_23);
    let aeim_4567 = _mm512_unpacklo_epi64(abefijmn_45, abefijmn_67);
    let bfjn_4567 = _mm512_unpackhi_epi64(abefijmn_45, abefijmn_67);
    let cgko_4567 = _mm512_unpacklo_epi64(cdghklop_45, cdghklop_67);
    let dhlp_4567 = _mm512_unpackhi_epi64(cdghklop_45, cdghklop_67);
    let aeim_89ab = _mm512_unpacklo_epi64(abefijmn_89, abefijmn_ab);
    let bfjn_89ab = _mm512_unpackhi_epi64(abefijmn_89, abefijmn_ab);
    let cgko_89ab = _mm512_unpacklo_epi64(cdghklop_89, cdghklop_ab);
    let dhlp_89ab = _mm512_unpackhi_epi64(cdghklop_89, cdghklop_ab);
    let aeim_cdef = _mm512_unpacklo_epi64(abefijmn_cd, abefijmn_ef);
    let bfjn_cdef = _mm512_unpackhi_epi64(abefijmn_cd, abefijmn_ef);
    let cgko_cdef = _mm512_unpacklo_epi64(cdghklop_cd, cdghklop_ef);
    let dhlp_cdef = _mm512_unpackhi_epi64(cdghklop_cd, cdghklop_ef);

    // Then interleave 128-bit lanes, producing vectors like:
    // a0, a1, a2, a3, i0, i1, i2, i3, a4, a5, a6, a7, i4, i5, i6, i7
    const LO_LANES: i32 = 0x88; // 0b10001000 = (0, 2, 0, 2)
    const HI_LANES: i32 = 0xdd; // 0b11011101 = (1, 3, 1, 3)
    let ai_01234567 = _mm512_shuffle_i32x4(aeim_0123, aeim_4567, LO_LANES);
    let bj_01234567 = _mm512_shuffle_i32x4(bfjn_0123, bfjn_4567, LO_LANES);
    let ck_01234567 = _mm512_shuffle_i32x4(cgko_0123, cgko_4567, LO_LANES);
    let dl_01234567 = _mm512_shuffle_i32x4(dhlp_0123, dhlp_4567, LO_LANES);
    let em_01234567 = _mm512_shuffle_i32x4(aeim_0123, aeim_4567, HI_LANES);
    let fn_01234567 = _mm512_shuffle_i32x4(bfjn_0123, bfjn_4567, HI_LANES);
    let go_01234567 = _mm512_shuffle_i32x4(cgko_0123, cgko_4567, HI_LANES);
    let hp_01234567 = _mm512_shuffle_i32x4(dhlp_0123, dhlp_4567, HI_LANES);
    let ai_89abcdef = _mm512_shuffle_i32x4(aeim_89ab, aeim_cdef, LO_LANES);
    let bj_89abcdef = _mm512_shuffle_i32x4(bfjn_89ab, bfjn_cdef, LO_LANES);
    let ck_89abcdef = _mm512_shuffle_i32x4(cgko_89ab, cgko_cdef, LO_LANES);
    let dl_89abcdef = _mm512_shuffle_i32x4(dhlp_89ab, dhlp_cdef, LO_LANES);
    let em_89abcdef = _mm512_shuffle_i32x4(aeim_89ab, aeim_cdef, HI_LANES);
    let fn_89abcdef = _mm512_shuffle_i32x4(bfjn_89ab, bfjn_cdef, HI_LANES);
    let go_89abcdef = _mm512_shuffle_i32x4(cgko_89ab, cgko_cdef, HI_LANES);
    let hp_89abcdef = _mm512_shuffle_i32x4(dhlp_89ab, dhlp_cdef, HI_LANES);

    // Finally interleave 128-bit lanes again (the same permutation as the previous pass, but
    // different inputs), producing vectors like:
    //
    // a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15
    [
        _mm512_shuffle_i32x4(ai_01234567, ai_89abcdef, LO_LANES), // a_0123456789abcdef
        _mm512_shuffle_i32x4(bj_01234567, bj_89abcdef, LO_LANES), // b_0123456789abcdef
        _mm512_shuffle_i32x4(ck_01234567, ck_89abcdef, LO_LANES), // c_0123456789abcdef
        _mm512_shuffle_i32x4(dl_01234567, dl_89abcdef, LO_LANES), // d_0123456789abcdef
        _mm512_shuffle_i32x4(em_01234567, em_89abcdef, LO_LANES), // e_0123456789abcdef
        _mm512_shuffle_i32x4(fn_01234567, fn_89abcdef, LO_LANES), // f_0123456789abcdef
        _mm512_shuffle_i32x4(go_01234567, go_89abcdef, LO_LANES), // g_0123456789abcdef
        _mm512_shuffle_i32x4(hp_01234567, hp_89abcdef, LO_LANES), // h_0123456789abcdef
        _mm512_shuffle_i32x4(ai_01234567, ai_89abcdef, HI_LANES), // i_0123456789abcdef
        _mm512_shuffle_i32x4(bj_01234567, bj_89abcdef, HI_LANES), // j_0123456789abcdef
        _mm512_shuffle_i32x4(ck_01234567, ck_89abcdef, HI_LANES), // k_0123456789abcdef
        _mm512_shuffle_i32x4(dl_01234567, dl_89abcdef, HI_LANES), // l_0123456789abcdef
        _mm512_shuffle_i32x4(em_01234567, em_89abcdef, HI_LANES), // m_0123456789abcdef
        _mm512_shuffle_i32x4(fn_01234567, fn_89abcdef, HI_LANES), // n_0123456789abcdef
        _mm512_shuffle_i32x4(go_01234567, go_89abcdef, HI_LANES), // o_0123456789abcdef
        _mm512_shuffle_i32x4(hp_01234567, hp_89abcdef, HI_LANES), // p_0123456789abcdef
    ]
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn xof_16(
    block: &[u8; BLOCK_LEN],
    cv: &[u32; 8],
    counter: u64,
    block_len: u32,
    flags: u32,
    output: &mut [u8; BLOCK_LEN * 16],
) {
    let vecs = xof_inner_16(block, cv, counter, block_len, flags);
    #[inline(always)]
    unsafe fn write_vec(vecs: &[__m512i; 16], out: *mut u8, i: usize) {
        let addr = out.add(64 * i) as *mut i32;
        _mm512_storeu_si512(addr, vecs[i]);
    }
    write_vec(&vecs, output.as_mut_ptr(), 0x0);
    write_vec(&vecs, output.as_mut_ptr(), 0x1);
    write_vec(&vecs, output.as_mut_ptr(), 0x2);
    write_vec(&vecs, output.as_mut_ptr(), 0x3);
    write_vec(&vecs, output.as_mut_ptr(), 0x4);
    write_vec(&vecs, output.as_mut_ptr(), 0x5);
    write_vec(&vecs, output.as_mut_ptr(), 0x6);
    write_vec(&vecs, output.as_mut_ptr(), 0x7);
    write_vec(&vecs, output.as_mut_ptr(), 0x8);
    write_vec(&vecs, output.as_mut_ptr(), 0x9);
    write_vec(&vecs, output.as_mut_ptr(), 0xa);
    write_vec(&vecs, output.as_mut_ptr(), 0xb);
    write_vec(&vecs, output.as_mut_ptr(), 0xc);
    write_vec(&vecs, output.as_mut_ptr(), 0xd);
    write_vec(&vecs, output.as_mut_ptr(), 0xe);
    write_vec(&vecs, output.as_mut_ptr(), 0xf);
}

#[test]
fn test_xof_16() {
    if !crate::platform::avx512_detected() {
        return;
    }
    let block_len = 63;
    let mut block = [0; 64];
    crate::test::paint_test_input(&mut block[..block_len as usize]); // all but last byte
    let flags = (crate::CHUNK_START | crate::CHUNK_END | crate::ROOT) as u32;

    // Test a few different initial counter values.
    // - 0: The base case.
    // - u32::MAX: The low word of the counter overflows for all inputs except the first.
    // - i32::MAX: *No* overflow. But carry bugs in tricky SIMD code can screw this up, if you XOR
    //   when you're supposed to ANDNOT...
    let initial_counters = [0, u32::MAX as u64, i32::MAX as u64];
    for counter in initial_counters {
        dbg!(counter);
        let mut output = [0xff; BLOCK_LEN * 16];
        unsafe {
            xof_16(&block, IV, counter, block_len, flags, &mut output);
        }
        for i in 0..16 {
            dbg!(i);
            let expected = crate::portable::compress_xof(
                IV,
                &block,
                block_len as u8,
                counter + i as u64,
                flags as u8,
            );
            assert_eq!(expected, output[BLOCK_LEN * i..][..BLOCK_LEN]);
        }
    }
}

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn xof_xor_16(
    block: &[u8; BLOCK_LEN],
    cv: &[u32; 8],
    counter: u64,
    block_len: u32,
    flags: u32,
    output: &mut [u8; BLOCK_LEN * 16],
) {
    let vecs = xof_inner_16(block, cv, counter, block_len, flags);
    #[inline(always)]
    unsafe fn write_vec(vecs: &[__m512i; 16], out: *mut u8, i: usize) {
        let addr = out.add(64 * i) as *mut i32;
        _mm512_storeu_si512(addr, _mm512_xor_si512(vecs[i], _mm512_loadu_si512(addr)));
    }
    write_vec(&vecs, output.as_mut_ptr(), 0x0);
    write_vec(&vecs, output.as_mut_ptr(), 0x1);
    write_vec(&vecs, output.as_mut_ptr(), 0x2);
    write_vec(&vecs, output.as_mut_ptr(), 0x3);
    write_vec(&vecs, output.as_mut_ptr(), 0x4);
    write_vec(&vecs, output.as_mut_ptr(), 0x5);
    write_vec(&vecs, output.as_mut_ptr(), 0x6);
    write_vec(&vecs, output.as_mut_ptr(), 0x7);
    write_vec(&vecs, output.as_mut_ptr(), 0x8);
    write_vec(&vecs, output.as_mut_ptr(), 0x9);
    write_vec(&vecs, output.as_mut_ptr(), 0xa);
    write_vec(&vecs, output.as_mut_ptr(), 0xb);
    write_vec(&vecs, output.as_mut_ptr(), 0xc);
    write_vec(&vecs, output.as_mut_ptr(), 0xd);
    write_vec(&vecs, output.as_mut_ptr(), 0xe);
    write_vec(&vecs, output.as_mut_ptr(), 0xf);
}

#[test]
fn test_xof_xor_16() {
    if !crate::platform::avx512_detected() {
        return;
    }
    let block_len = 63;
    let mut block = [0; 64];
    crate::test::paint_test_input(&mut block[..block_len as usize]); // all but last byte
    let flags = (crate::CHUNK_START | crate::CHUNK_END | crate::ROOT) as u32;

    // Test a few different initial counter values.
    // - 0: The base case.
    // - u32::MAX: The low word of the counter overflows for all inputs except the first.
    // - i32::MAX: *No* overflow. But carry bugs in tricky SIMD code can screw this up, if you XOR
    //   when you're supposed to ANDNOT...
    let initial_counters = [0, u32::MAX as u64, i32::MAX as u64];
    for counter in initial_counters {
        dbg!(counter);
        let mut initial_output_buffer = [0; BLOCK_LEN * 16];
        crate::test::paint_test_input(&mut initial_output_buffer);
        let mut output = initial_output_buffer;
        unsafe {
            xof_xor_16(&block, IV, counter, block_len, flags, &mut output);
        }
        for i in 0..16 {
            dbg!(i);
            let mut expected_block = crate::portable::compress_xof(
                IV,
                &block,
                block_len as u8,
                counter + i as u64,
                flags as u8,
            );
            for j in 0..expected_block.len() {
                expected_block[j] ^= initial_output_buffer[i * BLOCK_LEN + j];
            }
            assert_eq!(expected_block, output[BLOCK_LEN * i..][..BLOCK_LEN]);
        }
    }
}
