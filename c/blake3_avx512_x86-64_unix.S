#if defined(__ELF__) && defined(__linux__)
.section .note.GNU-stack,"",%progbits
#endif

#if defined(__ELF__) && defined(__CET__) && defined(__has_include)
#if __has_include(<cet.h>)
#include <cet.h>
#endif
#endif

#if !defined(_CET_ENDBR)
#define _CET_ENDBR
#endif

.intel_syntax noprefix
.global _blake3_hash_many_avx512
.global blake3_hash_many_avx512
.global blake3_compress_in_place_avx512
.global _blake3_compress_in_place_avx512
.global blake3_compress_xof_avx512
.global _blake3_compress_xof_avx512
.global blake3_xof_many_avx512
.global _blake3_xof_many_avx512

#ifdef __APPLE__
.text
#else
.section .text
#endif
.p2align  6
_blake3_hash_many_avx512:
blake3_hash_many_avx512:
        _CET_ENDBR
        push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15
        mov rbp, rsp
        sub rsp, 0x140
        and rsp, 0xFFFFFFFFFFFFFFC0
        movzx r9d, r9b
        neg r9d
        kmovw k1, r9d
        vpbroadcastd ymm0, r8d
        shr r8, 0x20
        vpbroadcastd ymm1, r8d
        vmovdqa32 ymm2 {k1} {z}, ymmword ptr [rip+ADD0+ 0]
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
        shl rdx, 0x06
        mov qword ptr [rsp+0x100], rdx
        cmp rsi, 0x10
        jb 5f
.p2align 5
2:
        vpbroadcastd zmm0, dword ptr [rcx]
        vpbroadcastd zmm1, dword ptr [rcx+0x04]
        vpbroadcastd zmm2, dword ptr [rcx+0x08]
        vpbroadcastd zmm3, dword ptr [rcx+0x0C]
        vpbroadcastd zmm4, dword ptr [rcx+0x10]
        vpbroadcastd zmm5, dword ptr [rcx+0x14]
        vpbroadcastd zmm6, dword ptr [rcx+0x18]
        vpbroadcastd zmm7, dword ptr [rcx+0x1C]
        movzx eax, byte ptr [rbp+0x38]
        movzx edx, byte ptr [rbp+0x40]
        or eax, edx
        xor edx, edx
3:
        movzx ebx, byte ptr [rbp+0x48]
        or ebx, eax
        add rdx, 0x40
        cmp rdx, qword ptr [rsp+0x100]
        cmovz eax, ebx
        mov dword ptr [rsp+0x80], eax
        mov rax, qword ptr [rdi]
        mov rbx, qword ptr [rdi+0x08]
        mov r8, qword ptr [rdi+0x10]
        mov r9, qword ptr [rdi+0x18]
        mov r10, qword ptr [rdi+0x40]
        mov r11, qword ptr [rdi+0x48]
        mov r12, qword ptr [rdi+0x50]
        mov r13, qword ptr [rdi+0x58]
        vmovdqu32 ymm8, ymmword ptr [rax+rdx*1-0x40]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rdx*1-0x40], 0x01
        vmovdqu32 ymm9, ymmword ptr [rbx+rdx*1-0x40]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rdx*1-0x40], 0x01
        vpunpckldq zmm10, zmm8, zmm9
        vpunpckhdq zmm11, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [r8+rdx*1-0x40]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rdx*1-0x40], 0x01
        vmovdqu32 ymm9, ymmword ptr [r9+rdx*1-0x40]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rdx*1-0x40], 0x01
        vpunpckldq zmm12, zmm8, zmm9
        vpunpckhdq zmm13, zmm8, zmm9
        mov rax, qword ptr [rdi+0x20]
        mov rbx, qword ptr [rdi+0x28]
        mov r8, qword ptr [rdi+0x30]
        mov r9, qword ptr [rdi+0x38]
        mov r10, qword ptr [rdi+0x60]
        mov r11, qword ptr [rdi+0x68]
        mov r12, qword ptr [rdi+0x70]
        mov r13, qword ptr [rdi+0x78]
        vmovdqu32 ymm8, ymmword ptr [rax+rdx*1-0x40]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rdx*1-0x40], 0x01
        vmovdqu32 ymm9, ymmword ptr [rbx+rdx*1-0x40]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rdx*1-0x40], 0x01
        vpunpckldq zmm14, zmm8, zmm9
        vpunpckhdq zmm15, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [r8+rdx*1-0x40]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rdx*1-0x40], 0x01
        vmovdqu32 ymm9, ymmword ptr [r9+rdx*1-0x40]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rdx*1-0x40], 0x01
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
        mov rax, qword ptr [rdi]
        mov rbx, qword ptr [rdi+0x08]
        mov r8, qword ptr [rdi+0x10]
        mov r9, qword ptr [rdi+0x18]
        mov r10, qword ptr [rdi+0x40]
        mov r11, qword ptr [rdi+0x48]
        mov r12, qword ptr [rdi+0x50]
        mov r13, qword ptr [rdi+0x58]
        vmovdqu32 ymm11, ymmword ptr [rax+rdx*1-0x20]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rdx*1-0x20], 0x01
        vmovdqu32 ymm13, ymmword ptr [rbx+rdx*1-0x20]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rdx*1-0x20], 0x01
        vpunpckldq zmm15, zmm11, zmm13
        vpunpckhdq zmm17, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [r8+rdx*1-0x20]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rdx*1-0x20], 0x01
        vmovdqu32 ymm13, ymmword ptr [r9+rdx*1-0x20]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rdx*1-0x20], 0x01
        vpunpckldq zmm22, zmm11, zmm13
        vpunpckhdq zmm23, zmm11, zmm13
        prefetcht0 byte ptr [rax+rdx*1+0x80]
        prefetcht0 byte ptr [rbx+rdx*1+0x80]
        prefetcht0 byte ptr [r8+rdx*1+0x80]
        prefetcht0 byte ptr [r9+rdx*1+0x80]
        prefetcht0 byte ptr [r10+rdx*1+0x80]
        prefetcht0 byte ptr [r11+rdx*1+0x80]
        prefetcht0 byte ptr [r12+rdx*1+0x80]
        prefetcht0 byte ptr [r13+rdx*1+0x80]
        mov rax, qword ptr [rdi+0x20]
        mov rbx, qword ptr [rdi+0x28]
        mov r8, qword ptr [rdi+0x30]
        mov r9, qword ptr [rdi+0x38]
        mov r10, qword ptr [rdi+0x60]
        mov r11, qword ptr [rdi+0x68]
        mov r12, qword ptr [rdi+0x70]
        mov r13, qword ptr [rdi+0x78]
        vmovdqu32 ymm11, ymmword ptr [rax+rdx*1-0x20]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rdx*1-0x20], 0x01
        vmovdqu32 ymm13, ymmword ptr [rbx+rdx*1-0x20]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rdx*1-0x20], 0x01
        vpunpckldq zmm24, zmm11, zmm13
        vpunpckhdq zmm25, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [r8+rdx*1-0x20]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rdx*1-0x20], 0x01
        vmovdqu32 ymm13, ymmword ptr [r9+rdx*1-0x20]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rdx*1-0x20], 0x01
        vpunpckldq zmm26, zmm11, zmm13
        vpunpckhdq zmm27, zmm11, zmm13
        prefetcht0 byte ptr [rax+rdx*1+0x80]
        prefetcht0 byte ptr [rbx+rdx*1+0x80]
        prefetcht0 byte ptr [r8+rdx*1+0x80]
        prefetcht0 byte ptr [r9+rdx*1+0x80]
        prefetcht0 byte ptr [r10+rdx*1+0x80]
        prefetcht0 byte ptr [r11+rdx*1+0x80]
        prefetcht0 byte ptr [r12+rdx*1+0x80]
        prefetcht0 byte ptr [r13+rdx*1+0x80]
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
        movzx eax, byte ptr [rbp+0x38]
        jb 3b
        mov rbx, qword ptr [rbp+0x50]
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
        vextracti64x4 ymmword ptr [rbx], zmm8, 0x00
        vextracti64x4 ymmword ptr [rbx+0x20], zmm10, 0x00
        vextracti64x4 ymmword ptr [rbx+0x40], zmm12, 0x00
        vextracti64x4 ymmword ptr [rbx+0x60], zmm14, 0x00
        vextracti64x4 ymmword ptr [rbx+0x80], zmm0, 0x00
        vextracti64x4 ymmword ptr [rbx+0xA0], zmm1, 0x00
        vextracti64x4 ymmword ptr [rbx+0xC0], zmm2, 0x00
        vextracti64x4 ymmword ptr [rbx+0xE0], zmm3, 0x00
        vextracti64x4 ymmword ptr [rbx+0x100], zmm8, 0x01
        vextracti64x4 ymmword ptr [rbx+0x120], zmm10, 0x01
        vextracti64x4 ymmword ptr [rbx+0x140], zmm12, 0x01
        vextracti64x4 ymmword ptr [rbx+0x160], zmm14, 0x01
        vextracti64x4 ymmword ptr [rbx+0x180], zmm0, 0x01
        vextracti64x4 ymmword ptr [rbx+0x1A0], zmm1, 0x01
        vextracti64x4 ymmword ptr [rbx+0x1C0], zmm2, 0x01
        vextracti64x4 ymmword ptr [rbx+0x1E0], zmm3, 0x01
        vmovdqa32 zmm8, zmmword ptr [rsp]
        vmovdqa32 zmm9, zmmword ptr [rsp+0x40]
        vmovdqa32 zmm10, zmm8
        vpaddd zmm10 {k1}, zmm8, dword ptr [rip+ADD16] {1to16}
        vpcmpud k2 {k1}, zmm10, zmm8, 0x01
        vpaddd zmm9 {k2}, zmm9, dword ptr [rip+ADD1] {1to16}
        vmovdqa32 zmmword ptr [rsp], zmm10
        vmovdqa32 zmmword ptr [rsp+0x40], zmm9
        add rbx, 0x200
        mov qword ptr [rbp+0x50], rbx
        add rdi, 0x80
        sub rsi, 0x10
        cmp rsi, 0x10
        jnb 2b
        test esi, esi
        jnz 5f
9:
        vzeroupper
        mov rsp, rbp
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbp
        pop rbx
        ret
.p2align 6
5:
        mov rax, rsp
        test sil, 0x08
        jz 3f
        vpbroadcastd ymm0, dword ptr [rcx]
        vpbroadcastd ymm1, dword ptr [rcx+0x04]
        vpbroadcastd ymm2, dword ptr [rcx+0x08]
        vpbroadcastd ymm3, dword ptr [rcx+0x0C]
        vpbroadcastd ymm4, dword ptr [rcx+0x10]
        vpbroadcastd ymm5, dword ptr [rcx+0x14]
        vpbroadcastd ymm6, dword ptr [rcx+0x18]
        vpbroadcastd ymm7, dword ptr [rcx+0x1C]
        movzx edx, byte ptr [rbp+0x38]
        movzx ebx, byte ptr [rbp+0x40]
        or edx, ebx
        xor ebx, ebx
2:
        movzx r8d, byte ptr [rbp+0x48]
        or r8d, edx
        add rbx, 0x40
        cmp rbx, qword ptr [rsp+0x100]
        cmovz edx, r8d
        mov dword ptr [rsp+0x80], edx
        mov edx, 0xCC
        kmovw k2, edx
        mov edx, 0x33
        kmovw k3, edx
        mov rdx, qword ptr [rdi]
        mov r8, qword ptr [rdi+0x20]
        vmovups xmm8, xmmword ptr [rdx+rbx*1-0x40]
        vinserti32x4 ymm8, ymm8, xmmword ptr [r8+rbx*1-0x40], 0x01
        vmovups xmm12, xmmword ptr [rdx+rbx*1-0x30]
        vinserti32x4 ymm12, ymm12, xmmword ptr [r8+rbx*1-0x30], 0x01
        mov rdx, qword ptr [rdi+0x08]
        mov r8, qword ptr [rdi+0x28]
        vmovups xmm9, xmmword ptr [rdx+rbx*1-0x40]
        vinserti32x4 ymm9, ymm9, xmmword ptr [r8+rbx*1-0x40], 0x01
        vmovups xmm13, xmmword ptr [rdx+rbx*1-0x30]
        vinserti32x4 ymm13, ymm13, xmmword ptr [r8+rbx*1-0x30], 0x01
        mov rdx, qword ptr [rdi+0x10]
        mov r8, qword ptr [rdi+0x30]
        vmovups xmm10, xmmword ptr [rdx+rbx*1-0x40]
        vinserti32x4 ymm10, ymm10, xmmword ptr [r8+rbx*1-0x40], 0x01
        vmovups xmm14, xmmword ptr [rdx+rbx*1-0x30]
        vinserti32x4 ymm14, ymm14, xmmword ptr [r8+rbx*1-0x30], 0x01
        mov rdx, qword ptr [rdi+0x18]
        mov r8, qword ptr [rdi+0x38]
        vmovups xmm11, xmmword ptr [rdx+rbx*1-0x40]
        vinserti32x4 ymm11, ymm11, xmmword ptr [r8+rbx*1-0x40], 0x01
        vmovups xmm15, xmmword ptr [rdx+rbx*1-0x30]
        vinserti32x4 ymm15, ymm15, xmmword ptr [r8+rbx*1-0x30], 0x01
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
        mov rdx, qword ptr [rdi]
        mov r8, qword ptr [rdi+0x20]
        vmovups xmm16, xmmword ptr [rdx+rbx*1-0x20]
        vinserti32x4 ymm16, ymm16, xmmword ptr [r8+rbx*1-0x20], 0x01
        vmovups xmm20, xmmword ptr [rdx+rbx*1-0x10]
        vinserti32x4 ymm20, ymm20, xmmword ptr [r8+rbx*1-0x10], 0x01
        mov rdx, qword ptr [rdi+0x08]
        mov r8, qword ptr [rdi+0x28]
        vmovups xmm17, xmmword ptr [rdx+rbx*1-0x20]
        vinserti32x4 ymm17, ymm17, xmmword ptr [r8+rbx*1-0x20], 0x01
        vmovups xmm21, xmmword ptr [rdx+rbx*1-0x10]
        vinserti32x4 ymm21, ymm21, xmmword ptr [r8+rbx*1-0x10], 0x01
        mov rdx, qword ptr [rdi+0x10]
        mov r8, qword ptr [rdi+0x30]
        vmovups xmm18, xmmword ptr [rdx+rbx*1-0x20]
        vinserti32x4 ymm18, ymm18, xmmword ptr [r8+rbx*1-0x20], 0x01
        vmovups xmm22, xmmword ptr [rdx+rbx*1-0x10]
        vinserti32x4 ymm22, ymm22, xmmword ptr [r8+rbx*1-0x10], 0x01
        mov rdx, qword ptr [rdi+0x18]
        mov r8, qword ptr [rdi+0x38]
        vmovups xmm19, xmmword ptr [rdx+rbx*1-0x20]
        vinserti32x4 ymm19, ymm19, xmmword ptr [r8+rbx*1-0x20], 0x01
        vmovups xmm23, xmmword ptr [rdx+rbx*1-0x10]
        vinserti32x4 ymm23, ymm23, xmmword ptr [r8+rbx*1-0x10], 0x01
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
        vmovdqa32 ymm28, ymmword ptr [rax]
        vmovdqa32 ymm29, ymmword ptr [rax+0x40]
        vpbroadcastd ymm30, dword ptr [rip+BLAKE3_BLOCK_LEN]
        vpbroadcastd ymm31, dword ptr [rsp+0x80]
        mov dl, 0x07
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
        dec dl
        jnz 4b
        vpxord ymm0, ymm0, ymm21
        vpxord ymm1, ymm1, ymm25
        vpxord ymm2, ymm2, ymm26
        vpxord ymm3, ymm3, ymm27
        vpxord ymm4, ymm4, ymm28
        vpxord ymm5, ymm5, ymm29
        vpxord ymm6, ymm6, ymm30
        vpxord ymm7, ymm7, ymm31
        movzx edx, byte ptr [rbp+0x38]
        jb 2b
        mov r8, qword ptr [rbp+0x50]
        vunpcklps ymm8, ymm0, ymm1
        vunpcklps ymm9, ymm2, ymm3
        vunpckhps ymm10, ymm0, ymm1
        vunpcklps ymm11, ymm4, ymm5
        vunpcklps ymm0, ymm6, ymm7
        vshufps ymm12, ymm8, ymm9, 0x4E
        vblendps ymm1, ymm8, ymm12, 0xCC
        vshufps ymm8, ymm11, ymm0, 0x4E
        vunpckhps ymm13, ymm2, ymm3
        vblendps ymm2, ymm11, ymm8, 0xCC
        vblendps ymm3, ymm12, ymm9, 0xCC
        vperm2f128 ymm12, ymm1, ymm2, 0x20
        vmovups ymmword ptr [r8], ymm12
        vunpckhps ymm14, ymm4, ymm5
        vblendps ymm4, ymm8, ymm0, 0xCC
        vunpckhps ymm15, ymm6, ymm7
        vperm2f128 ymm7, ymm3, ymm4, 0x20
        vmovups ymmword ptr [r8+0x20], ymm7
        vshufps ymm5, ymm10, ymm13, 0x4E
        vblendps ymm6, ymm5, ymm13, 0xCC
        vshufps ymm13, ymm14, ymm15, 0x4E
        vblendps ymm10, ymm10, ymm5, 0xCC
        vblendps ymm14, ymm14, ymm13, 0xCC
        vperm2f128 ymm8, ymm10, ymm14, 0x20
        vmovups ymmword ptr [r8+0x40], ymm8
        vblendps ymm15, ymm13, ymm15, 0xCC
        vperm2f128 ymm13, ymm6, ymm15, 0x20
        vmovups ymmword ptr [r8+0x60], ymm13
        vperm2f128 ymm9, ymm1, ymm2, 0x31
        vperm2f128 ymm11, ymm3, ymm4, 0x31
        vmovups ymmword ptr [r8+0x80], ymm9
        vperm2f128 ymm14, ymm10, ymm14, 0x31
        vperm2f128 ymm15, ymm6, ymm15, 0x31
        vmovups ymmword ptr [r8+0xA0], ymm11
        vmovups ymmword ptr [r8+0xC0], ymm14
        vmovups ymmword ptr [r8+0xE0], ymm15
        lea r9, qword ptr [rax+0x20]
        kortestw k1, k1
        cmovnz rax, r9
        add r8, 0x100
        mov qword ptr [rbp+0x50], r8
        add rdi, 0x40
3:
        mov rdx, qword ptr [rbp+0x50]
        movzx ebx, byte ptr [rbp+0x38]
        movzx r8d, byte ptr [rbp+0x48]
        test sil, 0x04
        jz 3f
        vbroadcasti32x4 zmm0, xmmword ptr [rcx]
        vbroadcasti32x4 zmm1, xmmword ptr [rcx+0x10]
        vbroadcasti32x4 zmm4, xmmword ptr [rip+BLAKE3_IV]
        mov r9d, 0x4444
        kmovw k2, r9d
        vmovdqa xmm6, xmmword ptr [rax]
        vmovdqa xmm7, xmmword ptr [rax+0x40]
        vpunpckldq xmm8, xmm6, xmm7
        vpunpckhdq xmm9, xmm6, xmm7
        vpermq ymm8, ymm8, 0xDC
        vpermq ymm9, ymm9, 0xDC
        vpbroadcastd zmm6, dword ptr [rip+BLAKE3_BLOCK_LEN]
        vinserti64x4 zmm5, zmm8, ymm9, 0x01
        vpblendmd zmm5 {k2}, zmm5, zmm6
        mov r9, qword ptr [rdi]
        mov r10, qword ptr [rdi+0x08]
        mov r11, qword ptr [rdi+0x10]
        mov r12, qword ptr [rdi+0x18]
        mov r13d, 0xAAAA
        kmovw k2, r13d
        mov r13d, 0x8888
        kmovw k3, r13d
        movzx r13d, byte ptr [rbp+0x40]
        or r13d, ebx
        xor r14d, r14d
2:
        movzx r15d, byte ptr [rbp+0x48]
        or r15d, r13d
        add r14, 0x40
        cmp r14, qword ptr [rsp+0x100]
        cmovz r13d, r15d
        mov dword ptr [rsp+0x80], r13d
        vmovdqa32 zmm2, zmm4
        vpbroadcastd zmm6, dword ptr [rsp+0x80]
        vpblendmd zmm3 {k3}, zmm5, zmm6
        vmovdqu32 zmm10, zmmword ptr [r9+r14*1-0x40]
        vinserti32x4 zmm10, zmm10, xmmword ptr [r10+r14*1-0x40], 0x01
        vinserti32x4 zmm10, zmm10, xmmword ptr [r11+r14*1-0x40], 0x02
        vinserti32x4 zmm10, zmm10, xmmword ptr [r12+r14*1-0x40], 0x03
        vmovdqu32 zmm11, zmmword ptr [r9+r14*1-0x30]
        vinserti32x4 zmm11, zmm11, xmmword ptr [r10+r14*1-0x30], 0x01
        vinserti32x4 zmm11, zmm11, xmmword ptr [r11+r14*1-0x30], 0x02
        vinserti32x4 zmm11, zmm11, xmmword ptr [r12+r14*1-0x30], 0x03
        vshufps zmm6, zmm10, zmm11, 0x88
        vshufps zmm7, zmm10, zmm11, 0xDD
        vmovdqu32 zmm10, zmmword ptr [r9+r14*1-0x20]
        vinserti32x4 zmm10, zmm10, xmmword ptr [r10+r14*1-0x20], 0x01
        vinserti32x4 zmm10, zmm10, xmmword ptr [r11+r14*1-0x20], 0x02
        vinserti32x4 zmm10, zmm10, xmmword ptr [r12+r14*1-0x20], 0x03
        vmovdqu32 zmm11, zmmword ptr [r9+r14*1-0x10]
        vinserti32x4 zmm11, zmm11, xmmword ptr [r10+r14*1-0x10], 0x01
        vinserti32x4 zmm11, zmm11, xmmword ptr [r11+r14*1-0x10], 0x02
        vinserti32x4 zmm11, zmm11, xmmword ptr [r12+r14*1-0x10], 0x03
        vshufps zmm8, zmm10, zmm11, 0x88
        vshufps zmm9, zmm10, zmm11, 0xDD
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
        vshufps zmm12, zmm6, zmm7, 0xD6
        vpshufd zmm13, zmm6, 0x0F
        vpshufd zmm6, zmm12, 0x39
        vshufps zmm12, zmm8, zmm9, 0xFA
        vpblendmd zmm13 {k2}, zmm13, zmm12
        vpunpcklqdq zmm12, zmm9, zmm7
        vpblendmd zmm12 {k3}, zmm12, zmm8
        vpshufd zmm12, zmm12, 0x78
        vpunpckhdq zmm7, zmm7, zmm9
        vpunpckldq zmm8, zmm8, zmm7
        vpshufd zmm9, zmm8, 0x1E
        vmovdqa32 zmm7, zmm13
        vmovdqa32 zmm8, zmm12
        jmp 4b
4:
        vpxord zmm0, zmm0, zmm2
        vpxord zmm1, zmm1, zmm3
        mov r13d, ebx
        jb 2b
        vmovdqu xmmword ptr [rdx], xmm0
        vmovdqu xmmword ptr [rdx+0x10], xmm1
        vextracti128 xmmword ptr [rdx+0x20], ymm0, 0x01
        vextracti128 xmmword ptr [rdx+0x30], ymm1, 0x01
        vextracti32x4 xmmword ptr [rdx+0x40], zmm0, 0x02
        vextracti32x4 xmmword ptr [rdx+0x50], zmm1, 0x02
        vextracti32x4 xmmword ptr [rdx+0x60], zmm0, 0x03
        vextracti32x4 xmmword ptr [rdx+0x70], zmm1, 0x03
        lea r15, qword ptr [rax+0x10]
        kortestw k1, k1
        cmovnz rax, r15
        add rdx, 0x80
        add rdi, 0x20
3:
        test sil, 0x02
        jz 3f
        vbroadcasti128 ymm0, xmmword ptr [rcx]
        vbroadcasti128 ymm1, xmmword ptr [rcx+0x10]
        vbroadcasti128 ymm4, xmmword ptr [rip+BLAKE3_IV]
        vmovd xmm5, dword ptr [rax]
        vpinsrd xmm5, xmm5, dword ptr [rax+0x40], 0x01
        vpinsrd xmm5, xmm5, dword ptr [rip+BLAKE3_BLOCK_LEN], 0x02
        vmovd xmm6, dword ptr [rax+0x04]
        vpinsrd xmm6, xmm6, dword ptr [rax+0x44], 0x01
        vpinsrd xmm6, xmm6, dword ptr [rip+BLAKE3_BLOCK_LEN], 0x02
        vinserti128 ymm5, ymm5, xmm6, 0x01
        mov r9, qword ptr [rdi]
        mov r10, qword ptr [rdi+0x08]
        mov r11d, ebx
        movzx r12d, byte ptr [rbp+0x40]
        or r11d, r12d
        xor r12d, r12d
2:
        movzx r13d, byte ptr [rbp+0x48]
        or r13d, r11d
        add r12, 0x40
        cmp r12, qword ptr [rsp+0x100]
        cmovz r11d, r13d
        mov dword ptr [rsp+0x80], r11d
        vmovdqa ymm2, ymm4
        vpbroadcastd ymm6, dword ptr [rsp+0x80]
        vpblendd ymm3, ymm5, ymm6, 0x88
        vmovdqu ymm10, ymmword ptr [r9+r12*1-0x40]
        vinserti128 ymm10, ymm10, xmmword ptr [r10+r12*1-0x40], 0x01
        vmovdqu ymm11, ymmword ptr [r9+r12*1-0x30]
        vinserti128 ymm11, ymm11, xmmword ptr [r10+r12*1-0x30], 0x01
        vshufps ymm6, ymm10, ymm11, 0x88
        vshufps ymm7, ymm10, ymm11, 0xDD
        vmovdqu ymm10, ymmword ptr [r9+r12*1-0x20]
        vinserti128 ymm10, ymm10, xmmword ptr [r10+r12*1-0x20], 0x01
        vmovdqu ymm11, ymmword ptr [r9+r12*1-0x10]
        vinserti128 ymm11, ymm11, xmmword ptr [r10+r12*1-0x10], 0x01
        vshufps ymm8, ymm10, ymm11, 0x88
        vshufps ymm9, ymm10, ymm11, 0xDD
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
        mov r11d, ebx
        jb 2b
        vmovdqu xmmword ptr [rdx], xmm0
        vmovdqu xmmword ptr [rdx+0x10], xmm1
        vextracti128 xmmword ptr [rdx+0x20], ymm0, 0x01
        vextracti128 xmmword ptr [rdx+0x30], ymm1, 0x01
        lea r13, qword ptr [rax+0x08]
        kortestw k1, k1
        cmovnz rax, r13
        add rdx, 0x40
        add rdi, 0x10
3:
        test sil, 0x01
        jz 9b
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+0x10]
        vmovdqa xmm4, xmmword ptr [rip+BLAKE3_IV]
        vmovd xmm5, dword ptr [rax]
        vpinsrd xmm5, xmm5, dword ptr [rax+0x40], 0x01
        vpinsrd xmm5, xmm5, dword ptr [rip+BLAKE3_BLOCK_LEN], 0x02
        mov r9, qword ptr [rdi]
        mov r10d, ebx
        movzx r11d, byte ptr [rbp+0x40]
        or r10d, r11d
        xor r11d, r11d
2:
        movzx r12d, byte ptr [rbp+0x48]
        or r12d, r10d
        add r11, 0x40
        cmp r11, qword ptr [rsp+0x100]
        cmovz r10d, r12d
        vmovdqa xmm2, xmm4
        vpinsrd xmm3, xmm5, r10d, 0x03
        vmovdqu xmm10, xmmword ptr [r9+r11*1-0x40]
        vmovdqu xmm11, xmmword ptr [r9+r11*1-0x30]
        vshufps xmm6, xmm10, xmm11, 0x88
        vshufps xmm7, xmm10, xmm11, 0xDD
        vmovdqu xmm10, xmmword ptr [r9+r11*1-0x20]
        vmovdqu xmm11, xmmword ptr [r9+r11*1-0x10]
        vshufps xmm8, xmm10, xmm11, 0x88
        vshufps xmm9, xmm10, xmm11, 0xDD
        vpshufd xmm8, xmm8, 0x93
        vpshufd xmm9, xmm9, 0x93
        mov r12b, 0x07
4:
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
        vpshufd xmm0, xmm0, 0x93
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x39
        vpaddd xmm0, xmm0, xmm8
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x10
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x0C
        vpaddd xmm0, xmm0, xmm9
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 0x08
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0x07
        vpshufd xmm0, xmm0, 0x39
        vpshufd xmm3, xmm3, 0x4E
        vpshufd xmm2, xmm2, 0x93
        dec r12b
        jz 4f
        vshufps xmm10, xmm6, xmm7, 0xD6
        vpshufd xmm11, xmm6, 0x0F
        vpshufd xmm6, xmm10, 0x39
        vshufps xmm10, xmm8, xmm9, 0xFA
        vpblendd xmm11, xmm11, xmm10, 0xAA
        vpunpcklqdq xmm10, xmm9, xmm7
        vpblendd xmm10, xmm10, xmm8, 0x88
        vpshufd xmm10, xmm10, 0x78
        vpunpckhdq xmm7, xmm7, xmm9
        vpunpckldq xmm8, xmm8, xmm7
        vpshufd xmm9, xmm8, 0x1E
        vmovdqa xmm7, xmm11
        vmovdqa xmm8, xmm10
        jmp 4b
4:
        vpxor xmm0, xmm0, xmm2
        vpxor xmm1, xmm1, xmm3
        mov r10d, ebx
        jb 2b
        vmovdqu xmmword ptr [rdx], xmm0
        vmovdqu xmmword ptr [rdx+0x10], xmm1
        jmp 9b

.p2align 6
_blake3_compress_in_place_avx512:
blake3_compress_in_place_avx512:
        _CET_ENDBR
        vmovdqu xmm0, xmmword ptr [rdi]
        vmovdqu xmm1, xmmword ptr [rdi+0x10]
        movzx   eax, r8b
        movzx   edx, dl
        shl     rax, 32
        add     rdx, rax
        vmovq   xmm3, rcx
        vmovq   xmm4, rdx
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]
        vmovups xmm8, xmmword ptr [rsi]
        vmovups xmm9, xmmword ptr [rsi+0x10]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rsi+0x20]
        vmovups xmm9, xmmword ptr [rsi+0x30]
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
        vmovdqu xmmword ptr [rdi], xmm0
        vmovdqu xmmword ptr [rdi+0x10], xmm1
        ret

.p2align 6
_blake3_compress_xof_avx512:
blake3_compress_xof_avx512:
        _CET_ENDBR
        vmovdqu xmm0, xmmword ptr [rdi]
        vmovdqu xmm1, xmmword ptr [rdi+0x10]
        movzx   eax, r8b
        movzx   edx, dl
        shl     rax, 32
        add     rdx, rax
        vmovq   xmm3, rcx
        vmovq   xmm4, rdx
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]
        vmovups xmm8, xmmword ptr [rsi]
        vmovups xmm9, xmmword ptr [rsi+0x10]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rsi+0x20]
        vmovups xmm9, xmmword ptr [rsi+0x30]
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
        vpxor   xmm2, xmm2, [rdi]
        vpxor   xmm3, xmm3, [rdi+0x10]
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+0x10], xmm1
        vmovdqu xmmword ptr [r9+0x20], xmm2
        vmovdqu xmmword ptr [r9+0x30], xmm3
        ret

.p2align 6
blake3_xof_many_avx512:
_blake3_xof_many_avx512:
        _CET_ENDBR
        mov rax, qword ptr [rsp+0x08]
        cmp rax, 0x01
        jnbe 2f
        vmovdqu xmm0, xmmword ptr [rdi]
        vmovdqu xmm1, xmmword ptr [rdi+0x10]
        movzx edx, dl
        movzx r8d, r8b
        shl r8, 0x20
        or rdx, r8
        vmovq xmm2, rdx
        vmovq xmm3, rcx
        vpunpcklqdq xmm3, xmm3, xmm2
        vmovaps xmm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqu xmm8, xmmword ptr [rsi]
        vmovdqu xmm9, xmmword ptr [rsi+0x10]
        vshufps xmm4, xmm8, xmm9, 0x88
        vshufps xmm5, xmm8, xmm9, 0xDD
        vmovdqu xmm8, xmmword ptr [rsi+0x20]
        vmovdqu xmm9, xmmword ptr [rsi+0x30]
        vshufps xmm6, xmm8, xmm9, 0x88
        vshufps xmm7, xmm8, xmm9, 0xDD
        vpshufd xmm6, xmm6, 0x93
        vpshufd xmm7, xmm7, 0x93
        mov cl, 0x07
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
        dec cl
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
        vpxor xmm0, xmm0, xmm2
        vpxor xmm1, xmm1, xmm3
        vpxor xmm2, xmm2, xmmword ptr [rdi]
        vpxor xmm3, xmm3, xmmword ptr [rdi+0x10]
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+0x10], xmm1
        vmovdqu xmmword ptr [r9+0x20], xmm2
        vmovdqu xmmword ptr [r9+0x30], xmm3
        vzeroupper
        ret
2:
        push rbp
        mov rbp, rsp
        sub rsp, 0x100
        and rsp, 0xFFFFFFFFFFFFFFC0
        vpbroadcastd zmm0, ecx
        shr rcx, 0x20
        vpbroadcastd zmm1, ecx
        vpaddd zmm2, zmm0, zmmword ptr [rip+ADD0]
        vpcmpud k1, zmm2, zmm0, 0x01
        vpaddd zmm1 {k1}, zmm1, dword ptr [rip+ADD1] {1to16}
        vmovdqa32 zmmword ptr [rsp], zmm2
        vmovdqa32 zmmword ptr [rsp+0x40], zmm1
        movzx edx, dl
        movzx r8d, r8b
        cmp rax, 0x08
        jbe 2f
.p2align 5
3:
        vpbroadcastd zmm0, dword ptr [rsi]
        vpbroadcastd zmm1, dword ptr [rsi+0x04]
        vpbroadcastd zmm2, dword ptr [rsi+0x08]
        vpbroadcastd zmm3, dword ptr [rsi+0x0C]
        vpbroadcastd zmm4, dword ptr [rsi+0x10]
        vpbroadcastd zmm5, dword ptr [rsi+0x14]
        vpbroadcastd zmm6, dword ptr [rsi+0x18]
        vpbroadcastd zmm7, dword ptr [rsi+0x1C]
        vpbroadcastd zmm8, dword ptr [rsi+0x20]
        vpbroadcastd zmm9, dword ptr [rsi+0x24]
        vpbroadcastd zmm10, dword ptr [rsi+0x28]
        vpbroadcastd zmm11, dword ptr [rsi+0x2C]
        vpbroadcastd zmm12, dword ptr [rsi+0x30]
        vpbroadcastd zmm13, dword ptr [rsi+0x34]
        vpbroadcastd zmm14, dword ptr [rsi+0x38]
        vpbroadcastd zmm15, dword ptr [rsi+0x3C]
        vpbroadcastd zmm16, dword ptr [rdi]
        vpbroadcastd zmm17, dword ptr [rdi+0x04]
        vpbroadcastd zmm18, dword ptr [rdi+0x08]
        vpbroadcastd zmm19, dword ptr [rdi+0x0C]
        vpbroadcastd zmm20, dword ptr [rdi+0x10]
        vpbroadcastd zmm21, dword ptr [rdi+0x14]
        vpbroadcastd zmm22, dword ptr [rdi+0x18]
        vpbroadcastd zmm23, dword ptr [rdi+0x1C]
        vpbroadcastd zmm24, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd zmm25, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd zmm26, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd zmm27, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa32 zmm28, zmmword ptr [rsp]
        vmovdqa32 zmm29, zmmword ptr [rsp+0x40]
        vpbroadcastd zmm30, edx
        vpbroadcastd zmm31, r8d
        mov cl, 0x07
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
        dec cl
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
        vbroadcasti64x4 zmm8, ymmword ptr [rdi]
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
        cmp rax, 0x18
        lea rax, qword ptr [rax-0x10]
        jnbe 3b
        test al, al
        jnz 2f
9:
        vzeroupper
        mov rsp, rbp
        pop rbp
        ret
2:
        cmp al, 0x04
        jbe 2f
        vpbroadcastd ymm16, dword ptr [rsi]
        vpbroadcastd ymm17, dword ptr [rsi+0x04]
        vpbroadcastd ymm18, dword ptr [rsi+0x08]
        vpbroadcastd ymm19, dword ptr [rsi+0x0C]
        vpbroadcastd ymm20, dword ptr [rsi+0x10]
        vpbroadcastd ymm21, dword ptr [rsi+0x14]
        vpbroadcastd ymm22, dword ptr [rsi+0x18]
        vpbroadcastd ymm23, dword ptr [rsi+0x1C]
        vpbroadcastd ymm24, dword ptr [rsi+0x20]
        vpbroadcastd ymm25, dword ptr [rsi+0x24]
        vpbroadcastd ymm26, dword ptr [rsi+0x28]
        vpbroadcastd ymm27, dword ptr [rsi+0x2C]
        vpbroadcastd ymm28, dword ptr [rsi+0x30]
        vpbroadcastd ymm29, dword ptr [rsi+0x34]
        vpbroadcastd ymm30, dword ptr [rsi+0x38]
        vpbroadcastd ymm31, dword ptr [rsi+0x3C]
        vpbroadcastd ymm0, dword ptr [rdi]
        vpbroadcastd ymm1, dword ptr [rdi+0x04]
        vpbroadcastd ymm2, dword ptr [rdi+0x08]
        vpbroadcastd ymm3, dword ptr [rdi+0x0C]
        vpbroadcastd ymm4, dword ptr [rdi+0x10]
        vpbroadcastd ymm5, dword ptr [rdi+0x14]
        vpbroadcastd ymm6, dword ptr [rdi+0x18]
        vpbroadcastd ymm7, dword ptr [rdi+0x1C]
        vpbroadcastd ymm8, dword ptr [rip+BLAKE3_IV_0]
        vpbroadcastd ymm9, dword ptr [rip+BLAKE3_IV_1]
        vpbroadcastd ymm10, dword ptr [rip+BLAKE3_IV_2]
        vpbroadcastd ymm11, dword ptr [rip+BLAKE3_IV_3]
        vmovdqa ymm12, ymmword ptr [rsp]
        vmovdqa ymm13, ymmword ptr [rsp+0x40]
        vpbroadcastd ymm14, edx
        vpbroadcastd ymm15, r8d
        mov cl, 0x07
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
        dec cl
        jnz 3b
        vpxord ymm0, ymm0, ymm8
        vpxord ymm8, ymm8, dword ptr [rdi] {1to8}
        vpxord ymm1, ymm1, ymm9
        vpxord ymm9, ymm9, dword ptr [rdi+0x04] {1to8}
        vpxord ymm2, ymm2, ymm10
        vpxord ymm10, ymm10, dword ptr [rdi+0x08] {1to8}
        vpxord ymm3, ymm3, ymm11
        vpxord ymm11, ymm11, dword ptr [rdi+0x0C] {1to8}
        vpxord ymm4, ymm4, ymm12
        vpxord ymm12, ymm12, dword ptr [rdi+0x10] {1to8}
        vpxord ymm5, ymm5, ymm13
        vpxord ymm13, ymm13, dword ptr [rdi+0x14] {1to8}
        vpxord ymm6, ymm6, ymm14
        vpxord ymm14, ymm14, dword ptr [rdi+0x18] {1to8}
        vpxord ymm7, ymm7, ymm15
        vpxord ymm15, ymm15, dword ptr [rdi+0x1C] {1to8}
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
2:
        mov ecx, 0xAAAA
        kmovw k1, ecx
        mov ecx, 0x8888
        kmovw k2, ecx
        mov ecx, 0x55
        kmovw k3, ecx
        mov ecx, r8d
        shl rcx, 0x20
        or rcx, rdx
        cmp al, 0x02
        jbe 2f
        vbroadcasti32x4 zmm0, xmmword ptr [rdi]
        vbroadcasti32x4 zmm1, xmmword ptr [rdi+0x10]
        vbroadcasti32x4 zmm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqa32 xmm4, xmmword ptr [rsp]
        vmovdqa32 xmm5, xmmword ptr [rsp+0x40]
        vpbroadcastq zmm3, rcx
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti64x4 zmm6, zmm6, ymm5, 0x01
        vpermq zmm3 {k3}, zmm6, 0xDC
        vbroadcasti32x4 zmm8, xmmword ptr [rsi]
        vbroadcasti32x4 zmm9, xmmword ptr [rsi+0x10]
        vshufps zmm4, zmm8, zmm9, 0x88
        vshufps zmm5, zmm8, zmm9, 0xDD
        vbroadcasti32x4 zmm8, xmmword ptr [rsi+0x20]
        vbroadcasti32x4 zmm9, xmmword ptr [rsi+0x30]
        vshufps zmm6, zmm8, zmm9, 0x88
        vshufps zmm7, zmm8, zmm9, 0xDD
        vpshufd zmm6, zmm6, 0x93
        vpshufd zmm7, zmm7, 0x93
        mov dl, 0x07
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
        dec dl
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
        vbroadcasti32x4 zmm4, xmmword ptr [rdi]
        vbroadcasti32x4 zmm5, xmmword ptr [rdi+0x10]
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
        vbroadcasti32x4 ymm0, xmmword ptr [rdi]
        vbroadcasti32x4 ymm1, xmmword ptr [rdi+0x10]
        vbroadcasti32x4 ymm2, xmmword ptr [rip+BLAKE3_IV]
        vmovdqa xmm4, xmmword ptr [rsp]
        vmovdqa xmm5, xmmword ptr [rsp+0x40]
        vpbroadcastq ymm3, rcx
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti128 ymm6, ymm6, xmm5, 0x01
        vpermq ymm3 {k3}, ymm6, 0xDC
        vbroadcasti32x4 ymm8, xmmword ptr [rsi]
        vbroadcasti32x4 ymm9, xmmword ptr [rsi+0x10]
        vshufps ymm4, ymm8, ymm9, 0x88
        vshufps ymm5, ymm8, ymm9, 0xDD
        vbroadcasti32x4 ymm8, xmmword ptr [rsi+0x20]
        vbroadcasti32x4 ymm9, xmmword ptr [rsi+0x30]
        vshufps ymm6, ymm8, ymm9, 0x88
        vshufps ymm7, ymm8, ymm9, 0xDD
        vpshufd ymm6, ymm6, 0x93
        vpshufd ymm7, ymm7, 0x93
        mov dl, 0x07
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
        dec dl
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
        vbroadcasti128 ymm4, xmmword ptr [rdi]
        vbroadcasti128 ymm5, xmmword ptr [rdi+0x10]
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

#ifdef __APPLE__
.static_data
#else
.section .rodata
#endif
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

