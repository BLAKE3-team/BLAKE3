.intel_syntax noprefix

.global _blake3_hash_many_avx512
.global blake3_hash_many_avx512
.global blake3_compress_in_place_avx512
.global _blake3_compress_in_place_avx512
.global blake3_compress_xof_avx512
.global _blake3_compress_xof_avx512
.global blake3_xof_many_avx512
.global _blake3_xof_many_avx512

.section .text
.p2align  6
_blake3_hash_many_avx512:
blake3_hash_many_avx512:
        push rbx
        push rbp
        push rsi
        push rdi
        push r12
        push r13
        push r14
        push r15
        mov rbp, rsp
        sub rsp, 0x1E8
        movaps xmmword ptr [rbp-0xA8], xmm6
        movaps xmmword ptr [rbp-0x98], xmm7
        movaps xmmword ptr [rbp-0x88], xmm8
        movaps xmmword ptr [rbp-0x78], xmm9
        movaps xmmword ptr [rbp-0x68], xmm10
        movaps xmmword ptr [rbp-0x58], xmm11
        movaps xmmword ptr [rbp-0x48], xmm12
        movaps xmmword ptr [rbp-0x38], xmm13
        movaps xmmword ptr [rbp-0x28], xmm14
        movaps xmmword ptr [rbp-0x18], xmm15
        and rsp, 0xFFFFFFFFFFFFFFC0
        mov rax, qword ptr [rbp+0x68]
        movzx ebx, byte ptr [rbp+0x70]
        neg ebx
        kmovw k1, ebx
        vpbroadcastd ymm0, eax
        shr rax, 0x20
        vpbroadcastd ymm1, eax
        vmovdqa32 ymm2 {k1} {z}, ymmword ptr [rip+ADD0+0]
        vmovdqa32 ymm3 {k1} {z}, ymmword ptr [rip+ADD0+32]
        vpaddd ymm2, ymm0, ymm2
        vmovdqa ymmword ptr [rsp], ymm2
        vpaddd ymm3, ymm0, ymm3
        vmovdqa ymmword ptr [rsp+0x20], ymm3
        vpcmpud k2 {k1}, ymm2, ymm0, 0x01
        vpcmpud k3 {k1}, ymm3, ymm0, 0x01
        vmovdqa ymm2, ymm1
        vpaddd ymm2 {k2}, ymm2, dword ptr [rip+ADD1] {1to8}
        vmovdqa ymmword ptr [rsp+0x40], ymm2
        vpaddd ymm1 {k3}, ymm1, dword ptr [rip+ADD1] {1to8}
        vmovdqa ymmword ptr [rsp+0x60], ymm1
        shl r8, 0x06
        mov qword ptr [rsp+0x100], r8
        cmp rdx, 0x08
        jbe 5f
.p2align 5
2:
        vpbroadcastd zmm0, dword ptr [r9]
        vpbroadcastd zmm1, dword ptr [r9+0x04]
        vpbroadcastd zmm2, dword ptr [r9+0x08]
        vpbroadcastd zmm3, dword ptr [r9+0x0C]
        vpbroadcastd zmm4, dword ptr [r9+0x10]
        vpbroadcastd zmm5, dword ptr [r9+0x14]
        vpbroadcastd zmm6, dword ptr [r9+0x18]
        vpbroadcastd zmm7, dword ptr [r9+0x1C]
        movzx eax, byte ptr [rbp+0x78]
        movzx ebx, byte ptr [rbp+0x80]
        or eax, ebx
        xor ebx, ebx
3:
        movzx esi, byte ptr [rbp+0x88]
        or esi, eax
        add rbx, 0x40
        cmp rbx, qword ptr [rsp+0x100]
        cmovz eax, esi
        mov dword ptr [rsp+0x80], eax
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+0x08]
        mov rdi, qword ptr [rcx+0x10]
        mov r8, qword ptr [rcx+0x18]
        mov r10, qword ptr [rcx+0x40]
        vmovdqu32 ymm8, ymmword ptr [rax+rbx*1-0x40]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rbx*1-0x40], 0x01
        vmovdqu32 ymm9, ymmword ptr [rsi+rbx*1-0x40]
        cmp rdx, 0x0A
        jb 4f
        mov r11, qword ptr [rcx+0x48]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rbx*1-0x40], 0x01
4:
        vpunpckldq zmm10, zmm8, zmm9
        vpunpckhdq zmm11, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [rdi+rbx*1-0x40]
        cmp rdx, 0x0B
        jb 4f
        mov r12, qword ptr [rcx+0x50]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rbx*1-0x40], 0x01
4:
        vmovdqu32 ymm9, ymmword ptr [r8+rbx*1-0x40]
        cmp rdx, 0x0C
        jb 4f
        mov r13, qword ptr [rcx+0x58]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rbx*1-0x40], 0x01
4:
        vpunpckldq zmm12, zmm8, zmm9
        vpunpckhdq zmm13, zmm8, zmm9
        mov rax, qword ptr [rcx+0x20]
        mov rsi, qword ptr [rcx+0x28]
        mov rdi, qword ptr [rcx+0x30]
        mov r8, qword ptr [rcx+0x38]
        vmovdqu32 ymm8, ymmword ptr [rax+rbx*1-0x40]
        cmp rdx, 0x0D
        jb 4f
        mov r10, qword ptr [rcx+0x60]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rbx*1-0x40], 0x01
4:
        vmovdqu32 ymm9, ymmword ptr [rsi+rbx*1-0x40]
        cmp rdx, 0x0E
        jb 4f
        mov r11, qword ptr [rcx+0x68]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rbx*1-0x40], 0x01
4:
        vpunpckldq zmm14, zmm8, zmm9
        vpunpckhdq zmm15, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [rdi+rbx*1-0x40]
        cmp rdx, 0x0F
        jb 4f
        mov r12, qword ptr [rcx+0x70]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rbx*1-0x40], 0x01
4:
        vmovdqu32 ymm9, ymmword ptr [r8+rbx*1-0x40]
        cmp rdx, 0x10
        jb 4f
        mov r13, qword ptr [rcx+0x78]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rbx*1-0x40], 0x01
4:
        vpunpckldq zmm16, zmm8, zmm9
        vpunpckhdq zmm17, zmm8, zmm9
        vmovdqa32 zmm8, zmmword ptr [rip+INDEX0]
        vmovdqa32 zmm9, zmmword ptr [rip+INDEX1]
        vpunpcklqdq zmm18, zmm10, zmm12
        vpunpcklqdq zmm20, zmm14, zmm16
        vmovdqa32 zmm19, zmm18
        vpermt2d zmm18, zmm8, zmm20
        vpermt2d zmm19, zmm9, zmm20
        vpunpckhqdq zmm20, zmm10, zmm12
        vpunpckhqdq zmm22, zmm14, zmm16
        vmovdqa32 zmm21, zmm20
        vpermt2d zmm20, zmm8, zmm22
        vpermt2d zmm21, zmm9, zmm22
        vpunpcklqdq zmm10, zmm11, zmm13
        vpunpcklqdq zmm14, zmm15, zmm17
        vmovdqa32 zmm12, zmm10
        vpermt2d zmm10, zmm8, zmm14
        vpermt2d zmm12, zmm9, zmm14
        vpunpckhqdq zmm14, zmm11, zmm13
        vpunpckhqdq zmm22, zmm15, zmm17
        vmovdqa32 zmm16, zmm14
        vpermt2d zmm14, zmm8, zmm22
        vpermt2d zmm16, zmm9, zmm22
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+0x08]
        mov rdi, qword ptr [rcx+0x10]
        mov r8, qword ptr [rcx+0x18]
        mov r10, qword ptr [rcx+0x40]
        vmovdqu32 ymm11, ymmword ptr [rax+rbx*1-0x20]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rbx*1-0x20], 0x01
        vmovdqu32 ymm13, ymmword ptr [rsi+rbx*1-0x20]
        cmp rdx, 0x0A
        jb 4f
        mov r11, qword ptr [rcx+0x48]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r11+rbx*1+0x80]
4:
        vpunpckldq zmm15, zmm11, zmm13
        vpunpckhdq zmm17, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [rdi+rbx*1-0x20]
        cmp rdx, 0x0B
        jb 4f
        mov r12, qword ptr [rcx+0x50]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r13+rbx*1+0x80]
4:
        vmovdqu32 ymm13, ymmword ptr [r8+rbx*1-0x20]
        cmp rdx, 0x0C
        jb 4f
        mov r13, qword ptr [rcx+0x58]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r13+rbx*1+0x80]
4:
        vpunpckldq zmm22, zmm11, zmm13
        vpunpckhdq zmm23, zmm11, zmm13
        prefetcht0 byte ptr [rax+rbx*1+0x80]
        prefetcht0 byte ptr [rsi+rbx*1+0x80]
        prefetcht0 byte ptr [rdi+rbx*1+0x80]
        prefetcht0 byte ptr [r8+rbx*1+0x80]
        prefetcht0 byte ptr [r10+rbx*1+0x80]
        mov rax, qword ptr [rcx+0x20]
        mov rsi, qword ptr [rcx+0x28]
        mov rdi, qword ptr [rcx+0x30]
        mov r8, qword ptr [rcx+0x38]
        vmovdqu32 ymm11, ymmword ptr [rax+rbx*1-0x20]
        cmp rdx, 0x0D
        jb 4f
        mov r10, qword ptr [rcx+0x60]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r10+rbx*1+0x80]
4:
        vmovdqu32 ymm13, ymmword ptr [rsi+rbx*1-0x20]
        cmp rdx, 0x0E
        jb 4f
        mov r11, qword ptr [rcx+0x68]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r11+rbx*1+0x80]
4:
        vpunpckldq zmm24, zmm11, zmm13
        vpunpckhdq zmm25, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [rdi+rbx*1-0x20]
        cmp rdx, 0x0F
        jb 4f
        mov r12, qword ptr [rcx+0x70]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r12+rbx*1+0x80]
4:
        vmovdqu32 ymm13, ymmword ptr [r8+rbx*1-0x20]
        cmp rdx, 0x10
        jb 4f
        mov r13, qword ptr [rcx+0x78]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rbx*1-0x20], 0x01
        prefetcht0 byte ptr [r13+rbx*1+0x80]
4:
        vpunpckldq zmm26, zmm11, zmm13
        vpunpckhdq zmm27, zmm11, zmm13
        prefetcht0 byte ptr [r10+rbx*1+0x80]
        prefetcht0 byte ptr [r11+rbx*1+0x80]
        prefetcht0 byte ptr [r12+rbx*1+0x80]
        prefetcht0 byte ptr [r13+rbx*1+0x80]
        vpunpcklqdq zmm11, zmm15, zmm22
        vpunpcklqdq zmm28, zmm24, zmm26
        vmovdqa32 zmm13, zmm11
        vpermt2d zmm11, zmm8, zmm28
        vpermt2d zmm13, zmm9, zmm28
        vpunpckhqdq zmm28, zmm15, zmm22
        vpunpckhqdq zmm30, zmm24, zmm26
        vmovdqa32 zmm29, zmm28
        vpermt2d zmm28, zmm8, zmm30
        vpermt2d zmm29, zmm9, zmm30
        vpunpcklqdq zmm15, zmm17, zmm23
        vpunpcklqdq zmm24, zmm25, zmm27
        vmovdqa32 zmm22, zmm15
        vpermt2d zmm15, zmm8, zmm24
        vpermt2d zmm22, zmm9, zmm24
        vpunpckhqdq zmm24, zmm17, zmm23
        vpunpckhqdq zmm26, zmm25, zmm27
        vpermi2d zmm8, zmm24, zmm26
        vpermi2d zmm9, zmm24, zmm26
        vpbroadcastd zmm17, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd zmm23, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd zmm24, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd zmm25, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa32 zmm26, zmmword ptr [rsp]
        vmovdqa32 zmm27, zmmword ptr [rsp+0x40]
        vpbroadcastd zmm30, dword ptr [rip+BLAKE3_BLOCK_LEN]
        vpbroadcastd zmm31, dword ptr [rsp+0x80]
        mov al, 0x07
4:
        vpaddd zmm0, zmm0, zmm18
        vpaddd zmm1, zmm1, zmm10
        vpaddd zmm2, zmm2, zmm19
        vpaddd zmm3, zmm3, zmm12
        vmovdqa32 zmmword ptr [rsp+0x80], zmm18
        vmovdqa32 zmmword ptr [rsp+0xC0], zmm12
        vpaddd zmm0, zmm0, zmm4
        vpaddd zmm1, zmm1, zmm5
        vpaddd zmm2, zmm2, zmm6
        vpaddd zmm3, zmm3, zmm7
        vpxord zmm26, zmm26, zmm0
        vmovdqa32 zmm18, zmm10
        vpxord zmm27, zmm27, zmm1
        vpxord zmm30, zmm30, zmm2
        vpxord zmm31, zmm31, zmm3
        vprord zmm26, zmm26, 0x10
        vprord zmm27, zmm27, 0x10
        vprord zmm30, zmm30, 0x10
        vprord zmm31, zmm31, 0x10
        vpaddd zmm17, zmm17, zmm26
        vmovdqa32 zmm12, zmm19
        vpaddd zmm23, zmm23, zmm27
        vpaddd zmm24, zmm24, zmm30
        vpaddd zmm25, zmm25, zmm31
        vpxord zmm4, zmm4, zmm17
        vpxord zmm5, zmm5, zmm23
        vpxord zmm6, zmm6, zmm24
        vpxord zmm7, zmm7, zmm25
        vprord zmm4, zmm4, 0x0C
        vprord zmm5, zmm5, 0x0C
        vprord zmm6, zmm6, 0x0C
        vprord zmm7, zmm7, 0x0C
        vpaddd zmm0, zmm0, zmm20
        vpaddd zmm1, zmm1, zmm14
        vpaddd zmm2, zmm2, zmm21
        vpaddd zmm3, zmm3, zmm16
        vpaddd zmm0, zmm0, zmm4
        vmovdqa32 zmm10, zmm14
        vpaddd zmm1, zmm1, zmm5
        vpaddd zmm2, zmm2, zmm6
        vpaddd zmm3, zmm3, zmm7
        vpxord zmm26, zmm26, zmm0
        vpxord zmm27, zmm27, zmm1
        vpxord zmm30, zmm30, zmm2
        vpxord zmm31, zmm31, zmm3
        vprord zmm26, zmm26, 0x08
        vmovdqa32 zmm19, zmm16
        vprord zmm27, zmm27, 0x08
        vprord zmm30, zmm30, 0x08
        vprord zmm31, zmm31, 0x08
        vpaddd zmm17, zmm17, zmm26
        vpaddd zmm23, zmm23, zmm27
        vpaddd zmm24, zmm24, zmm30
        vpaddd zmm25, zmm25, zmm31
        vpxord zmm4, zmm4, zmm17
        vmovdqa32 zmm14, zmm15
        vpxord zmm5, zmm5, zmm23
        vpxord zmm6, zmm6, zmm24
        vpxord zmm7, zmm7, zmm25
        vprord zmm4, zmm4, 0x07
        vprord zmm5, zmm5, 0x07
        vprord zmm6, zmm6, 0x07
        vprord zmm7, zmm7, 0x07
        vpaddd zmm0, zmm0, zmm11
        vmovdqa32 zmm16, zmm29
        vpaddd zmm1, zmm1, zmm15
        vpaddd zmm2, zmm2, zmm13
        vpaddd zmm3, zmm3, zmm22
        vpaddd zmm0, zmm0, zmm5
        vpaddd zmm1, zmm1, zmm6
        vpaddd zmm2, zmm2, zmm7
        vpaddd zmm3, zmm3, zmm4
        vpxord zmm31, zmm31, zmm0
        vmovdqa32 zmm15, zmm13
        vpxord zmm26, zmm26, zmm1
        vpxord zmm27, zmm27, zmm2
        vpxord zmm30, zmm30, zmm3
        vprord zmm31, zmm31, 0x10
        vprord zmm26, zmm26, 0x10
        vprord zmm27, zmm27, 0x10
        vprord zmm30, zmm30, 0x10
        vpaddd zmm24, zmm24, zmm31
        vmovdqa32 zmm13, zmm28
        vpaddd zmm25, zmm25, zmm26
        vpaddd zmm17, zmm17, zmm27
        vpaddd zmm23, zmm23, zmm30
        vpxord zmm5, zmm5, zmm24
        vpxord zmm6, zmm6, zmm25
        vpxord zmm7, zmm7, zmm17
        vpxord zmm4, zmm4, zmm23
        vprord zmm5, zmm5, 0x0C
        vprord zmm6, zmm6, 0x0C
        vprord zmm7, zmm7, 0x0C
        vprord zmm4, zmm4, 0x0C
        vpaddd zmm0, zmm0, zmm28
        vpaddd zmm1, zmm1, zmm8
        vpaddd zmm2, zmm2, zmm29
        vmovdqa32 zmm29, zmm22
        vpaddd zmm3, zmm3, zmm9
        vpaddd zmm0, zmm0, zmm5
        vpaddd zmm1, zmm1, zmm6
        vpaddd zmm2, zmm2, zmm7
        vmovdqa32 zmm28, zmm8
        vpaddd zmm3, zmm3, zmm4
        vpxord zmm31, zmm31, zmm0
        vpxord zmm26, zmm26, zmm1
        vmovdqa32 zmm22, zmm9
        vpxord zmm27, zmm27, zmm2
        vpxord zmm30, zmm30, zmm3
        vprord zmm31, zmm31, 0x08
        vmovdqa32 zmm8, zmm21
        vprord zmm26, zmm26, 0x08
        vprord zmm27, zmm27, 0x08
        vprord zmm30, zmm30, 0x08
        vmovdqa32 zmm9, zmm11
        vpaddd zmm24, zmm24, zmm31
        vpaddd zmm25, zmm25, zmm26
        vpaddd zmm17, zmm17, zmm27
        vpaddd zmm23, zmm23, zmm30
        vmovdqa32 zmm11, zmm20
        vpxord zmm5, zmm5, zmm24
        vpxord zmm6, zmm6, zmm25
        vpxord zmm7, zmm7, zmm17
        vpxord zmm4, zmm4, zmm23
        vmovdqa32 zmm21, zmmword ptr [rsp+0x80]
        vprord zmm5, zmm5, 0x07
        vprord zmm6, zmm6, 0x07
        vprord zmm7, zmm7, 0x07
        vprord zmm4, zmm4, 0x07
        vmovdqa32 zmm20, zmmword ptr [rsp+0xC0]
        dec al
        jnz 4b
        vpxord zmm0, zmm0, zmm17
        vpxord zmm1, zmm1, zmm23
        vpxord zmm2, zmm2, zmm24
        vpxord zmm3, zmm3, zmm25
        vpxord zmm4, zmm4, zmm26
        vpxord zmm5, zmm5, zmm27
        vpxord zmm6, zmm6, zmm30
        vpxord zmm7, zmm7, zmm31
        movzx eax, byte ptr [rbp+0x78]
        cmp rbx, qword ptr [rsp+0x100]
        jb 3b
        mov rsi, qword ptr [rbp+0x90]
        vpunpckldq zmm8, zmm0, zmm2
        vpunpckhdq zmm9, zmm0, zmm2
        vpunpckldq zmm10, zmm1, zmm3
        vpunpckhdq zmm11, zmm1, zmm3
        vpunpckldq zmm12, zmm4, zmm6
        vpunpckhdq zmm13, zmm4, zmm6
        vpunpckldq zmm14, zmm5, zmm7
        vpunpckhdq zmm15, zmm5, zmm7
        vpunpckldq zmm0, zmm8, zmm10
        vpunpckhdq zmm1, zmm8, zmm10
        vpunpckldq zmm2, zmm9, zmm11
        vpunpckhdq zmm3, zmm9, zmm11
        vpunpckldq zmm4, zmm12, zmm14
        vpunpckhdq zmm5, zmm12, zmm14
        vpunpckldq zmm6, zmm13, zmm15
        vpunpckhdq zmm7, zmm13, zmm15
        vmovdqa32 zmm16, zmmword ptr [rip+INDEX0]
        vmovdqa32 zmm18, zmmword ptr [rip+INDEX1]
        vmovdqa32 zmm8, zmm0
        vpermt2d zmm8, zmm16, zmm4
        vpermt2d zmm0, zmm18, zmm4
        vmovdqa32 zmm10, zmm1
        vpermt2d zmm10, zmm16, zmm5
        vpermt2d zmm1, zmm18, zmm5
        vmovdqa32 zmm12, zmm2
        vpermt2d zmm12, zmm16, zmm6
        vpermt2d zmm2, zmm18, zmm6
        vmovdqa32 zmm14, zmm3
        vpermt2d zmm14, zmm16, zmm7
        vpermt2d zmm3, zmm18, zmm7
        vextracti64x4 ymmword ptr [rsi], zmm8, 0x00
        vextracti64x4 ymmword ptr [rsi+0x20], zmm10, 0x00
        vextracti64x4 ymmword ptr [rsi+0x40], zmm12, 0x00
        vextracti64x4 ymmword ptr [rsi+0x60], zmm14, 0x00
        vextracti64x4 ymmword ptr [rsi+0x80], zmm0, 0x00
        vextracti64x4 ymmword ptr [rsi+0xA0], zmm1, 0x00
        vextracti64x4 ymmword ptr [rsi+0xC0], zmm2, 0x00
        vextracti64x4 ymmword ptr [rsi+0xE0], zmm3, 0x00
        vextracti64x4 ymmword ptr [rsi+0x100], zmm8, 0x01
        cmp rdx, 0x0A
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x120], zmm10, 0x01
        cmp rdx, 0x0B
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x140], zmm12, 0x01
        cmp rdx, 0x0C
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x160], zmm14, 0x01
        cmp rdx, 0x0D
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x180], zmm0, 0x01
        cmp rdx, 0x0E
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x1A0], zmm1, 0x01
        cmp rdx, 0x0F
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x1C0], zmm2, 0x01
        cmp rdx, 0x10
        jb 9f
        vextracti64x4 ymmword ptr [rsi+0x1E0], zmm3, 0x01
        vmovdqa32 zmm8, zmmword ptr [rsp]
        vmovdqa32 zmm9, zmmword ptr [rsp+0x40]
        vmovdqa32 zmm10, zmm8
        vpaddd zmm10 {k1}, zmm8, dword ptr [rip+ADD16] {1to16}
        vpcmpud k2 {k1}, zmm10, zmm8, 0x01
        vpaddd zmm9 {k2}, zmm9, dword ptr [rip+ADD1] {1to16}
        vmovdqa32 zmmword ptr [rsp], zmm10
        vmovdqa32 zmmword ptr [rsp+0x40], zmm9
        add rsi, 0x200
        mov qword ptr [rbp+0x90], rsi
        add rcx, 0x80
        sub rdx, 0x10
        cmp rdx, 0x08
        jnbe 2b
        test edx, edx
        jnz 5f
9:
        vzeroupper
        movaps xmm6, xmmword ptr [rbp-0xA8]
        movaps xmm7, xmmword ptr [rbp-0x98]
        movaps xmm8, xmmword ptr [rbp-0x88]
        movaps xmm9, xmmword ptr [rbp-0x78]
        movaps xmm10, xmmword ptr [rbp-0x68]
        movaps xmm11, xmmword ptr [rbp-0x58]
        movaps xmm12, xmmword ptr [rbp-0x48]
        movaps xmm13, xmmword ptr [rbp-0x38]
        movaps xmm14, xmmword ptr [rbp-0x28]
        movaps xmm15, xmmword ptr [rbp-0x18]
        mov rsp, rbp
        pop r15
        pop r14
        pop r13
        pop r12
        pop rdi
        pop rsi
        pop rbp
        pop rbx
        ret
.p2align 6
5:
        cmp dl, 0x04
        jbe 3f
        vpbroadcastd ymm0, dword ptr [r9]
        vpbroadcastd ymm1, dword ptr [r9+0x04]
        vpbroadcastd ymm2, dword ptr [r9+0x08]
        vpbroadcastd ymm3, dword ptr [r9+0x0C]
        vpbroadcastd ymm4, dword ptr [r9+0x10]
        vpbroadcastd ymm5, dword ptr [r9+0x14]
        vpbroadcastd ymm6, dword ptr [r9+0x18]
        vpbroadcastd ymm7, dword ptr [r9+0x1C]
        movzx eax, byte ptr [rbp+0x78]
        movzx ebx, byte ptr [rbp+0x80]
        or eax, ebx
        xor ebx, ebx
2:
        movzx esi, byte ptr [rbp+0x88]
        or esi, eax
        add rbx, 0x40
        cmp rbx, qword ptr [rsp+0x100]
        cmovz eax, esi
        mov dword ptr [rsp+0x80], eax
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+0x20]
        vmovups xmm8, xmmword ptr [rax+rbx*1-0x40]
        vinserti32x4 ymm8, ymm8, xmmword ptr [rsi+rbx*1-0x40], 0x01
        vmovups xmm12, xmmword ptr [rax+rbx*1-0x30]
        vinserti32x4 ymm12, ymm12, xmmword ptr [rsi+rbx*1-0x30], 0x01
        mov rax, qword ptr [rcx+0x08]
        vmovups xmm9, xmmword ptr [rax+rbx*1-0x40]
        vmovups xmm13, xmmword ptr [rax+rbx*1-0x30]
        cmp dl, 0x06
        jb 4f
        mov rsi, qword ptr [rcx+0x28]
        vinserti32x4 ymm9, ymm9, xmmword ptr [rsi+rbx*1-0x40], 0x01
        vinserti32x4 ymm13, ymm13, xmmword ptr [rsi+rbx*1-0x30], 0x01
4:
        mov rax, qword ptr [rcx+0x10]
        vmovups xmm10, xmmword ptr [rax+rbx*1-0x40]
        vmovups xmm14, xmmword ptr [rax+rbx*1-0x30]
        cmp dl, 0x07
        jb 4f
        mov rsi, qword ptr [rcx+0x30]
        vinserti32x4 ymm10, ymm10, xmmword ptr [rsi+rbx*1-0x40], 0x01
        vinserti32x4 ymm14, ymm14, xmmword ptr [rsi+rbx*1-0x30], 0x01
4:
        mov rax, qword ptr [rcx+0x18]
        vmovups xmm11, xmmword ptr [rax+rbx*1-0x40]
        vmovups xmm15, xmmword ptr [rax+rbx*1-0x30]
        cmp dl, 0x08
        jb 4f
        mov rsi, qword ptr [rcx+0x38]
        vinserti32x4 ymm11, ymm11, xmmword ptr [rsi+rbx*1-0x40], 0x01
        vinserti32x4 ymm15, ymm15, xmmword ptr [rsi+rbx*1-0x30], 0x01
4:
        vpunpckldq ymm24, ymm8, ymm9
        vpunpckhdq ymm9, ymm8, ymm9
        vpunpckldq ymm8, ymm10, ymm11
        vpunpckhdq ymm11, ymm10, ymm11
        vpunpckldq ymm10, ymm12, ymm13
        vpunpckhdq ymm13, ymm12, ymm13
        vpunpckldq ymm12, ymm14, ymm15
        vpunpckhdq ymm15, ymm14, ymm15
        vshufps ymm14, ymm24, ymm8, 0x44
        vshufps ymm8, ymm24, ymm8, 0xEE
        vshufps ymm24, ymm9, ymm11, 0x44
        vshufps ymm11, ymm9, ymm11, 0xEE
        vshufps ymm9, ymm10, ymm12, 0x44
        vshufps ymm12, ymm10, ymm12, 0xEE
        vshufps ymm10, ymm13, ymm15, 0x44
        vshufps ymm15, ymm13, ymm15, 0xEE
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+0x20]
        vmovups xmm16, xmmword ptr [rax+rbx*1-0x20]
        vinserti32x4 ymm16, ymm16, xmmword ptr [rsi+rbx*1-0x20], 0x01
        vmovups xmm20, xmmword ptr [rax+rbx*1-0x10]
        vinserti32x4 ymm20, ymm20, xmmword ptr [rsi+rbx*1-0x10], 0x01
        mov rax, qword ptr [rcx+0x08]
        vmovups xmm17, xmmword ptr [rax+rbx*1-0x20]
        vmovups xmm21, xmmword ptr [rax+rbx*1-0x10]
        cmp dl, 0x06
        jb 4f
        mov rsi, qword ptr [rcx+0x28]
        vinserti32x4 ymm17, ymm17, xmmword ptr [rsi+rbx*1-0x20], 0x01
        vinserti32x4 ymm21, ymm21, xmmword ptr [rsi+rbx*1-0x10], 0x01
4:
        mov rax, qword ptr [rcx+0x10]
        vmovups xmm18, xmmword ptr [rax+rbx*1-0x20]
        vmovups xmm22, xmmword ptr [rax+rbx*1-0x10]
        cmp dl, 0x07
        jb 4f
        mov rsi, qword ptr [rcx+0x30]
        vinserti32x4 ymm18, ymm18, xmmword ptr [rsi+rbx*1-0x20], 0x01
        vinserti32x4 ymm22, ymm22, xmmword ptr [rsi+rbx*1-0x10], 0x01
4:
        mov rax, qword ptr [rcx+0x18]
        vmovups xmm19, xmmword ptr [rax+rbx*1-0x20]
        vmovups xmm23, xmmword ptr [rax+rbx*1-0x10]
        cmp dl, 0x08
        jb 4f
        mov rsi, qword ptr [rcx+0x38]
        vinserti32x4 ymm19, ymm19, xmmword ptr [rsi+rbx*1-0x20], 0x01
        vinserti32x4 ymm23, ymm23, xmmword ptr [rsi+rbx*1-0x10], 0x01
4:
        vpunpckldq ymm13, ymm16, ymm17
        vpunpckhdq ymm17, ymm16, ymm17
        vpunpckldq ymm16, ymm18, ymm19
        vpunpckhdq ymm19, ymm18, ymm19
        vpunpckldq ymm18, ymm20, ymm21
        vpunpckhdq ymm21, ymm20, ymm21
        vpunpckldq ymm20, ymm22, ymm23
        vpunpckhdq ymm23, ymm22, ymm23
        vshufps ymm22, ymm13, ymm16, 0x44
        vshufps ymm16, ymm13, ymm16, 0xEE
        vshufps ymm13, ymm17, ymm19, 0x44
        vshufps ymm19, ymm17, ymm19, 0xEE
        vshufps ymm17, ymm18, ymm20, 0x44
        vshufps ymm20, ymm18, ymm20, 0xEE
        vshufps ymm18, ymm21, ymm23, 0x44
        vshufps ymm23, ymm21, ymm23, 0xEE
        vpbroadcastd ymm21, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd ymm25, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd ymm26, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd ymm27, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa32 ymm28, ymmword ptr [rsp]
        vmovdqa32 ymm29, ymmword ptr [rsp+0x40]
        vpbroadcastd ymm30, dword ptr [rip+BLAKE3_BLOCK_LEN]
        vpbroadcastd ymm31, dword ptr [rsp+0x80]
        mov al, 0x07
4:
        vpaddd ymm0, ymm0, ymm14
        vpaddd ymm1, ymm1, ymm24
        vpaddd ymm2, ymm2, ymm9
        vpaddd ymm3, ymm3, ymm10
        vmovdqa32 ymmword ptr [rsp+0x80], ymm14
        vmovdqa32 ymmword ptr [rsp+0xC0], ymm10
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm28, ymm28, ymm0
        vmovdqa32 ymm14, ymm24
        vpxord ymm29, ymm29, ymm1
        vpxord ymm30, ymm30, ymm2
        vpxord ymm31, ymm31, ymm3
        vprord ymm28, ymm28, 0x10
        vprord ymm29, ymm29, 0x10
        vprord ymm30, ymm30, 0x10
        vprord ymm31, ymm31, 0x10
        vpaddd ymm21, ymm21, ymm28
        vmovdqa32 ymm10, ymm9
        vpaddd ymm25, ymm25, ymm29
        vpaddd ymm26, ymm26, ymm30
        vpaddd ymm27, ymm27, ymm31
        vpxord ymm4, ymm4, ymm21
        vpxord ymm5, ymm5, ymm25
        vpxord ymm6, ymm6, ymm26
        vpxord ymm7, ymm7, ymm27
        vprord ymm4, ymm4, 0x0C
        vprord ymm5, ymm5, 0x0C
        vprord ymm6, ymm6, 0x0C
        vprord ymm7, ymm7, 0x0C
        vpaddd ymm0, ymm0, ymm8
        vpaddd ymm1, ymm1, ymm11
        vpaddd ymm2, ymm2, ymm12
        vpaddd ymm3, ymm3, ymm15
        vpaddd ymm0, ymm0, ymm4
        vmovdqa32 ymm24, ymm11
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm28, ymm28, ymm0
        vpxord ymm29, ymm29, ymm1
        vpxord ymm30, ymm30, ymm2
        vpxord ymm31, ymm31, ymm3
        vprord ymm28, ymm28, 0x08
        vmovdqa32 ymm9, ymm15
        vprord ymm29, ymm29, 0x08
        vprord ymm30, ymm30, 0x08
        vprord ymm31, ymm31, 0x08
        vpaddd ymm21, ymm21, ymm28
        vpaddd ymm25, ymm25, ymm29
        vpaddd ymm26, ymm26, ymm30
        vpaddd ymm27, ymm27, ymm31
        vpxord ymm4, ymm4, ymm21
        vmovdqa32 ymm11, ymm13
        vpxord ymm5, ymm5, ymm25
        vpxord ymm6, ymm6, ymm26
        vpxord ymm7, ymm7, ymm27
        vprord ymm4, ymm4, 0x07
        vprord ymm5, ymm5, 0x07
        vprord ymm6, ymm6, 0x07
        vprord ymm7, ymm7, 0x07
        vpaddd ymm0, ymm0, ymm22
        vmovdqa32 ymm15, ymm20
        vpaddd ymm1, ymm1, ymm13
        vpaddd ymm2, ymm2, ymm17
        vpaddd ymm3, ymm3, ymm18
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm1, ymm1, ymm6
        vpaddd ymm2, ymm2, ymm7
        vpaddd ymm3, ymm3, ymm4
        vpxord ymm31, ymm31, ymm0
        vmovdqa32 ymm13, ymm17
        vpxord ymm28, ymm28, ymm1
        vpxord ymm29, ymm29, ymm2
        vpxord ymm30, ymm30, ymm3
        vprord ymm31, ymm31, 0x10
        vprord ymm28, ymm28, 0x10
        vprord ymm29, ymm29, 0x10
        vprord ymm30, ymm30, 0x10
        vpaddd ymm26, ymm26, ymm31
        vmovdqa32 ymm17, ymm16
        vpaddd ymm27, ymm27, ymm28
        vpaddd ymm21, ymm21, ymm29
        vpaddd ymm25, ymm25, ymm30
        vpxord ymm5, ymm5, ymm26
        vpxord ymm6, ymm6, ymm27
        vpxord ymm7, ymm7, ymm21
        vpxord ymm4, ymm4, ymm25
        vprord ymm5, ymm5, 0x0C
        vprord ymm6, ymm6, 0x0C
        vprord ymm7, ymm7, 0x0C
        vprord ymm4, ymm4, 0x0C
        vpaddd ymm0, ymm0, ymm16
        vpaddd ymm1, ymm1, ymm19
        vpaddd ymm2, ymm2, ymm20
        vmovdqa32 ymm20, ymm18
        vpaddd ymm3, ymm3, ymm23
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm1, ymm1, ymm6
        vpaddd ymm2, ymm2, ymm7
        vmovdqa32 ymm16, ymm19
        vpaddd ymm3, ymm3, ymm4
        vpxord ymm31, ymm31, ymm0
        vpxord ymm28, ymm28, ymm1
        vmovdqa32 ymm18, ymm23
        vpxord ymm29, ymm29, ymm2
        vpxord ymm30, ymm30, ymm3
        vprord ymm31, ymm31, 0x08
        vmovdqa32 ymm19, ymm12
        vprord ymm28, ymm28, 0x08
        vprord ymm29, ymm29, 0x08
        vprord ymm30, ymm30, 0x08
        vmovdqa32 ymm23, ymm22
        vpaddd ymm26, ymm26, ymm31
        vpaddd ymm27, ymm27, ymm28
        vpaddd ymm21, ymm21, ymm29
        vpaddd ymm25, ymm25, ymm30
        vmovdqa32 ymm22, ymm8
        vpxord ymm5, ymm5, ymm26
        vpxord ymm6, ymm6, ymm27
        vpxord ymm7, ymm7, ymm21
        vpxord ymm4, ymm4, ymm25
        vmovdqa32 ymm12, ymmword ptr [rsp+0x80]
        vprord ymm5, ymm5, 0x07
        vprord ymm6, ymm6, 0x07
        vprord ymm7, ymm7, 0x07
        vprord ymm4, ymm4, 0x07
        vmovdqa32 ymm8, ymmword ptr [rsp+0xC0]
        dec al
        jnz 4b
        vpxord ymm0, ymm0, ymm21
        vpxord ymm1, ymm1, ymm25
        vpxord ymm2, ymm2, ymm26
        vpxord ymm3, ymm3, ymm27
        vpxord ymm4, ymm4, ymm28
        vpxord ymm5, ymm5, ymm29
        vpxord ymm6, ymm6, ymm30
        vpxord ymm7, ymm7, ymm31
        movzx eax, byte ptr [rbp+0x78]
        cmp rbx, qword ptr [rsp+0x100]
        jb 2b
        mov rsi, qword ptr [rbp+0x90]
        vpunpckldq ymm8, ymm0, ymm1
        vpunpckldq ymm9, ymm2, ymm3
        vpunpckhdq ymm10, ymm0, ymm1
        vpunpckldq ymm11, ymm4, ymm5
        vpunpckldq ymm0, ymm6, ymm7
        vshufps ymm12, ymm8, ymm9, 0x4E
        vpblendd ymm1, ymm8, ymm12, 0xCC
        vshufps ymm8, ymm11, ymm0, 0x4E
        vpunpckhdq ymm13, ymm2, ymm3
        vpblendd ymm2, ymm11, ymm8, 0xCC
        vpblendd ymm3, ymm12, ymm9, 0xCC
        vperm2i128 ymm12, ymm1, ymm2, 0x20
        vmovdqu ymmword ptr [rsi], ymm12
        vpunpckhdq ymm14, ymm4, ymm5
        vpblendd ymm4, ymm8, ymm0, 0xCC
        vpunpckhdq ymm15, ymm6, ymm7
        vperm2i128 ymm7, ymm3, ymm4, 0x20
        vmovdqu ymmword ptr [rsi+0x20], ymm7
        vshufps ymm5, ymm10, ymm13, 0x4E
        vpblendd ymm6, ymm5, ymm13, 0xCC
        vshufps ymm13, ymm14, ymm15, 0x4E
        vpblendd ymm10, ymm10, ymm5, 0xCC
        vpblendd ymm14, ymm14, ymm13, 0xCC
        vperm2i128 ymm8, ymm10, ymm14, 0x20
        vmovdqu ymmword ptr [rsi+0x40], ymm8
        vpblendd ymm15, ymm13, ymm15, 0xCC
        vperm2i128 ymm13, ymm6, ymm15, 0x20
        vmovdqu ymmword ptr [rsi+0x60], ymm13
        vperm2i128 ymm9, ymm1, ymm2, 0x31
        vmovdqu ymmword ptr [rsi+0x80], ymm9
        cmp dl, 0x06
        jb 4f
        vperm2i128 ymm11, ymm3, ymm4, 0x31
        vmovdqu ymmword ptr [rsi+0xA0], ymm11
        cmp dl, 0x07
        jb 4f
        vperm2i128 ymm14, ymm10, ymm14, 0x31
        vmovdqu ymmword ptr [rsi+0xC0], ymm14
        cmp dl, 0x08
        jb 4f
        vperm2i128 ymm15, ymm6, ymm15, 0x31
        vmovdqu ymmword ptr [rsi+0xE0], ymm15
4:
        jmp 9b
3:
        mov rax, qword ptr [rsp+0x100]
        mov rbx, qword ptr [rbp+0x90]
        movzx esi, byte ptr [rbp+0x78]
        movzx edi, byte ptr [rbp+0x88]
        mov r8d, 0xAAAA
        kmovw k2, r8d
        mov r8d, 0x8888
        kmovw k3, r8d
        cmp dl, 0x02
        jbe 3f
        vbroadcasti32x4 zmm0, xmmword ptr [r9]
        vbroadcasti32x4 zmm1, xmmword ptr [r9+0x10]
        vbroadcasti32x4 zmm4, xmmword ptr [rip+BLAKE3_IV]
        mov r8d, 0x4444
        kmovw k4, r8d
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+0x40]
        vpbroadcastd zmm5, dword ptr [rip+BLAKE3_BLOCK_LEN]
        vpunpckldq xmm8, xmm6, xmm7
        vpunpckhdq xmm7, xmm6, xmm7
        vinserti64x4 zmm8, zmm8, ymm7, 0x01
        vpermq zmm8, zmm8, 0xDC
        vpblendmd zmm5 {k4}, zmm8, zmm5
        mov r8, qword ptr [rcx]
        mov r10, qword ptr [rcx+0x08]
        mov r11, qword ptr [rcx+0x10]
        cmp dl, 0x04
        jb 4f
        mov r12, qword ptr [rcx+0x18]
4:
        movzx r13d, byte ptr [rbp+0x80]
        or r13d, esi
        xor r14d, r14d
2:
        movzx r15d, byte ptr [rbp+0x88]
        or r15d, r13d
        add r14, 0x40
        cmp r14, rax
        cmovz r13d, r15d
        mov dword ptr [rsp+0x80], r13d
        vmovdqa32 zmm2, zmm4
        vpblendmd zmm3 {k3}, zmm5, dword ptr [rsp+0x80] {1to16}
        vmovdqu32 zmm10, zmmword ptr [r8+r14*1-0x40]
        vmovdqu32 zmm11, zmmword ptr [r8+r14*1-0x30]
        vmovdqu32 zmm12, zmmword ptr [r8+r14*1-0x20]
        vmovdqu32 zmm13, zmmword ptr [r8+r14*1-0x10]
        vinserti32x4 zmm10, zmm10, xmmword ptr [r10+r14*1-0x40], 0x01
        vinserti32x4 zmm11, zmm11, xmmword ptr [r10+r14*1-0x30], 0x01
        vinserti32x4 zmm12, zmm12, xmmword ptr [r10+r14*1-0x20], 0x01
        vinserti32x4 zmm13, zmm13, xmmword ptr [r10+r14*1-0x10], 0x01
        vinserti32x4 zmm10, zmm10, xmmword ptr [r11+r14*1-0x40], 0x02
        vinserti32x4 zmm11, zmm11, xmmword ptr [r11+r14*1-0x30], 0x02
        vinserti32x4 zmm12, zmm12, xmmword ptr [r11+r14*1-0x20], 0x02
        vinserti32x4 zmm13, zmm13, xmmword ptr [r11+r14*1-0x10], 0x02
        cmp dl, 0x04
        jb 4f
        vinserti32x4 zmm10, zmm10, xmmword ptr [r12+r14*1-0x40], 0x03
        vinserti32x4 zmm11, zmm11, xmmword ptr [r12+r14*1-0x30], 0x03
        vinserti32x4 zmm12, zmm12, xmmword ptr [r12+r14*1-0x20], 0x03
        vinserti32x4 zmm13, zmm13, xmmword ptr [r12+r14*1-0x10], 0x03
4:
        vshufps zmm6, zmm10, zmm11, 0x88
        vshufps zmm7, zmm10, zmm11, 0xDD
        vshufps zmm8, zmm12, zmm13, 0x88
        vshufps zmm9, zmm12, zmm13, 0xDD
        vpshufd zmm8, zmm8, 0x93
        vpshufd zmm9, zmm9, 0x93
        mov r15b, 0x07
4:
        vpaddd zmm0, zmm0, zmm6
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x10
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x0C
        vpaddd zmm0, zmm0, zmm7
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x08
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x07
        vpshufd zmm0, zmm0, 0x93
        vpshufd zmm3, zmm3, 0x4E
        vpshufd zmm2, zmm2, 0x39
        vpaddd zmm0, zmm0, zmm8
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x10
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x0C
        vpaddd zmm0, zmm0, zmm9
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x08
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x07
        vpshufd zmm0, zmm0, 0x39
        vpshufd zmm3, zmm3, 0x4E
        vpshufd zmm2, zmm2, 0x93
        dec r15b
        jz 4f
        vshufps zmm14, zmm6, zmm7, 0xD6
        vpshufd zmm15, zmm6, 0x0F
        vpshufd zmm6, zmm14, 0x39
        vshufps zmm14, zmm8, zmm9, 0xFA
        vpblendmd zmm15 {k2}, zmm15, zmm14
        vpunpcklqdq zmm14, zmm9, zmm7
        vpblendmd zmm14 {k3}, zmm14, zmm8
        vpshufd zmm14, zmm14, 0x78
        vpunpckhdq zmm7, zmm7, zmm9
        vpunpckldq zmm8, zmm8, zmm7
        vpshufd zmm9, zmm8, 0x1E
        vmovdqa32 zmm7, zmm15
        vmovdqa32 zmm8, zmm14
        jmp 4b
4:
        vpxord zmm0, zmm0, zmm2
        vpxord zmm1, zmm1, zmm3
        mov r13d, esi
        cmp r14, rax
        jb 2b
        vmovdqu xmmword ptr [rbx], xmm0
        vmovdqu xmmword ptr [rbx+0x10], xmm1
        vextracti128 xmmword ptr [rbx+0x20], ymm0, 0x01
        vextracti128 xmmword ptr [rbx+0x30], ymm1, 0x01
        vextracti32x4 xmmword ptr [rbx+0x40], zmm0, 0x02
        vextracti32x4 xmmword ptr [rbx+0x50], zmm1, 0x02
        cmp dl, 0x04
        jb 4f
        vextracti32x4 xmmword ptr [rbx+0x60], zmm0, 0x03
        vextracti32x4 xmmword ptr [rbx+0x70], zmm1, 0x03
4:
        jmp 9b
3:
        test dl, dl
        jz 9b
        vbroadcasti128 ymm0, xmmword ptr [r9]
        vbroadcasti128 ymm1, xmmword ptr [r9+0x10]
        vbroadcasti128 ymm4, xmmword ptr [rip+BLAKE3_IV]
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+0x40]
        mov r8d, 0x40
        vpbroadcastq ymm5, r8
        mov r8d, 0x55
        kmovw k4, r8d
        vpunpckldq xmm8, xmm6, xmm7
        vpunpckhdq xmm7, xmm6, xmm7
        vinserti128 ymm8, ymm8, xmm7, 0x01
        vpermq ymm5 {k4}, ymm8, 0xDC
        mov r8, qword ptr [rcx]
        cmp dl, 0x02
        jb 4f
        mov r10, qword ptr [rcx+0x08]
4:
        mov r11d, esi
        movzx r12d, byte ptr [rbp+0x80]
        or r11d, r12d
        xor r12d, r12d
2:
        movzx r13d, byte ptr [rbp+0x88]
        or r13d, r11d
        add r12, 0x40
        cmp r12, rax
        cmovz r11d, r13d
        mov dword ptr [rsp+0x80], r11d
        vmovdqa ymm2, ymm4
        vpblendmd ymm3 {k3}, ymm5, dword ptr [rsp+0x80] {1to8}
        vmovdqu ymm10, ymmword ptr [r8+r12*1-0x40]
        vmovdqu ymm11, ymmword ptr [r8+r12*1-0x30]
        vmovdqu ymm12, ymmword ptr [r8+r12*1-0x20]
        vmovdqu ymm13, ymmword ptr [r8+r12*1-0x10]
        cmp dl, 0x02
        jb 4f
        vinserti128 ymm10, ymm10, xmmword ptr [r10+r12*1-0x40], 0x01
        vinserti128 ymm11, ymm11, xmmword ptr [r10+r12*1-0x30], 0x01
        vinserti128 ymm12, ymm12, xmmword ptr [r10+r12*1-0x20], 0x01
        vinserti128 ymm13, ymm13, xmmword ptr [r10+r12*1-0x10], 0x01
4:
        vshufps ymm6, ymm10, ymm11, 0x88
        vshufps ymm7, ymm10, ymm11, 0xDD
        vshufps ymm8, ymm12, ymm13, 0x88
        vshufps ymm9, ymm12, ymm13, 0xDD
        vpshufd ymm8, ymm8, 0x93
        vpshufd ymm9, ymm9, 0x93
        mov r13b, 0x07
4:
        vpaddd ymm0, ymm0, ymm6
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x10
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x0C
        vpaddd ymm0, ymm0, ymm7
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x08
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x07
        vpshufd ymm0, ymm0, 0x93
        vpshufd ymm3, ymm3, 0x4E
        vpshufd ymm2, ymm2, 0x39
        vpaddd ymm0, ymm0, ymm8
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x10
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x0C
        vpaddd ymm0, ymm0, ymm9
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x08
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x07
        vpshufd ymm0, ymm0, 0x39
        vpshufd ymm3, ymm3, 0x4E
        vpshufd ymm2, ymm2, 0x93
        dec r13b
        jz 4f
        vshufps ymm10, ymm6, ymm7, 0xD6
        vpshufd ymm11, ymm6, 0x0F
        vpshufd ymm6, ymm10, 0x39
        vshufps ymm10, ymm8, ymm9, 0xFA
        vpblendd ymm11, ymm11, ymm10, 0xAA
        vpunpcklqdq ymm10, ymm9, ymm7
        vpblendd ymm10, ymm10, ymm8, 0x88
        vpshufd ymm10, ymm10, 0x78
        vpunpckhdq ymm7, ymm7, ymm9
        vpunpckldq ymm8, ymm8, ymm7
        vpshufd ymm9, ymm8, 0x1E
        vmovdqa ymm7, ymm11
        vmovdqa ymm8, ymm10
        jmp 4b
4:
        vpxor ymm0, ymm0, ymm2
        vpxor ymm1, ymm1, ymm3
        mov r11d, esi
        cmp r12, rax
        jb 2b
        vmovdqu xmmword ptr [rbx], xmm0
        vmovdqu xmmword ptr [rbx+0x10], xmm1
        cmp dl, 0x02
        jb 4f
        vextracti128 xmmword ptr [rbx+0x20], ymm0, 0x01
        vextracti128 xmmword ptr [rbx+0x30], ymm1, 0x01
4:
        jmp 9b

.p2align 6
_blake3_compress_in_place_avx512:
blake3_compress_in_place_avx512:
        sub     rsp, 72
        vmovdqa xmmword ptr [rsp], xmm6
        vmovdqa xmmword ptr [rsp+0x10], xmm7
        vmovdqa xmmword ptr [rsp+0x20], xmm8
        vmovdqa xmmword ptr [rsp+0x30], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+0x10]
        movzx   eax, byte ptr [rsp+0x70]
        movzx   r8d, r8b
        shl     rax, 32
        add     r8, rax
        vmovq   xmm3, r9
        vmovq   xmm4, r8
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]
        vmovups xmm8, xmmword ptr [rdx]
        vmovups xmm9, xmmword ptr [rdx+0x10]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rdx+0x20]
        vmovups xmm9, xmmword ptr [rdx+0x30]
        vshufps xmm6, xmm8, xmm9, 136
        vshufps xmm7, xmm8, xmm9, 221
        vpshufd xmm6, xmm6, 0x93
        vpshufd xmm7, xmm7, 0x93
        mov     al, 7
9:
        vpaddd  xmm0, xmm0, xmm4
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 16
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 12
        vpaddd  xmm0, xmm0, xmm5
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 8
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 7
        vpshufd xmm0, xmm0, 0x93
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x39
        vpaddd  xmm0, xmm0, xmm6
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 16
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 12
        vpaddd  xmm0, xmm0, xmm7
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 8
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 7
        vpshufd xmm0, xmm0, 0x39
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x93
        dec     al
        jz      9f
        vshufps xmm8, xmm4, xmm5, 214
        vpshufd xmm9, xmm4, 0x0F
        vpshufd xmm4, xmm8, 0x39
        vshufps xmm8, xmm6, xmm7, 250
        vpblendd xmm9, xmm9, xmm8, 0xAA
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 0x88
        vpshufd xmm8, xmm8, 0x78
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 0x1E
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp     9b
9:
        vpxor   xmm0, xmm0, xmm2
        vpxor   xmm1, xmm1, xmm3
        vmovdqu xmmword ptr [rcx], xmm0
        vmovdqu xmmword ptr [rcx+0x10], xmm1
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+0x10]
        vmovdqa xmm8, xmmword ptr [rsp+0x20]
        vmovdqa xmm9, xmmword ptr [rsp+0x30]
        add     rsp, 72
        ret


.p2align 6
_blake3_compress_xof_avx512:
blake3_compress_xof_avx512:
        sub     rsp, 72
        vmovdqa xmmword ptr [rsp], xmm6
        vmovdqa xmmword ptr [rsp+0x10], xmm7
        vmovdqa xmmword ptr [rsp+0x20], xmm8
        vmovdqa xmmword ptr [rsp+0x30], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+0x10]
        movzx   eax, byte ptr [rsp+0x70]
        movzx   r8d, r8b
        mov     r10, qword ptr [rsp+0x78]
        shl     rax, 32
        add     r8, rax
        vmovq   xmm3, r9
        vmovq   xmm4, r8
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]
        vmovups xmm8, xmmword ptr [rdx]
        vmovups xmm9, xmmword ptr [rdx+0x10]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rdx+0x20]
        vmovups xmm9, xmmword ptr [rdx+0x30]
        vshufps xmm6, xmm8, xmm9, 136
        vshufps xmm7, xmm8, xmm9, 221
        vpshufd xmm6, xmm6, 0x93
        vpshufd xmm7, xmm7, 0x93
        mov     al, 7
9:
        vpaddd  xmm0, xmm0, xmm4
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 16
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 12
        vpaddd  xmm0, xmm0, xmm5
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 8
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 7
        vpshufd xmm0, xmm0, 0x93
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x39
        vpaddd  xmm0, xmm0, xmm6
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 16
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 12
        vpaddd  xmm0, xmm0, xmm7
        vpaddd  xmm0, xmm0, xmm1
        vpxord  xmm3, xmm3, xmm0
        vprord  xmm3, xmm3, 8
        vpaddd  xmm2, xmm2, xmm3
        vpxord  xmm1, xmm1, xmm2
        vprord  xmm1, xmm1, 7
        vpshufd xmm0, xmm0, 0x39
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x93
        dec     al
        jz      9f
        vshufps xmm8, xmm4, xmm5, 214
        vpshufd xmm9, xmm4, 0x0F
        vpshufd xmm4, xmm8, 0x39
        vshufps xmm8, xmm6, xmm7, 250
        vpblendd xmm9, xmm9, xmm8, 0xAA
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 0x88
        vpshufd xmm8, xmm8, 0x78
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 0x1E
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp     9b
9:
        vpxor   xmm0, xmm0, xmm2
        vpxor   xmm1, xmm1, xmm3
        vpxor   xmm2, xmm2, xmmword ptr [rcx]
        vpxor   xmm3, xmm3, xmmword ptr [rcx+0x10]
        vmovdqu xmmword ptr [r10], xmm0
        vmovdqu xmmword ptr [r10+0x10], xmm1
        vmovdqu xmmword ptr [r10+0x20], xmm2
        vmovdqu xmmword ptr [r10+0x30], xmm3
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+0x10]
        vmovdqa xmm8, xmmword ptr [rsp+0x20]
        vmovdqa xmm9, xmmword ptr [rsp+0x30]
        add     rsp, 72
        ret

.p2align 6
blake3_xof_many_avx512:
_blake3_xof_many_avx512:
        mov rax, qword ptr [rsp+0x38]
        cmp rax, 0x01
        jnbe 2f
        sub rsp, 0x48
        movaps xmmword ptr [rsp], xmm6
        movaps xmmword ptr [rsp+0x10], xmm7
        movaps xmmword ptr [rsp+0x20], xmm8
        movaps xmmword ptr [rsp+0x30], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+0x10]
        movzx r8d, r8b
        movzx r10d, byte ptr [rsp+0x70]
        shl r10, 0x20
        or r8, r10
        vmovq xmm2, r8
        vmovq xmm3, r9
        vpunpcklqdq xmm3, xmm3, xmm2
        vmovaps xmm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqu xmm8, xmmword ptr [rdx]
        vmovdqu xmm9, xmmword ptr [rdx+0x10]
        vshufps xmm4, xmm8, xmm9, 0x88
        vshufps xmm5, xmm8, xmm9, 0xDD
        vmovdqu xmm8, xmmword ptr [rdx+0x20]
        vmovdqu xmm9, xmmword ptr [rdx+0x30]
        vshufps xmm6, xmm8, xmm9, 0x88
        vshufps xmm7, xmm8, xmm9, 0xDD
        vpshufd xmm6, xmm6, 0x93
        vpshufd xmm7, xmm7, 0x93
        mov r8b, 0x07
3:
        vpaddd xmm0, xmm0, xmm4
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x10
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x0C
        vpaddd xmm0, xmm0, xmm5
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x08
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x07
        vpshufd xmm0, xmm0, 0x93
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x39
        vpaddd xmm0, xmm0, xmm6
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x10
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x0C
        vpaddd xmm0, xmm0, xmm7
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x08
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x07
        vpshufd xmm0, xmm0, 0x39
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x93
        dec r8b
        jz 3f
        vshufps xmm8, xmm4, xmm5, 0xD6
        vpshufd xmm9, xmm4, 0x0F
        vpshufd xmm4, xmm8, 0x39
        vshufps xmm8, xmm6, xmm7, 0xFA
        vpblendd xmm9, xmm9, xmm8, 0xAA
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 0x88
        vpshufd xmm8, xmm8, 0x78
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 0x1E
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp 3b
3:
        mov r8, qword ptr [rsp+0x78]
        vpxor xmm0, xmm0, xmm2
        vpxor xmm1, xmm1, xmm3
        vpxor xmm2, xmm2, xmmword ptr [rcx]
        vpxor xmm3, xmm3, xmmword ptr [rcx+0x10]
        vmovdqu xmmword ptr [r8], xmm0
        vmovdqu xmmword ptr [r8+0x10], xmm1
        vmovdqu xmmword ptr [r8+0x20], xmm2
        vmovdqu xmmword ptr [r8+0x30], xmm3
        vzeroupper
        movaps xmm6, xmmword ptr [rsp]
        movaps xmm7, xmmword ptr [rsp+0x10]
        movaps xmm8, xmmword ptr [rsp+0x20]
        movaps xmm9, xmmword ptr [rsp+0x30]
        add rsp, 0x48
        ret
2:
        push rbp
        mov rbp, rsp
        sub rsp, 0x1A0
        movaps xmmword ptr [rbp-0xA0], xmm6
        movaps xmmword ptr [rbp-0x90], xmm7
        movaps xmmword ptr [rbp-0x80], xmm8
        movaps xmmword ptr [rbp-0x70], xmm9
        movaps xmmword ptr [rbp-0x60], xmm10
        movaps xmmword ptr [rbp-0x50], xmm11
        movaps xmmword ptr [rbp-0x40], xmm12
        movaps xmmword ptr [rbp-0x30], xmm13
        movaps xmmword ptr [rbp-0x20], xmm14
        movaps xmmword ptr [rbp-0x10], xmm15
        and rsp, 0xFFFFFFFFFFFFFFC0
        vpbroadcastd zmm0, r9d
        shr r9, 0x20
        vpbroadcastd zmm1, r9d
        vpaddd zmm2, zmm0, zmmword ptr [rip+ADD0]
        vpcmpud k1, zmm2, zmm0, 0x01
        vpaddd zmm1 {k1}, zmm1, dword ptr [rip+ADD1] {1to16}
        vmovdqa32 zmmword ptr [rsp], zmm2
        vmovdqa32 zmmword ptr [rsp+0x40], zmm1
        mov r9, qword ptr [rbp+0x38]
        movzx r8d, r8b
        movzx r10d, byte ptr [rbp+0x30]
        cmp rax, 0x08
        jbe 2f
.p2align 5
3:
        vpbroadcastd zmm0, dword ptr [rdx]
        vpbroadcastd zmm1, dword ptr [rdx+0x04]
        vpbroadcastd zmm2, dword ptr [rdx+0x08]
        vpbroadcastd zmm3, dword ptr [rdx+0x0C]
        vpbroadcastd zmm4, dword ptr [rdx+0x10]
        vpbroadcastd zmm5, dword ptr [rdx+0x14]
        vpbroadcastd zmm6, dword ptr [rdx+0x18]
        vpbroadcastd zmm7, dword ptr [rdx+0x1C]
        vpbroadcastd zmm8, dword ptr [rdx+0x20]
        vpbroadcastd zmm9, dword ptr [rdx+0x24]
        vpbroadcastd zmm10, dword ptr [rdx+0x28]
        vpbroadcastd zmm11, dword ptr [rdx+0x2C]
        vpbroadcastd zmm12, dword ptr [rdx+0x30]
        vpbroadcastd zmm13, dword ptr [rdx+0x34]
        vpbroadcastd zmm14, dword ptr [rdx+0x38]
        vpbroadcastd zmm15, dword ptr [rdx+0x3C]
        vpbroadcastd zmm16, dword ptr [rcx]
        vpbroadcastd zmm17, dword ptr [rcx+0x04]
        vpbroadcastd zmm18, dword ptr [rcx+0x08]
        vpbroadcastd zmm19, dword ptr [rcx+0x0C]
        vpbroadcastd zmm20, dword ptr [rcx+0x10]
        vpbroadcastd zmm21, dword ptr [rcx+0x14]
        vpbroadcastd zmm22, dword ptr [rcx+0x18]
        vpbroadcastd zmm23, dword ptr [rcx+0x1C]
        vpbroadcastd zmm24, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd zmm25, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd zmm26, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd zmm27, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa32 zmm28, zmmword ptr [rsp]
        vmovdqa32 zmm29, zmmword ptr [rsp+0x40]
        vpbroadcastd zmm30, r8d
        vpbroadcastd zmm31, r10d
        mov r11b, 0x07
4:
        vpaddd zmm16, zmm16, zmm0
        vpaddd zmm17, zmm17, zmm2
        vpaddd zmm18, zmm18, zmm4
        vpaddd zmm19, zmm19, zmm6
        vmovdqa32 zmmword ptr [rsp+0x80], zmm0
        vmovdqa32 zmmword ptr [rsp+0xC0], zmm6
        vpaddd zmm16, zmm16, zmm20
        vpaddd zmm17, zmm17, zmm21
        vpaddd zmm18, zmm18, zmm22
        vpaddd zmm19, zmm19, zmm23
        vpxord zmm28, zmm28, zmm16
        vmovdqa32 zmm0, zmm2
        vpxord zmm29, zmm29, zmm17
        vpxord zmm30, zmm30, zmm18
        vpxord zmm31, zmm31, zmm19
        vprord zmm28, zmm28, 0x10
        vprord zmm29, zmm29, 0x10
        vprord zmm30, zmm30, 0x10
        vprord zmm31, zmm31, 0x10
        vpaddd zmm24, zmm24, zmm28
        vmovdqa32 zmm6, zmm4
        vpaddd zmm25, zmm25, zmm29
        vpaddd zmm26, zmm26, zmm30
        vpaddd zmm27, zmm27, zmm31
        vpxord zmm20, zmm20, zmm24
        vpxord zmm21, zmm21, zmm25
        vpxord zmm22, zmm22, zmm26
        vpxord zmm23, zmm23, zmm27
        vprord zmm20, zmm20, 0x0C
        vprord zmm21, zmm21, 0x0C
        vprord zmm22, zmm22, 0x0C
        vprord zmm23, zmm23, 0x0C
        vpaddd zmm16, zmm16, zmm1
        vpaddd zmm17, zmm17, zmm3
        vpaddd zmm18, zmm18, zmm5
        vpaddd zmm19, zmm19, zmm7
        vpaddd zmm16, zmm16, zmm20
        vmovdqa32 zmm2, zmm3
        vpaddd zmm17, zmm17, zmm21
        vpaddd zmm18, zmm18, zmm22
        vpaddd zmm19, zmm19, zmm23
        vpxord zmm28, zmm28, zmm16
        vpxord zmm29, zmm29, zmm17
        vpxord zmm30, zmm30, zmm18
        vpxord zmm31, zmm31, zmm19
        vprord zmm28, zmm28, 0x08
        vmovdqa32 zmm4, zmm7
        vprord zmm29, zmm29, 0x08
        vprord zmm30, zmm30, 0x08
        vprord zmm31, zmm31, 0x08
        vpaddd zmm24, zmm24, zmm28
        vpaddd zmm25, zmm25, zmm29
        vpaddd zmm26, zmm26, zmm30
        vpaddd zmm27, zmm27, zmm31
        vpxord zmm20, zmm20, zmm24
        vmovdqa32 zmm3, zmm10
        vpxord zmm21, zmm21, zmm25
        vpxord zmm22, zmm22, zmm26
        vpxord zmm23, zmm23, zmm27
        vprord zmm20, zmm20, 0x07
        vprord zmm21, zmm21, 0x07
        vprord zmm22, zmm22, 0x07
        vprord zmm23, zmm23, 0x07
        vpaddd zmm16, zmm16, zmm8
        vmovdqa32 zmm7, zmm13
        vpaddd zmm17, zmm17, zmm10
        vpaddd zmm18, zmm18, zmm12
        vpaddd zmm19, zmm19, zmm14
        vpaddd zmm16, zmm16, zmm21
        vpaddd zmm17, zmm17, zmm22
        vpaddd zmm18, zmm18, zmm23
        vpaddd zmm19, zmm19, zmm20
        vpxord zmm31, zmm31, zmm16
        vmovdqa32 zmm10, zmm12
        vpxord zmm28, zmm28, zmm17
        vpxord zmm29, zmm29, zmm18
        vpxord zmm30, zmm30, zmm19
        vprord zmm31, zmm31, 0x10
        vprord zmm28, zmm28, 0x10
        vprord zmm29, zmm29, 0x10
        vprord zmm30, zmm30, 0x10
        vpaddd zmm26, zmm26, zmm31
        vmovdqa32 zmm12, zmm9
        vpaddd zmm27, zmm27, zmm28
        vpaddd zmm24, zmm24, zmm29
        vpaddd zmm25, zmm25, zmm30
        vpxord zmm21, zmm21, zmm26
        vpxord zmm22, zmm22, zmm27
        vpxord zmm23, zmm23, zmm24
        vpxord zmm20, zmm20, zmm25
        vprord zmm21, zmm21, 0x0C
        vprord zmm22, zmm22, 0x0C
        vprord zmm23, zmm23, 0x0C
        vprord zmm20, zmm20, 0x0C
        vpaddd zmm16, zmm16, zmm9
        vpaddd zmm17, zmm17, zmm11
        vpaddd zmm18, zmm18, zmm13
        vmovdqa32 zmm13, zmm14
        vpaddd zmm19, zmm19, zmm15
        vpaddd zmm16, zmm16, zmm21
        vpaddd zmm17, zmm17, zmm22
        vpaddd zmm18, zmm18, zmm23
        vmovdqa32 zmm9, zmm11
        vpaddd zmm19, zmm19, zmm20
        vpxord zmm31, zmm31, zmm16
        vpxord zmm28, zmm28, zmm17
        vmovdqa32 zmm14, zmm15
        vpxord zmm29, zmm29, zmm18
        vpxord zmm30, zmm30, zmm19
        vprord zmm31, zmm31, 0x08
        vmovdqa32 zmm11, zmm5
        vprord zmm28, zmm28, 0x08
        vprord zmm29, zmm29, 0x08
        vprord zmm30, zmm30, 0x08
        vmovdqa32 zmm15, zmm8
        vpaddd zmm26, zmm26, zmm31
        vpaddd zmm27, zmm27, zmm28
        vpaddd zmm24, zmm24, zmm29
        vpaddd zmm25, zmm25, zmm30
        vmovdqa32 zmm8, zmm1
        vpxord zmm21, zmm21, zmm26
        vpxord zmm22, zmm22, zmm27
        vpxord zmm23, zmm23, zmm24
        vpxord zmm20, zmm20, zmm25
        vmovdqa32 zmm5, zmmword ptr [rsp+0x80]
        vprord zmm21, zmm21, 0x07
        vprord zmm22, zmm22, 0x07
        vprord zmm23, zmm23, 0x07
        vprord zmm20, zmm20, 0x07
        vmovdqa32 zmm1, zmmword ptr [rsp+0xC0]
        dec r11b
        jnz 4b
        vpxord zmm16, zmm16, zmm24
        vpxord zmm17, zmm17, zmm25
        vpxord zmm18, zmm18, zmm26
        vpxord zmm19, zmm19, zmm27
        vpxord zmm20, zmm20, zmm28
        vpxord zmm21, zmm21, zmm29
        vpxord zmm22, zmm22, zmm30
        vpxord zmm23, zmm23, zmm31
        vpunpckldq zmm0, zmm16, zmm18
        vpunpckhdq zmm1, zmm16, zmm18
        vpunpckldq zmm2, zmm17, zmm19
        vpunpckhdq zmm3, zmm17, zmm19
        vpunpckldq zmm4, zmm20, zmm22
        vpunpckhdq zmm5, zmm20, zmm22
        vpunpckldq zmm6, zmm21, zmm23
        vpunpckhdq zmm7, zmm21, zmm23
        vpunpckldq zmm16, zmm0, zmm2
        vpunpckhdq zmm17, zmm0, zmm2
        vpunpckldq zmm18, zmm1, zmm3
        vpunpckhdq zmm19, zmm1, zmm3
        vpunpckldq zmm20, zmm4, zmm6
        vpunpckhdq zmm21, zmm4, zmm6
        vpunpckldq zmm22, zmm5, zmm7
        vpunpckhdq zmm23, zmm5, zmm7
        vpunpckldq zmm0, zmm24, zmm26
        vpunpckhdq zmm1, zmm24, zmm26
        vpunpckldq zmm2, zmm25, zmm27
        vpunpckhdq zmm3, zmm25, zmm27
        vpunpckldq zmm4, zmm28, zmm30
        vpunpckhdq zmm5, zmm28, zmm30
        vpunpckldq zmm6, zmm29, zmm31
        vpunpckhdq zmm7, zmm29, zmm31
        vpunpckldq zmm24, zmm0, zmm2
        vpunpckhdq zmm25, zmm0, zmm2
        vpunpckldq zmm26, zmm1, zmm3
        vpunpckhdq zmm27, zmm1, zmm3
        vpunpckldq zmm28, zmm4, zmm6
        vpunpckhdq zmm29, zmm4, zmm6
        vpunpckldq zmm30, zmm5, zmm7
        vpunpckhdq zmm31, zmm5, zmm7
        vmovdqa32 zmm8, zmmword ptr [rip+INDEX0]
        vmovdqa32 zmm9, zmmword ptr [rip+INDEX1]
        vmovdqa32 zmm0, zmm16
        vpermt2d zmm0, zmm8, zmm20
        vpermt2d zmm16, zmm9, zmm20
        vmovdqa32 zmm1, zmm24
        vpermt2d zmm1, zmm8, zmm28
        vpermt2d zmm24, zmm9, zmm28
        vmovdqa32 zmm2, zmm17
        vpermt2d zmm2, zmm8, zmm21
        vpermt2d zmm17, zmm9, zmm21
        vmovdqa32 zmm3, zmm25
        vpermt2d zmm3, zmm8, zmm29
        vpermt2d zmm25, zmm9, zmm29
        vmovdqa32 zmm4, zmm18
        vpermt2d zmm4, zmm8, zmm22
        vpermt2d zmm18, zmm9, zmm22
        vmovdqa32 zmm5, zmm26
        vpermt2d zmm5, zmm8, zmm30
        vpermt2d zmm26, zmm9, zmm30
        vmovdqa32 zmm6, zmm19
        vpermt2d zmm6, zmm8, zmm23
        vpermt2d zmm19, zmm9, zmm23
        vmovdqa32 zmm7, zmm27
        vpermt2d zmm7, zmm8, zmm31
        vpermt2d zmm27, zmm9, zmm31
        vbroadcasti64x4 zmm8, ymmword ptr [rcx]
        vpxord zmm1, zmm1, zmm8
        vpxord zmm3, zmm3, zmm8
        vpxord zmm5, zmm5, zmm8
        vpxord zmm7, zmm7, zmm8
        vpxord zmm24, zmm24, zmm8
        vpxord zmm25, zmm25, zmm8
        vpxord zmm26, zmm26, zmm8
        vpxord zmm27, zmm27, zmm8
        vextracti64x4 ymmword ptr [r9], zmm0, 0x00
        vextracti64x4 ymmword ptr [r9+0x20], zmm1, 0x00
        vextracti64x4 ymmword ptr [r9+0x40], zmm2, 0x00
        vextracti64x4 ymmword ptr [r9+0x60], zmm3, 0x00
        vextracti64x4 ymmword ptr [r9+0x80], zmm4, 0x00
        vextracti64x4 ymmword ptr [r9+0xA0], zmm5, 0x00
        vextracti64x4 ymmword ptr [r9+0xC0], zmm6, 0x00
        vextracti64x4 ymmword ptr [r9+0xE0], zmm7, 0x00
        vextracti64x4 ymmword ptr [r9+0x100], zmm16, 0x00
        vextracti64x4 ymmword ptr [r9+0x120], zmm24, 0x00
        vextracti64x4 ymmword ptr [r9+0x140], zmm17, 0x00
        vextracti64x4 ymmword ptr [r9+0x160], zmm25, 0x00
        vextracti64x4 ymmword ptr [r9+0x180], zmm18, 0x00
        vextracti64x4 ymmword ptr [r9+0x1A0], zmm26, 0x00
        vextracti64x4 ymmword ptr [r9+0x1C0], zmm19, 0x00
        vextracti64x4 ymmword ptr [r9+0x1E0], zmm27, 0x00
        vextracti64x4 ymmword ptr [r9+0x200], zmm0, 0x01
        vextracti64x4 ymmword ptr [r9+0x220], zmm1, 0x01
        cmp rax, 0x0A
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x240], zmm2, 0x01
        vextracti64x4 ymmword ptr [r9+0x260], zmm3, 0x01
        cmp rax, 0x0B
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x280], zmm4, 0x01
        vextracti64x4 ymmword ptr [r9+0x2A0], zmm5, 0x01
        cmp rax, 0x0C
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x2C0], zmm6, 0x01
        vextracti64x4 ymmword ptr [r9+0x2E0], zmm7, 0x01
        cmp rax, 0x0D
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x300], zmm16, 0x01
        vextracti64x4 ymmword ptr [r9+0x320], zmm24, 0x01
        cmp rax, 0x0E
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x340], zmm17, 0x01
        vextracti64x4 ymmword ptr [r9+0x360], zmm25, 0x01
        cmp rax, 0x0F
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x380], zmm18, 0x01
        vextracti64x4 ymmword ptr [r9+0x3A0], zmm26, 0x01
        cmp rax, 0x10
        jb 9f
        vextracti64x4 ymmword ptr [r9+0x3C0], zmm19, 0x01
        vextracti64x4 ymmword ptr [r9+0x3E0], zmm27, 0x01
        vmovdqa32 zmm0, zmmword ptr [rsp]
        vmovdqa32 zmm1, zmmword ptr [rsp+0x40]
        vpaddd zmm2, zmm0, dword ptr [rip+ADD16] {1to16}
        vpcmpud k1, zmm2, zmm0, 0x01
        vpaddd zmm1 {k1}, zmm1, dword ptr [rip+ADD1] {1to16}
        vmovdqa32 zmmword ptr [rsp], zmm2
        vmovdqa32 zmmword ptr [rsp+0x40], zmm1
        add r9, 0x400
        sub rax, 0x10
        cmp rax, 0x08
        jnbe 3b
        test al, al
        jnz 2f
9:
        vzeroupper
        movaps xmm6, xmmword ptr [rbp-0xA0]
        movaps xmm7, xmmword ptr [rbp-0x90]
        movaps xmm8, xmmword ptr [rbp-0x80]
        movaps xmm9, xmmword ptr [rbp-0x70]
        movaps xmm10, xmmword ptr [rbp-0x60]
        movaps xmm11, xmmword ptr [rbp-0x50]
        movaps xmm12, xmmword ptr [rbp-0x40]
        movaps xmm13, xmmword ptr [rbp-0x30]
        movaps xmm14, xmmword ptr [rbp-0x20]
        movaps xmm15, xmmword ptr [rbp-0x10]
        mov rsp, rbp
        pop rbp
        ret
2:
        cmp al, 0x04
        jbe 2f
        vpbroadcastd ymm16, dword ptr [rdx]
        vpbroadcastd ymm17, dword ptr [rdx+0x04]
        vpbroadcastd ymm18, dword ptr [rdx+0x08]
        vpbroadcastd ymm19, dword ptr [rdx+0x0C]
        vpbroadcastd ymm20, dword ptr [rdx+0x10]
        vpbroadcastd ymm21, dword ptr [rdx+0x14]
        vpbroadcastd ymm22, dword ptr [rdx+0x18]
        vpbroadcastd ymm23, dword ptr [rdx+0x1C]
        vpbroadcastd ymm24, dword ptr [rdx+0x20]
        vpbroadcastd ymm25, dword ptr [rdx+0x24]
        vpbroadcastd ymm26, dword ptr [rdx+0x28]
        vpbroadcastd ymm27, dword ptr [rdx+0x2C]
        vpbroadcastd ymm28, dword ptr [rdx+0x30]
        vpbroadcastd ymm29, dword ptr [rdx+0x34]
        vpbroadcastd ymm30, dword ptr [rdx+0x38]
        vpbroadcastd ymm31, dword ptr [rdx+0x3C]
        vpbroadcastd ymm0, dword ptr [rcx]
        vpbroadcastd ymm1, dword ptr [rcx+0x04]
        vpbroadcastd ymm2, dword ptr [rcx+0x08]
        vpbroadcastd ymm3, dword ptr [rcx+0x0C]
        vpbroadcastd ymm4, dword ptr [rcx+0x10]
        vpbroadcastd ymm5, dword ptr [rcx+0x14]
        vpbroadcastd ymm6, dword ptr [rcx+0x18]
        vpbroadcastd ymm7, dword ptr [rcx+0x1C]
        vpbroadcastd ymm8, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd ymm9, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd ymm10, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd ymm11, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa ymm12, ymmword ptr [rsp]
        vmovdqa ymm13, ymmword ptr [rsp+0x40]
        vpbroadcastd ymm14, r8d
        vpbroadcastd ymm15, r10d
        mov r11b, 0x07
3:
        vpaddd ymm0, ymm0, ymm16
        vpaddd ymm1, ymm1, ymm18
        vpaddd ymm2, ymm2, ymm20
        vpaddd ymm3, ymm3, ymm22
        vmovdqa32 ymmword ptr [rsp+0x80], ymm16
        vmovdqa32 ymmword ptr [rsp+0xC0], ymm22
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm12, ymm12, ymm0
        vmovdqa32 ymm16, ymm18
        vpxord ymm13, ymm13, ymm1
        vpxord ymm14, ymm14, ymm2
        vpxord ymm15, ymm15, ymm3
        vprord ymm12, ymm12, 0x10
        vprord ymm13, ymm13, 0x10
        vprord ymm14, ymm14, 0x10
        vprord ymm15, ymm15, 0x10
        vpaddd ymm8, ymm8, ymm12
        vmovdqa32 ymm22, ymm20
        vpaddd ymm9, ymm9, ymm13
        vpaddd ymm10, ymm10, ymm14
        vpaddd ymm11, ymm11, ymm15
        vpxord ymm4, ymm4, ymm8
        vpxord ymm5, ymm5, ymm9
        vpxord ymm6, ymm6, ymm10
        vpxord ymm7, ymm7, ymm11
        vprord ymm4, ymm4, 0x0C
        vprord ymm5, ymm5, 0x0C
        vprord ymm6, ymm6, 0x0C
        vprord ymm7, ymm7, 0x0C
        vpaddd ymm0, ymm0, ymm17
        vpaddd ymm1, ymm1, ymm19
        vpaddd ymm2, ymm2, ymm21
        vpaddd ymm3, ymm3, ymm23
        vpaddd ymm0, ymm0, ymm4
        vmovdqa32 ymm18, ymm19
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm12, ymm12, ymm0
        vpxord ymm13, ymm13, ymm1
        vpxord ymm14, ymm14, ymm2
        vpxord ymm15, ymm15, ymm3
        vprord ymm12, ymm12, 0x08
        vmovdqa32 ymm20, ymm23
        vprord ymm13, ymm13, 0x08
        vprord ymm14, ymm14, 0x08
        vprord ymm15, ymm15, 0x08
        vpaddd ymm8, ymm8, ymm12
        vpaddd ymm9, ymm9, ymm13
        vpaddd ymm10, ymm10, ymm14
        vpaddd ymm11, ymm11, ymm15
        vpxord ymm4, ymm4, ymm8
        vmovdqa32 ymm19, ymm26
        vpxord ymm5, ymm5, ymm9
        vpxord ymm6, ymm6, ymm10
        vpxord ymm7, ymm7, ymm11
        vprord ymm4, ymm4, 0x07
        vprord ymm5, ymm5, 0x07
        vprord ymm6, ymm6, 0x07
        vprord ymm7, ymm7, 0x07
        vpaddd ymm0, ymm0, ymm24
        vmovdqa32 ymm23, ymm29
        vpaddd ymm1, ymm1, ymm26
        vpaddd ymm2, ymm2, ymm28
        vpaddd ymm3, ymm3, ymm30
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm1, ymm1, ymm6
        vpaddd ymm2, ymm2, ymm7
        vpaddd ymm3, ymm3, ymm4
        vpxord ymm15, ymm15, ymm0
        vmovdqa32 ymm26, ymm28
        vpxord ymm12, ymm12, ymm1
        vpxord ymm13, ymm13, ymm2
        vpxord ymm14, ymm14, ymm3
        vprord ymm15, ymm15, 0x10
        vprord ymm12, ymm12, 0x10
        vprord ymm13, ymm13, 0x10
        vprord ymm14, ymm14, 0x10
        vpaddd ymm10, ymm10, ymm15
        vmovdqa32 ymm28, ymm25
        vpaddd ymm11, ymm11, ymm12
        vpaddd ymm8, ymm8, ymm13
        vpaddd ymm9, ymm9, ymm14
        vpxord ymm5, ymm5, ymm10
        vpxord ymm6, ymm6, ymm11
        vpxord ymm7, ymm7, ymm8
        vpxord ymm4, ymm4, ymm9
        vprord ymm5, ymm5, 0x0C
        vprord ymm6, ymm6, 0x0C
        vprord ymm7, ymm7, 0x0C
        vprord ymm4, ymm4, 0x0C
        vpaddd ymm0, ymm0, ymm25
        vpaddd ymm1, ymm1, ymm27
        vpaddd ymm2, ymm2, ymm29
        vmovdqa32 ymm29, ymm30
        vpaddd ymm3, ymm3, ymm31
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm1, ymm1, ymm6
        vpaddd ymm2, ymm2, ymm7
        vmovdqa32 ymm25, ymm27
        vpaddd ymm3, ymm3, ymm4
        vpxord ymm15, ymm15, ymm0
        vpxord ymm12, ymm12, ymm1
        vmovdqa32 ymm30, ymm31
        vpxord ymm13, ymm13, ymm2
        vpxord ymm14, ymm14, ymm3
        vprord ymm15, ymm15, 0x08
        vmovdqa32 ymm27, ymm21
        vprord ymm12, ymm12, 0x08
        vprord ymm13, ymm13, 0x08
        vprord ymm14, ymm14, 0x08
        vmovdqa32 ymm31, ymm24
        vpaddd ymm10, ymm10, ymm15
        vpaddd ymm11, ymm11, ymm12
        vpaddd ymm8, ymm8, ymm13
        vpaddd ymm9, ymm9, ymm14
        vmovdqa32 ymm24, ymm17
        vpxord ymm5, ymm5, ymm10
        vpxord ymm6, ymm6, ymm11
        vpxord ymm7, ymm7, ymm8
        vpxord ymm4, ymm4, ymm9
        vmovdqa32 ymm21, ymmword ptr [rsp+0x80]
        vprord ymm5, ymm5, 0x07
        vprord ymm6, ymm6, 0x07
        vprord ymm7, ymm7, 0x07
        vprord ymm4, ymm4, 0x07
        vmovdqa32 ymm17, ymmword ptr [rsp+0xC0]
        dec r11b
        jnz 3b
        vpxord ymm0, ymm0, ymm8
        vpxord ymm8, ymm8, dword ptr [rcx] {1to8}
        vpxord ymm1, ymm1, ymm9
        vpxord ymm9, ymm9, dword ptr [rcx+0x04] {1to8}
        vpxord ymm2, ymm2, ymm10
        vpxord ymm10, ymm10, dword ptr [rcx+0x08] {1to8}
        vpxord ymm3, ymm3, ymm11
        vpxord ymm11, ymm11, dword ptr [rcx+0x0C] {1to8}
        vpxord ymm4, ymm4, ymm12
        vpxord ymm12, ymm12, dword ptr [rcx+0x10] {1to8}
        vpxord ymm5, ymm5, ymm13
        vpxord ymm13, ymm13, dword ptr [rcx+0x14] {1to8}
        vpxord ymm6, ymm6, ymm14
        vpxord ymm14, ymm14, dword ptr [rcx+0x18] {1to8}
        vpxord ymm7, ymm7, ymm15
        vpxord ymm15, ymm15, dword ptr [rcx+0x1C] {1to8}
        vpunpckldq ymm16, ymm0, ymm1
        vpunpckhdq ymm17, ymm0, ymm1
        vpunpckldq ymm18, ymm2, ymm3
        vpunpckhdq ymm19, ymm2, ymm3
        vpunpckldq ymm20, ymm4, ymm5
        vpunpckhdq ymm21, ymm4, ymm5
        vpunpckldq ymm22, ymm6, ymm7
        vpunpckhdq ymm23, ymm6, ymm7
        vpunpckldq ymm24, ymm8, ymm9
        vpunpckhdq ymm25, ymm8, ymm9
        vpunpckldq ymm26, ymm10, ymm11
        vpunpckhdq ymm27, ymm10, ymm11
        vpunpckldq ymm28, ymm12, ymm13
        vpunpckhdq ymm29, ymm12, ymm13
        vpunpckldq ymm30, ymm14, ymm15
        vpunpckhdq ymm31, ymm14, ymm15
        vpunpcklqdq ymm0, ymm16, ymm18
        vpunpckhqdq ymm1, ymm16, ymm18
        vpunpcklqdq ymm2, ymm17, ymm19
        vpunpckhqdq ymm3, ymm17, ymm19
        vpunpcklqdq ymm4, ymm20, ymm22
        vpunpckhqdq ymm5, ymm20, ymm22
        vpunpcklqdq ymm6, ymm21, ymm23
        vpunpckhqdq ymm7, ymm21, ymm23
        vpunpcklqdq ymm8, ymm24, ymm26
        vpunpckhqdq ymm9, ymm24, ymm26
        vpunpcklqdq ymm10, ymm25, ymm27
        vpunpckhqdq ymm11, ymm25, ymm27
        vpunpcklqdq ymm12, ymm28, ymm30
        vpunpckhqdq ymm13, ymm28, ymm30
        vpunpcklqdq ymm14, ymm29, ymm31
        vpunpckhqdq ymm15, ymm29, ymm31
        vshufi32x4 ymm16, ymm0, ymm4, 0x00
        vshufi32x4 ymm17, ymm8, ymm12, 0x00
        vshufi32x4 ymm18, ymm1, ymm5, 0x00
        vshufi32x4 ymm19, ymm9, ymm13, 0x00
        vshufi32x4 ymm20, ymm2, ymm6, 0x00
        vshufi32x4 ymm21, ymm10, ymm14, 0x00
        vshufi32x4 ymm22, ymm3, ymm7, 0x00
        vshufi32x4 ymm23, ymm11, ymm15, 0x00
        vshufi32x4 ymm24, ymm0, ymm4, 0x03
        vshufi32x4 ymm25, ymm8, ymm12, 0x03
        vshufi32x4 ymm26, ymm1, ymm5, 0x03
        vshufi32x4 ymm27, ymm9, ymm13, 0x03
        vshufi32x4 ymm28, ymm2, ymm6, 0x03
        vshufi32x4 ymm29, ymm10, ymm14, 0x03
        vshufi32x4 ymm30, ymm3, ymm7, 0x03
        vshufi32x4 ymm31, ymm11, ymm15, 0x03
        vmovdqu32 ymmword ptr [r9], ymm16
        vmovdqu32 ymmword ptr [r9+0x20], ymm17
        vmovdqu32 ymmword ptr [r9+0x40], ymm18
        vmovdqu32 ymmword ptr [r9+0x60], ymm19
        vmovdqu32 ymmword ptr [r9+0x80], ymm20
        vmovdqu32 ymmword ptr [r9+0xA0], ymm21
        vmovdqu32 ymmword ptr [r9+0xC0], ymm22
        vmovdqu32 ymmword ptr [r9+0xE0], ymm23
        vmovdqu32 ymmword ptr [r9+0x100], ymm24
        vmovdqu32 ymmword ptr [r9+0x120], ymm25
        cmp al, 0x06
        jb 3f
        vmovdqu32 ymmword ptr [r9+0x140], ymm26
        vmovdqu32 ymmword ptr [r9+0x160], ymm27
        cmp al, 0x07
        jb 3f
        vmovdqu32 ymmword ptr [r9+0x180], ymm28
        vmovdqu32 ymmword ptr [r9+0x1A0], ymm29
        cmp al, 0x08
        jb 3f
        vmovdqu32 ymmword ptr [r9+0x1C0], ymm30
        vmovdqu32 ymmword ptr [r9+0x1E0], ymm31
3:
        jmp 9b
        mov r11d, 0xAAAA
        kmovw k1, r11d
        mov r11d, 0x8888
        kmovw k2, r11d
        mov r11d, 0x55
        kmovw k3, r11d
        mov r11d, r10d
        shl r11, 0x20
        or r11, r8
        cmp al, 0x02
        jbe 2f
        vbroadcasti32x4 zmm0, xmmword ptr [rcx]
        vbroadcasti32x4 zmm1, xmmword ptr [rcx+0x10]
        vbroadcasti32x4 zmm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqa32 xmm4, xmmword ptr [rsp]
        vmovdqa32 xmm5, xmmword ptr [rsp+0x40]
        vpbroadcastq zmm3, r11
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti64x4 zmm6, zmm6, ymm5, 0x01
        vpermq zmm3 {k3}, zmm6, 0xDC
        vbroadcasti32x4 zmm8, xmmword ptr [rdx]
        vbroadcasti32x4 zmm9, xmmword ptr [rdx+0x10]
        vshufps zmm4, zmm8, zmm9, 0x88
        vshufps zmm5, zmm8, zmm9, 0xDD
        vbroadcasti32x4 zmm8, xmmword ptr [rdx+0x20]
        vbroadcasti32x4 zmm9, xmmword ptr [rdx+0x30]
        vshufps zmm6, zmm8, zmm9, 0x88
        vshufps zmm7, zmm8, zmm9, 0xDD
        vpshufd zmm6, zmm6, 0x93
        vpshufd zmm7, zmm7, 0x93
        mov r8b, 0x07
3:
        vpaddd zmm0, zmm0, zmm4
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x10
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x0C
        vpaddd zmm0, zmm0, zmm5
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x08
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x07
        vpshufd zmm0, zmm0, 0x93
        vpshufd zmm3, zmm3, 0x4E
        vpshufd zmm2, zmm2, 0x39
        vpaddd zmm0, zmm0, zmm6
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x10
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x0C
        vpaddd zmm0, zmm0, zmm7
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 0x08
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0x07
        vpshufd zmm0, zmm0, 0x39
        vpshufd zmm3, zmm3, 0x4E
        vpshufd zmm2, zmm2, 0x93
        dec r8b
        jz 3f
        vshufps zmm8, zmm4, zmm5, 0xD6
        vpshufd zmm9, zmm4, 0x0F
        vpshufd zmm4, zmm8, 0x39
        vshufps zmm8, zmm6, zmm7, 0xFA
        vpblendmd zmm9 {k1}, zmm9, zmm8
        vpunpcklqdq zmm8, zmm7, zmm5
        vpblendmd zmm8 {k2}, zmm8, zmm6
        vpshufd zmm8, zmm8, 0x78
        vpunpckhdq zmm5, zmm5, zmm7
        vpunpckldq zmm6, zmm6, zmm5
        vpshufd zmm7, zmm6, 0x1E
        vmovdqa32 zmm5, zmm9
        vmovdqa32 zmm6, zmm8
        jmp 3b
3:
        vpxord zmm0, zmm0, zmm2
        vpxord zmm1, zmm1, zmm3
        vbroadcasti32x4 zmm4, xmmword ptr [rcx]
        vbroadcasti32x4 zmm5, xmmword ptr [rcx+0x10]
        vpxord zmm2, zmm2, zmm4
        vpxord zmm3, zmm3, zmm5
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+0x10], xmm1
        vmovdqu xmmword ptr [r9+0x20], xmm2
        vmovdqu xmmword ptr [r9+0x30], xmm3
        vextracti128 xmmword ptr [r9+0x40], ymm0, 0x01
        vextracti128 xmmword ptr [r9+0x50], ymm1, 0x01
        vextracti128 xmmword ptr [r9+0x60], ymm2, 0x01
        vextracti128 xmmword ptr [r9+0x70], ymm3, 0x01
        vextracti32x4 xmmword ptr [r9+0x80], zmm0, 0x02
        vextracti32x4 xmmword ptr [r9+0x90], zmm1, 0x02
        vextracti32x4 xmmword ptr [r9+0xA0], zmm2, 0x02
        vextracti32x4 xmmword ptr [r9+0xB0], zmm3, 0x02
        cmp al, 0x04
        jb 3f
        vextracti32x4 xmmword ptr [r9+0xC0], zmm0, 0x03
        vextracti32x4 xmmword ptr [r9+0xD0], zmm1, 0x03
        vextracti32x4 xmmword ptr [r9+0xE0], zmm2, 0x03
        vextracti32x4 xmmword ptr [r9+0xF0], zmm3, 0x03
3:
        jmp 9b
2:
        test al, al
        jz 9b
        vbroadcasti32x4 ymm0, xmmword ptr [rcx]
        vbroadcasti32x4 ymm1, xmmword ptr [rcx+0x10]
        vbroadcasti32x4 ymm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqa xmm4, xmmword ptr [rsp]
        vmovdqa xmm5, xmmword ptr [rsp+0x40]
        vpbroadcastq ymm3, r11
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti128 ymm6, ymm6, xmm5, 0x01
        vpermq ymm3 {k3}, ymm6, 0xDC
        vbroadcasti32x4 ymm8, xmmword ptr [rdx]
        vbroadcasti32x4 ymm9, xmmword ptr [rdx+0x10]
        vshufps ymm4, ymm8, ymm9, 0x88
        vshufps ymm5, ymm8, ymm9, 0xDD
        vbroadcasti32x4 ymm8, xmmword ptr [rdx+0x20]
        vbroadcasti32x4 ymm9, xmmword ptr [rdx+0x30]
        vshufps ymm6, ymm8, ymm9, 0x88
        vshufps ymm7, ymm8, ymm9, 0xDD
        vpshufd ymm6, ymm6, 0x93
        vpshufd ymm7, ymm7, 0x93
        mov r8b, 0x07
3:
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x10
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x0C
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x08
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x07
        vpshufd ymm0, ymm0, 0x93
        vpshufd ymm3, ymm3, 0x4E
        vpshufd ymm2, ymm2, 0x39
        vpaddd ymm0, ymm0, ymm6
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x10
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x0C
        vpaddd ymm0, ymm0, ymm7
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 0x08
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0x07
        vpshufd ymm0, ymm0, 0x39
        vpshufd ymm3, ymm3, 0x4E
        vpshufd ymm2, ymm2, 0x93
        dec r8b
        jz 3f
        vshufps ymm8, ymm4, ymm5, 0xD6
        vpshufd ymm9, ymm4, 0x0F
        vpshufd ymm4, ymm8, 0x39
        vshufps ymm8, ymm6, ymm7, 0xFA
        vpblendd ymm9, ymm9, ymm8, 0xAA
        vpunpcklqdq ymm8, ymm7, ymm5
        vpblendd ymm8, ymm8, ymm6, 0x88
        vpshufd ymm8, ymm8, 0x78
        vpunpckhdq ymm5, ymm5, ymm7
        vpunpckldq ymm6, ymm6, ymm5
        vpshufd ymm7, ymm6, 0x1E
        vmovdqa ymm5, ymm9
        vmovdqa ymm6, ymm8
        jmp 3b
3:
        vpxord ymm0, ymm0, ymm2
        vpxord ymm1, ymm1, ymm3
        vbroadcasti128 ymm4, xmmword ptr [rcx]
        vbroadcasti128 ymm5, xmmword ptr [rcx+0x10]
        vpxord ymm2, ymm2, ymm4
        vpxord ymm3, ymm3, ymm5
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+0x10], xmm1
        vmovdqu xmmword ptr [r9+0x20], xmm2
        vmovdqu xmmword ptr [r9+0x30], xmm3
        cmp al, 0x02
        jb 3f
        vextracti128 xmmword ptr [r9+0x40], ymm0, 0x01
        vextracti128 xmmword ptr [r9+0x50], ymm1, 0x01
        vextracti128 xmmword ptr [r9+0x60], ymm2, 0x01
        vextracti128 xmmword ptr [r9+0x70], ymm3, 0x01
3:
        jmp 9b

.section .rdata
.p2align  6
ADD0:
        .long    0,  1,  2,  3,  4,  5,  6,  7
        .long    8,  9, 10, 11, 12, 13, 14, 15
INDEX0:
        .long    0,  1,  2,  3, 16, 17, 18, 19
        .long    8,  9, 10, 11, 24, 25, 26, 27
INDEX1:
        .long    4,  5,  6,  7, 20, 21, 22, 23
        .long   12, 13, 14, 15, 28, 29, 30, 31
BLAKE3_IV:
BLAKE3_IV_0:
        .long   0x6A09E667
BLAKE3_IV_1:
        .long   0xBB67AE85
BLAKE3_IV_2:
        .long   0x3C6EF372
BLAKE3_IV_3:
        .long   0xA54FF53A
ADD1:   .long    1
ADD16:  .long   16
BLAKE3_BLOCK_LEN:
        .long   64
