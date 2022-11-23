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
    // lower half final xors
    // NOTE: upper half final xors done by XOF callers
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
        core::mem::transmute(vecs)
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

#[target_feature(enable = "avx512f,avx512vl")]
pub unsafe fn chunks_16(
    _message: &[u8; 16 * CHUNK_LEN],
    _key: &[u32; 8],
    _counter: u64,
    _flags: u32,
) -> [__m512i; 8] {
    todo!();
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

// returns (low_words, high_words)
#[inline]
#[target_feature(enable = "avx512f,avx512vl")]
unsafe fn incrementing_counter(initial_value: u64) -> (__m512i, __m512i) {
    let mut values = [initial_value; 16];
    for i in 0..16 {
        // 64-bit overflow here is not supported and will panic in debug mode.
        values[i] += i as u64;
    }
    let low_words: __m512i = mem::transmute(values.map(|v| v as u32));
    let high_words: __m512i = mem::transmute(values.map(|v| (v >> 32) as u32));
    (low_words, high_words)
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

    // Interleave 32-bit words. This results in vectors like:
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

    // Interleave 64-bit words. This gives us our intermediate goal, which is vectors like:
    // a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
    [
        _mm512_unpacklo_epi64(abefijmn_01, abefijmn_23), // aeim_0123
        _mm512_unpackhi_epi64(abefijmn_01, abefijmn_23), // bfjn_0123
        _mm512_unpacklo_epi64(cdghklop_01, cdghklop_23), // cgko_0123
        _mm512_unpackhi_epi64(cdghklop_01, cdghklop_23), // dhlp_0123
        _mm512_unpacklo_epi64(abefijmn_45, abefijmn_67), // aeim_4567
        _mm512_unpackhi_epi64(abefijmn_45, abefijmn_67), // bfjn_4567
        _mm512_unpacklo_epi64(cdghklop_45, cdghklop_67), // cgko_4567
        _mm512_unpackhi_epi64(cdghklop_45, cdghklop_67), // dhlp_4567
        _mm512_unpacklo_epi64(abefijmn_89, abefijmn_ab), // aeim_89ab
        _mm512_unpackhi_epi64(abefijmn_89, abefijmn_ab), // bfjn_89ab
        _mm512_unpacklo_epi64(cdghklop_89, cdghklop_ab), // cgko_89ab
        _mm512_unpackhi_epi64(cdghklop_89, cdghklop_ab), // dhlp_89ab
        _mm512_unpacklo_epi64(abefijmn_cd, abefijmn_ef), // aeim_cdef
        _mm512_unpackhi_epi64(abefijmn_cd, abefijmn_ef), // bfjn_cdef
        _mm512_unpacklo_epi64(cdghklop_cd, cdghklop_ef), // cgko_cdef
        _mm512_unpackhi_epi64(cdghklop_cd, cdghklop_ef), // dhlp_cdef
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
    unsafe fn write_4_lanes<const LANE: i32>(vecs: &[__m512i; 16], first_vec: usize, out: *mut u8) {
        _mm_storeu_epi32(
            out.add(0 * 16) as *mut i32,
            _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 0]),
        );
        _mm_storeu_epi32(
            out.add(1 * 16) as *mut i32,
            _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 4]),
        );
        _mm_storeu_epi32(
            out.add(2 * 16) as *mut i32,
            _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 8]),
        );
        _mm_storeu_epi32(
            out.add(3 * 16) as *mut i32,
            _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 12]),
        );
    }

    let vecs = xof_inner_16(block, cv, counter, block_len, flags);
    write_4_lanes::<0>(&vecs, 0, output.as_mut_ptr().add(0 * 64));
    write_4_lanes::<0>(&vecs, 1, output.as_mut_ptr().add(1 * 64));
    write_4_lanes::<0>(&vecs, 2, output.as_mut_ptr().add(2 * 64));
    write_4_lanes::<0>(&vecs, 3, output.as_mut_ptr().add(3 * 64));
    write_4_lanes::<1>(&vecs, 0, output.as_mut_ptr().add(4 * 64));
    write_4_lanes::<1>(&vecs, 1, output.as_mut_ptr().add(5 * 64));
    write_4_lanes::<1>(&vecs, 2, output.as_mut_ptr().add(6 * 64));
    write_4_lanes::<1>(&vecs, 3, output.as_mut_ptr().add(7 * 64));
    write_4_lanes::<2>(&vecs, 0, output.as_mut_ptr().add(8 * 64));
    write_4_lanes::<2>(&vecs, 1, output.as_mut_ptr().add(9 * 64));
    write_4_lanes::<2>(&vecs, 2, output.as_mut_ptr().add(10 * 64));
    write_4_lanes::<2>(&vecs, 3, output.as_mut_ptr().add(11 * 64));
    write_4_lanes::<3>(&vecs, 0, output.as_mut_ptr().add(12 * 64));
    write_4_lanes::<3>(&vecs, 1, output.as_mut_ptr().add(13 * 64));
    write_4_lanes::<3>(&vecs, 2, output.as_mut_ptr().add(14 * 64));
    write_4_lanes::<3>(&vecs, 3, output.as_mut_ptr().add(15 * 64));
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
        let mut output = [0; BLOCK_LEN * 16];
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
    unsafe fn write_4_lanes<const LANE: i32>(vecs: &[__m512i; 16], first_vec: usize, out: *mut u8) {
        _mm_storeu_epi32(
            out.add(0 * 16) as *mut i32,
            // TODO: Does using a VEX intrinsic make a difference here?
            _mm_xor_epi32(
                _mm_loadu_epi32(out.add(0 * 16) as *mut i32),
                _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 0]),
            ),
        );
        _mm_storeu_epi32(
            out.add(1 * 16) as *mut i32,
            _mm_xor_epi32(
                _mm_loadu_epi32(out.add(1 * 16) as *mut i32),
                _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 4]),
            ),
        );
        _mm_storeu_epi32(
            out.add(2 * 16) as *mut i32,
            _mm_xor_epi32(
                _mm_loadu_epi32(out.add(2 * 16) as *mut i32),
                _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 8]),
            ),
        );
        _mm_storeu_epi32(
            out.add(3 * 16) as *mut i32,
            _mm_xor_epi32(
                _mm_loadu_epi32(out.add(3 * 16) as *mut i32),
                _mm512_extracti32x4_epi32::<LANE>(vecs[first_vec + 12]),
            ),
        );
    }

    let vecs = xof_inner_16(block, cv, counter, block_len, flags);
    write_4_lanes::<0>(&vecs, 0, output.as_mut_ptr().add(0 * 64));
    write_4_lanes::<0>(&vecs, 1, output.as_mut_ptr().add(1 * 64));
    write_4_lanes::<0>(&vecs, 2, output.as_mut_ptr().add(2 * 64));
    write_4_lanes::<0>(&vecs, 3, output.as_mut_ptr().add(3 * 64));
    write_4_lanes::<1>(&vecs, 0, output.as_mut_ptr().add(4 * 64));
    write_4_lanes::<1>(&vecs, 1, output.as_mut_ptr().add(5 * 64));
    write_4_lanes::<1>(&vecs, 2, output.as_mut_ptr().add(6 * 64));
    write_4_lanes::<1>(&vecs, 3, output.as_mut_ptr().add(7 * 64));
    write_4_lanes::<2>(&vecs, 0, output.as_mut_ptr().add(8 * 64));
    write_4_lanes::<2>(&vecs, 1, output.as_mut_ptr().add(9 * 64));
    write_4_lanes::<2>(&vecs, 2, output.as_mut_ptr().add(10 * 64));
    write_4_lanes::<2>(&vecs, 3, output.as_mut_ptr().add(11 * 64));
    write_4_lanes::<3>(&vecs, 0, output.as_mut_ptr().add(12 * 64));
    write_4_lanes::<3>(&vecs, 1, output.as_mut_ptr().add(13 * 64));
    write_4_lanes::<3>(&vecs, 2, output.as_mut_ptr().add(14 * 64));
    write_4_lanes::<3>(&vecs, 3, output.as_mut_ptr().add(15 * 64));
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
