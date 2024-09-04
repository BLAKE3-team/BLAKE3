public _blake3_hash_many_avx512
public blake3_hash_many_avx512
public blake3_compress_in_place_avx512
public _blake3_compress_in_place_avx512
public blake3_compress_xof_avx512
public _blake3_compress_xof_avx512
public _blake3_xof_many_avx512
public blake3_xof_many_avx512

_TEXT   SEGMENT ALIGN(16) 'CODE'

ALIGN   16
blake3_hash_many_avx512 PROC
_blake3_hash_many_avx512 PROC
        push rbx
        push rbp
        push rsi
        push rdi
        push r12
        push r13
        push r14
        push r15
        mov rbp, rsp
        sub rsp, 1E8h
        movaps xmmword ptr [rbp-0A8h], xmm6
        movaps xmmword ptr [rbp-98h], xmm7
        movaps xmmword ptr [rbp-88h], xmm8
        movaps xmmword ptr [rbp-78h], xmm9
        movaps xmmword ptr [rbp-68h], xmm10
        movaps xmmword ptr [rbp-58h], xmm11
        movaps xmmword ptr [rbp-48h], xmm12
        movaps xmmword ptr [rbp-38h], xmm13
        movaps xmmword ptr [rbp-28h], xmm14
        movaps xmmword ptr [rbp-18h], xmm15
        and rsp, -40h
        mov rax, qword ptr [rbp+68h]
        movzx ebx, byte ptr [rbp+70h]
        neg ebx
        kmovw k1, ebx
        vpbroadcastd ymm0, eax
        shr rax, 20h
        vpbroadcastd ymm1, eax
        vmovdqa32 ymm2 {k1} {z}, ymmword ptr [ADD0+0]
        vmovdqa32 ymm3 {k1} {z}, ymmword ptr [ADD0+32]
        vpaddd ymm2, ymm0, ymm2
        vmovdqa ymmword ptr [rsp], ymm2
        vpaddd ymm3, ymm0, ymm3
        vmovdqa ymmword ptr [rsp+20h], ymm3
        vpcmpud k2 {k1}, ymm2, ymm0, 1h
        vpcmpud k3 {k1}, ymm3, ymm0, 1h
        vmovdqa ymm2, ymm1
        vpaddd ymm2 {k2}, ymm2, dword bcst [ADD1]
        vmovdqa ymmword ptr [rsp+40h], ymm2
        vpaddd ymm1 {k3}, ymm1, dword bcst [ADD1]
        vmovdqa ymmword ptr [rsp+60h], ymm1
        shl r8, 6h
        mov qword ptr [rsp+100h], r8
        cmp rdx, 8h
        jbe final8blocks
ALIGN 16
outerloop16:
        vpbroadcastd zmm0, dword ptr [r9]
        vpbroadcastd zmm1, dword ptr [r9+4h]
        vpbroadcastd zmm2, dword ptr [r9+8h]
        vpbroadcastd zmm3, dword ptr [r9+0Ch]
        vpbroadcastd zmm4, dword ptr [r9+10h]
        vpbroadcastd zmm5, dword ptr [r9+14h]
        vpbroadcastd zmm6, dword ptr [r9+18h]
        vpbroadcastd zmm7, dword ptr [r9+1Ch]
        movzx eax, byte ptr [rbp+78h]
        movzx ebx, byte ptr [rbp+80h]
        or eax, ebx
        xor ebx, ebx
innerloop16:
        movzx esi, byte ptr [rbp+88h]
        or esi, eax
        add rbx, 40h
        cmp rbx, qword ptr [rsp+100h]
        cmovz eax, esi
        mov dword ptr [rsp+80h], eax
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+8h]
        mov rdi, qword ptr [rcx+10h]
        mov r8, qword ptr [rcx+18h]
        mov r10, qword ptr [rcx+40h]
        vmovdqu32 ymm8, ymmword ptr [rax+rbx*1-40h]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rbx*1-40h], 1h
        vmovdqu32 ymm9, ymmword ptr [rsi+rbx*1-40h]
        cmp rdx, 0Ah
        jb @F
        mov r11, qword ptr [rcx+48h]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rbx*1-40h], 1h
@@:
        vpunpckldq zmm10, zmm8, zmm9
        vpunpckhdq zmm11, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [rdi+rbx*1-40h]
        cmp rdx, 0Bh
        jb @F
        mov r12, qword ptr [rcx+50h]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rbx*1-40h], 1h
@@:
        vmovdqu32 ymm9, ymmword ptr [r8+rbx*1-40h]
        cmp rdx, 0Ch
        jb @F
        mov r13, qword ptr [rcx+58h]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rbx*1-40h], 1h
@@:
        vpunpckldq zmm12, zmm8, zmm9
        vpunpckhdq zmm13, zmm8, zmm9
        mov rax, qword ptr [rcx+20h]
        mov rsi, qword ptr [rcx+28h]
        mov rdi, qword ptr [rcx+30h]
        mov r8, qword ptr [rcx+38h]
        vmovdqu32 ymm8, ymmword ptr [rax+rbx*1-40h]
        cmp rdx, 0Dh
        jb @F
        mov r10, qword ptr [rcx+60h]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r10+rbx*1-40h], 1h
@@:
        vmovdqu32 ymm9, ymmword ptr [rsi+rbx*1-40h]
        cmp rdx, 0Eh
        jb @F
        mov r11, qword ptr [rcx+68h]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r11+rbx*1-40h], 1h
@@:
        vpunpckldq zmm14, zmm8, zmm9
        vpunpckhdq zmm15, zmm8, zmm9
        vmovdqu32 ymm8, ymmword ptr [rdi+rbx*1-40h]
        cmp rdx, 0Fh
        jb @F
        mov r12, qword ptr [rcx+70h]
        vinserti64x4 zmm8, zmm8, ymmword ptr [r12+rbx*1-40h], 1h
@@:
        vmovdqu32 ymm9, ymmword ptr [r8+rbx*1-40h]
        cmp rdx, 10h
        jb @F
        mov r13, qword ptr [rcx+78h]
        vinserti64x4 zmm9, zmm9, ymmword ptr [r13+rbx*1-40h], 1h
@@:
        vpunpckldq zmm16, zmm8, zmm9
        vpunpckhdq zmm17, zmm8, zmm9
        vmovdqa32 zmm8, zmmword ptr [INDEX0]
        vmovdqa32 zmm9, zmmword ptr [INDEX1]
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
        mov rsi, qword ptr [rcx+8h]
        mov rdi, qword ptr [rcx+10h]
        mov r8, qword ptr [rcx+18h]
        mov r10, qword ptr [rcx+40h]
        vmovdqu32 ymm11, ymmword ptr [rax+rbx*1-20h]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rbx*1-20h], 1h
        vmovdqu32 ymm13, ymmword ptr [rsi+rbx*1-20h]
        cmp rdx, 0Ah
        jb @F
        mov r11, qword ptr [rcx+48h]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rbx*1-20h], 1h
        prefetcht0 byte ptr [r11+rbx*1+80h]
@@:
        vpunpckldq zmm15, zmm11, zmm13
        vpunpckhdq zmm17, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [rdi+rbx*1-20h]
        cmp rdx, 0Bh
        jb @F
        mov r12, qword ptr [rcx+50h]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rbx*1-20h], 1h
        prefetcht0 byte ptr [r13+rbx*1+80h]
@@:
        vmovdqu32 ymm13, ymmword ptr [r8+rbx*1-20h]
        cmp rdx, 0Ch
        jb @F
        mov r13, qword ptr [rcx+58h]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rbx*1-20h], 1h
        prefetcht0 byte ptr [r13+rbx*1+80h]
@@:
        vpunpckldq zmm22, zmm11, zmm13
        vpunpckhdq zmm23, zmm11, zmm13
        prefetcht0 byte ptr [rax+rbx*1+80h]
        prefetcht0 byte ptr [rsi+rbx*1+80h]
        prefetcht0 byte ptr [rdi+rbx*1+80h]
        prefetcht0 byte ptr [r8+rbx*1+80h]
        prefetcht0 byte ptr [r10+rbx*1+80h]
        mov rax, qword ptr [rcx+20h]
        mov rsi, qword ptr [rcx+28h]
        mov rdi, qword ptr [rcx+30h]
        mov r8, qword ptr [rcx+38h]
        vmovdqu32 ymm11, ymmword ptr [rax+rbx*1-20h]
        cmp rdx, 0Dh
        jb @F
        mov r10, qword ptr [rcx+60h]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r10+rbx*1-20h], 1h
        prefetcht0 byte ptr [r10+rbx*1+80h]
@@:
        vmovdqu32 ymm13, ymmword ptr [rsi+rbx*1-20h]
        cmp rdx, 0Eh
        jb @F
        mov r11, qword ptr [rcx+68h]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r11+rbx*1-20h], 1h
        prefetcht0 byte ptr [r11+rbx*1+80h]
@@:
        vpunpckldq zmm24, zmm11, zmm13
        vpunpckhdq zmm25, zmm11, zmm13
        vmovdqu32 ymm11, ymmword ptr [rdi+rbx*1-20h]
        cmp rdx, 0Fh
        jb @F
        mov r12, qword ptr [rcx+70h]
        vinserti64x4 zmm11, zmm11, ymmword ptr [r12+rbx*1-20h], 1h
        prefetcht0 byte ptr [r12+rbx*1+80h]
@@:
        vmovdqu32 ymm13, ymmword ptr [r8+rbx*1-20h]
        cmp rdx, 10h
        jb @F
        mov r13, qword ptr [rcx+78h]
        vinserti64x4 zmm13, zmm13, ymmword ptr [r13+rbx*1-20h], 1h
        prefetcht0 byte ptr [r13+rbx*1+80h]
@@:
        vpunpckldq zmm26, zmm11, zmm13
        vpunpckhdq zmm27, zmm11, zmm13
        prefetcht0 byte ptr [r10+rbx*1+80h]
        prefetcht0 byte ptr [r11+rbx*1+80h]
        prefetcht0 byte ptr [r12+rbx*1+80h]
        prefetcht0 byte ptr [r13+rbx*1+80h]
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
        vpbroadcastd zmm17, dword ptr [BLAKE3_IV_0]
        vpbroadcastd zmm23, dword ptr [BLAKE3_IV_1]
        vpbroadcastd zmm24, dword ptr [BLAKE3_IV_2]
        vpbroadcastd zmm25, dword ptr [BLAKE3_IV_3]
        vmovdqa32 zmm26, zmmword ptr [rsp]
        vmovdqa32 zmm27, zmmword ptr [rsp+40h]
        vpbroadcastd zmm30, dword ptr [BLAKE3_BLOCK_LEN]
        vpbroadcastd zmm31, dword ptr [rsp+80h]
        mov al, 7h
@@:
        vpaddd zmm0, zmm0, zmm18
        vpaddd zmm1, zmm1, zmm10
        vpaddd zmm2, zmm2, zmm19
        vpaddd zmm3, zmm3, zmm12
        vmovdqa32 zmmword ptr [rsp+80h], zmm18
        vmovdqa32 zmmword ptr [rsp+0C0h], zmm12
        vpaddd zmm0, zmm0, zmm4
        vpaddd zmm1, zmm1, zmm5
        vpaddd zmm2, zmm2, zmm6
        vpaddd zmm3, zmm3, zmm7
        vpxord zmm26, zmm26, zmm0
        vmovdqa32 zmm18, zmm10
        vpxord zmm27, zmm27, zmm1
        vpxord zmm30, zmm30, zmm2
        vpxord zmm31, zmm31, zmm3
        vprord zmm26, zmm26, 10h
        vprord zmm27, zmm27, 10h
        vprord zmm30, zmm30, 10h
        vprord zmm31, zmm31, 10h
        vpaddd zmm17, zmm17, zmm26
        vmovdqa32 zmm12, zmm19
        vpaddd zmm23, zmm23, zmm27
        vpaddd zmm24, zmm24, zmm30
        vpaddd zmm25, zmm25, zmm31
        vpxord zmm4, zmm4, zmm17
        vpxord zmm5, zmm5, zmm23
        vpxord zmm6, zmm6, zmm24
        vpxord zmm7, zmm7, zmm25
        vprord zmm4, zmm4, 0Ch
        vprord zmm5, zmm5, 0Ch
        vprord zmm6, zmm6, 0Ch
        vprord zmm7, zmm7, 0Ch
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
        vprord zmm26, zmm26, 8h
        vmovdqa32 zmm19, zmm16
        vprord zmm27, zmm27, 8h
        vprord zmm30, zmm30, 8h
        vprord zmm31, zmm31, 8h
        vpaddd zmm17, zmm17, zmm26
        vpaddd zmm23, zmm23, zmm27
        vpaddd zmm24, zmm24, zmm30
        vpaddd zmm25, zmm25, zmm31
        vpxord zmm4, zmm4, zmm17
        vmovdqa32 zmm14, zmm15
        vpxord zmm5, zmm5, zmm23
        vpxord zmm6, zmm6, zmm24
        vpxord zmm7, zmm7, zmm25
        vprord zmm4, zmm4, 7h
        vprord zmm5, zmm5, 7h
        vprord zmm6, zmm6, 7h
        vprord zmm7, zmm7, 7h
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
        vprord zmm31, zmm31, 10h
        vprord zmm26, zmm26, 10h
        vprord zmm27, zmm27, 10h
        vprord zmm30, zmm30, 10h
        vpaddd zmm24, zmm24, zmm31
        vmovdqa32 zmm13, zmm28
        vpaddd zmm25, zmm25, zmm26
        vpaddd zmm17, zmm17, zmm27
        vpaddd zmm23, zmm23, zmm30
        vpxord zmm5, zmm5, zmm24
        vpxord zmm6, zmm6, zmm25
        vpxord zmm7, zmm7, zmm17
        vpxord zmm4, zmm4, zmm23
        vprord zmm5, zmm5, 0Ch
        vprord zmm6, zmm6, 0Ch
        vprord zmm7, zmm7, 0Ch
        vprord zmm4, zmm4, 0Ch
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
        vprord zmm31, zmm31, 8h
        vmovdqa32 zmm8, zmm21
        vprord zmm26, zmm26, 8h
        vprord zmm27, zmm27, 8h
        vprord zmm30, zmm30, 8h
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
        vmovdqa32 zmm21, zmmword ptr [rsp+80h]
        vprord zmm5, zmm5, 7h
        vprord zmm6, zmm6, 7h
        vprord zmm7, zmm7, 7h
        vprord zmm4, zmm4, 7h
        vmovdqa32 zmm20, zmmword ptr [rsp+0C0h]
        dec al
        jnz @B
        vpxord zmm0, zmm0, zmm17
        vpxord zmm1, zmm1, zmm23
        vpxord zmm2, zmm2, zmm24
        vpxord zmm3, zmm3, zmm25
        vpxord zmm4, zmm4, zmm26
        vpxord zmm5, zmm5, zmm27
        vpxord zmm6, zmm6, zmm30
        vpxord zmm7, zmm7, zmm31
        movzx eax, byte ptr [rbp+78h]
        cmp rbx, qword ptr [rsp+100h]
        jb innerloop16
        mov rsi, qword ptr [rbp+90h]
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
        vmovdqa32 zmm16, zmmword ptr [INDEX0]
        vmovdqa32 zmm18, zmmword ptr [INDEX1]
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
        vextracti64x4 ymmword ptr [rsi], zmm8, 0h
        vextracti64x4 ymmword ptr [rsi+20h], zmm10, 0h
        vextracti64x4 ymmword ptr [rsi+40h], zmm12, 0h
        vextracti64x4 ymmword ptr [rsi+60h], zmm14, 0h
        vextracti64x4 ymmword ptr [rsi+80h], zmm0, 0h
        vextracti64x4 ymmword ptr [rsi+0A0h], zmm1, 0h
        vextracti64x4 ymmword ptr [rsi+0C0h], zmm2, 0h
        vextracti64x4 ymmword ptr [rsi+0E0h], zmm3, 0h
        vextracti64x4 ymmword ptr [rsi+100h], zmm8, 1h
        cmp rdx, 0Ah
        jb unwind
        vextracti64x4 ymmword ptr [rsi+120h], zmm10, 1h
        cmp rdx, 0Bh
        jb unwind
        vextracti64x4 ymmword ptr [rsi+140h], zmm12, 1h
        cmp rdx, 0Ch
        jb unwind
        vextracti64x4 ymmword ptr [rsi+160h], zmm14, 1h
        cmp rdx, 0Dh
        jb unwind
        vextracti64x4 ymmword ptr [rsi+180h], zmm0, 1h
        cmp rdx, 0Eh
        jb unwind
        vextracti64x4 ymmword ptr [rsi+1A0h], zmm1, 1h
        cmp rdx, 0Fh
        jb unwind
        vextracti64x4 ymmword ptr [rsi+1C0h], zmm2, 1h
        cmp rdx, 10h
        jb unwind
        vextracti64x4 ymmword ptr [rsi+1E0h], zmm3, 1h
        vmovdqa32 zmm8, zmmword ptr [rsp]
        vmovdqa32 zmm9, zmmword ptr [rsp+40h]
        vmovdqa32 zmm10, zmm8
        vpaddd zmm10 {k1}, zmm8, dword bcst [ADD16]
        vpcmpud k2 {k1}, zmm10, zmm8, 1h
        vpaddd zmm9 {k2}, zmm9, dword bcst [ADD1]
        vmovdqa32 zmmword ptr [rsp], zmm10
        vmovdqa32 zmmword ptr [rsp+40h], zmm9
        add rsi, 200h
        mov qword ptr [rbp+90h], rsi
        add rcx, 80h
        sub rdx, 10h
        cmp rdx, 8h
        jnbe outerloop16
        test rdx, rdx
        jnz final8blocks
unwind:
        vzeroupper
        movaps xmm6, xmmword ptr [rbp-0A8h]
        movaps xmm7, xmmword ptr [rbp-98h]
        movaps xmm8, xmmword ptr [rbp-88h]
        movaps xmm9, xmmword ptr [rbp-78h]
        movaps xmm10, xmmword ptr [rbp-68h]
        movaps xmm11, xmmword ptr [rbp-58h]
        movaps xmm12, xmmword ptr [rbp-48h]
        movaps xmm13, xmmword ptr [rbp-38h]
        movaps xmm14, xmmword ptr [rbp-28h]
        movaps xmm15, xmmword ptr [rbp-18h]
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
ALIGN 16
final8blocks:
        cmp dl, 4h
        jbe final4blocks
        vpbroadcastd ymm0, dword ptr [r9]
        vpbroadcastd ymm1, dword ptr [r9+4h]
        vpbroadcastd ymm2, dword ptr [r9+8h]
        vpbroadcastd ymm3, dword ptr [r9+0Ch]
        vpbroadcastd ymm4, dword ptr [r9+10h]
        vpbroadcastd ymm5, dword ptr [r9+14h]
        vpbroadcastd ymm6, dword ptr [r9+18h]
        vpbroadcastd ymm7, dword ptr [r9+1Ch]
        movzx eax, byte ptr [rbp+78h]
        movzx ebx, byte ptr [rbp+80h]
        or eax, ebx
        xor ebx, ebx
innerloop8:
        movzx esi, byte ptr [rbp+88h]
        or esi, eax
        add rbx, 40h
        cmp rbx, qword ptr [rsp+100h]
        cmovz eax, esi
        mov dword ptr [rsp+80h], eax
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+20h]
        vmovups xmm8, xmmword ptr [rax+rbx*1-40h]
        vinserti32x4 ymm8, ymm8, xmmword ptr [rsi+rbx*1-40h], 1h
        vmovups xmm12, xmmword ptr [rax+rbx*1-30h]
        vinserti32x4 ymm12, ymm12, xmmword ptr [rsi+rbx*1-30h], 1h
        mov rax, qword ptr [rcx+8h]
        vmovups xmm9, xmmword ptr [rax+rbx*1-40h]
        vmovups xmm13, xmmword ptr [rax+rbx*1-30h]
        cmp dl, 6h
        jb @F
        mov rsi, qword ptr [rcx+28h]
        vinserti32x4 ymm9, ymm9, xmmword ptr [rsi+rbx*1-40h], 1h
        vinserti32x4 ymm13, ymm13, xmmword ptr [rsi+rbx*1-30h], 1h
@@:
        mov rax, qword ptr [rcx+10h]
        vmovups xmm10, xmmword ptr [rax+rbx*1-40h]
        vmovups xmm14, xmmword ptr [rax+rbx*1-30h]
        cmp dl, 7h
        jb @F
        mov rsi, qword ptr [rcx+30h]
        vinserti32x4 ymm10, ymm10, xmmword ptr [rsi+rbx*1-40h], 1h
        vinserti32x4 ymm14, ymm14, xmmword ptr [rsi+rbx*1-30h], 1h
@@:
        mov rax, qword ptr [rcx+18h]
        vmovups xmm11, xmmword ptr [rax+rbx*1-40h]
        vmovups xmm15, xmmword ptr [rax+rbx*1-30h]
        cmp dl, 8h
        jb @F
        mov rsi, qword ptr [rcx+38h]
        vinserti32x4 ymm11, ymm11, xmmword ptr [rsi+rbx*1-40h], 1h
        vinserti32x4 ymm15, ymm15, xmmword ptr [rsi+rbx*1-30h], 1h
@@:
        vpunpckldq ymm24, ymm8, ymm9
        vpunpckhdq ymm9, ymm8, ymm9
        vpunpckldq ymm8, ymm10, ymm11
        vpunpckhdq ymm11, ymm10, ymm11
        vpunpckldq ymm10, ymm12, ymm13
        vpunpckhdq ymm13, ymm12, ymm13
        vpunpckldq ymm12, ymm14, ymm15
        vpunpckhdq ymm15, ymm14, ymm15
        vshufps ymm14, ymm24, ymm8, 44h
        vshufps ymm8, ymm24, ymm8, 0EEh
        vshufps ymm24, ymm9, ymm11, 44h
        vshufps ymm11, ymm9, ymm11, 0EEh
        vshufps ymm9, ymm10, ymm12, 44h
        vshufps ymm12, ymm10, ymm12, 0EEh
        vshufps ymm10, ymm13, ymm15, 44h
        vshufps ymm15, ymm13, ymm15, 0EEh
        mov rax, qword ptr [rcx]
        mov rsi, qword ptr [rcx+20h]
        vmovups xmm16, xmmword ptr [rax+rbx*1-20h]
        vinserti32x4 ymm16, ymm16, xmmword ptr [rsi+rbx*1-20h], 1h
        vmovups xmm20, xmmword ptr [rax+rbx*1-10h]
        vinserti32x4 ymm20, ymm20, xmmword ptr [rsi+rbx*1-10h], 1h
        mov rax, qword ptr [rcx+8h]
        vmovups xmm17, xmmword ptr [rax+rbx*1-20h]
        vmovups xmm21, xmmword ptr [rax+rbx*1-10h]
        cmp dl, 6h
        jb @F
        mov rsi, qword ptr [rcx+28h]
        vinserti32x4 ymm17, ymm17, xmmword ptr [rsi+rbx*1-20h], 1h
        vinserti32x4 ymm21, ymm21, xmmword ptr [rsi+rbx*1-10h], 1h
@@:
        mov rax, qword ptr [rcx+10h]
        vmovups xmm18, xmmword ptr [rax+rbx*1-20h]
        vmovups xmm22, xmmword ptr [rax+rbx*1-10h]
        cmp dl, 7h
        jb @F
        mov rsi, qword ptr [rcx+30h]
        vinserti32x4 ymm18, ymm18, xmmword ptr [rsi+rbx*1-20h], 1h
        vinserti32x4 ymm22, ymm22, xmmword ptr [rsi+rbx*1-10h], 1h
@@:
        mov rax, qword ptr [rcx+18h]
        vmovups xmm19, xmmword ptr [rax+rbx*1-20h]
        vmovups xmm23, xmmword ptr [rax+rbx*1-10h]
        cmp dl, 8h
        jb @F
        mov rsi, qword ptr [rcx+38h]
        vinserti32x4 ymm19, ymm19, xmmword ptr [rsi+rbx*1-20h], 1h
        vinserti32x4 ymm23, ymm23, xmmword ptr [rsi+rbx*1-10h], 1h
@@:
        vpunpckldq ymm13, ymm16, ymm17
        vpunpckhdq ymm17, ymm16, ymm17
        vpunpckldq ymm16, ymm18, ymm19
        vpunpckhdq ymm19, ymm18, ymm19
        vpunpckldq ymm18, ymm20, ymm21
        vpunpckhdq ymm21, ymm20, ymm21
        vpunpckldq ymm20, ymm22, ymm23
        vpunpckhdq ymm23, ymm22, ymm23
        vshufps ymm22, ymm13, ymm16, 44h
        vshufps ymm16, ymm13, ymm16, 0EEh
        vshufps ymm13, ymm17, ymm19, 44h
        vshufps ymm19, ymm17, ymm19, 0EEh
        vshufps ymm17, ymm18, ymm20, 44h
        vshufps ymm20, ymm18, ymm20, 0EEh
        vshufps ymm18, ymm21, ymm23, 44h
        vshufps ymm23, ymm21, ymm23, 0EEh
        vpbroadcastd ymm21, dword ptr [BLAKE3_IV_0]
        vpbroadcastd ymm25, dword ptr [BLAKE3_IV_1]
        vpbroadcastd ymm26, dword ptr [BLAKE3_IV_2]
        vpbroadcastd ymm27, dword ptr [BLAKE3_IV_3]
        vmovdqa32 ymm28, ymmword ptr [rsp]
        vmovdqa32 ymm29, ymmword ptr [rsp+40h]
        vpbroadcastd ymm30, dword ptr [BLAKE3_BLOCK_LEN]
        vpbroadcastd ymm31, dword ptr [rsp+80h]
        mov al, 7h
@@:
        vpaddd ymm0, ymm0, ymm14
        vpaddd ymm1, ymm1, ymm24
        vpaddd ymm2, ymm2, ymm9
        vpaddd ymm3, ymm3, ymm10
        vmovdqa32 ymmword ptr [rsp+80h], ymm14
        vmovdqa32 ymmword ptr [rsp+0C0h], ymm10
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm28, ymm28, ymm0
        vmovdqa32 ymm14, ymm24
        vpxord ymm29, ymm29, ymm1
        vpxord ymm30, ymm30, ymm2
        vpxord ymm31, ymm31, ymm3
        vprord ymm28, ymm28, 10h
        vprord ymm29, ymm29, 10h
        vprord ymm30, ymm30, 10h
        vprord ymm31, ymm31, 10h
        vpaddd ymm21, ymm21, ymm28
        vmovdqa32 ymm10, ymm9
        vpaddd ymm25, ymm25, ymm29
        vpaddd ymm26, ymm26, ymm30
        vpaddd ymm27, ymm27, ymm31
        vpxord ymm4, ymm4, ymm21
        vpxord ymm5, ymm5, ymm25
        vpxord ymm6, ymm6, ymm26
        vpxord ymm7, ymm7, ymm27
        vprord ymm4, ymm4, 0Ch
        vprord ymm5, ymm5, 0Ch
        vprord ymm6, ymm6, 0Ch
        vprord ymm7, ymm7, 0Ch
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
        vprord ymm28, ymm28, 8h
        vmovdqa32 ymm9, ymm15
        vprord ymm29, ymm29, 8h
        vprord ymm30, ymm30, 8h
        vprord ymm31, ymm31, 8h
        vpaddd ymm21, ymm21, ymm28
        vpaddd ymm25, ymm25, ymm29
        vpaddd ymm26, ymm26, ymm30
        vpaddd ymm27, ymm27, ymm31
        vpxord ymm4, ymm4, ymm21
        vmovdqa32 ymm11, ymm13
        vpxord ymm5, ymm5, ymm25
        vpxord ymm6, ymm6, ymm26
        vpxord ymm7, ymm7, ymm27
        vprord ymm4, ymm4, 7h
        vprord ymm5, ymm5, 7h
        vprord ymm6, ymm6, 7h
        vprord ymm7, ymm7, 7h
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
        vprord ymm31, ymm31, 10h
        vprord ymm28, ymm28, 10h
        vprord ymm29, ymm29, 10h
        vprord ymm30, ymm30, 10h
        vpaddd ymm26, ymm26, ymm31
        vmovdqa32 ymm17, ymm16
        vpaddd ymm27, ymm27, ymm28
        vpaddd ymm21, ymm21, ymm29
        vpaddd ymm25, ymm25, ymm30
        vpxord ymm5, ymm5, ymm26
        vpxord ymm6, ymm6, ymm27
        vpxord ymm7, ymm7, ymm21
        vpxord ymm4, ymm4, ymm25
        vprord ymm5, ymm5, 0Ch
        vprord ymm6, ymm6, 0Ch
        vprord ymm7, ymm7, 0Ch
        vprord ymm4, ymm4, 0Ch
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
        vprord ymm31, ymm31, 8h
        vmovdqa32 ymm19, ymm12
        vprord ymm28, ymm28, 8h
        vprord ymm29, ymm29, 8h
        vprord ymm30, ymm30, 8h
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
        vmovdqa32 ymm12, ymmword ptr [rsp+80h]
        vprord ymm5, ymm5, 7h
        vprord ymm6, ymm6, 7h
        vprord ymm7, ymm7, 7h
        vprord ymm4, ymm4, 7h
        vmovdqa32 ymm8, ymmword ptr [rsp+0C0h]
        dec al
        jnz @B
        vpxord ymm0, ymm0, ymm21
        vpxord ymm1, ymm1, ymm25
        vpxord ymm2, ymm2, ymm26
        vpxord ymm3, ymm3, ymm27
        vpxord ymm4, ymm4, ymm28
        vpxord ymm5, ymm5, ymm29
        vpxord ymm6, ymm6, ymm30
        vpxord ymm7, ymm7, ymm31
        movzx eax, byte ptr [rbp+78h]
        cmp rbx, qword ptr [rsp+100h]
        jb innerloop8
        mov rsi, qword ptr [rbp+90h]
        vpunpckldq ymm8, ymm0, ymm1
        vpunpckldq ymm9, ymm2, ymm3
        vpunpckhdq ymm10, ymm0, ymm1
        vpunpckldq ymm11, ymm4, ymm5
        vpunpckldq ymm0, ymm6, ymm7
        vshufps ymm12, ymm8, ymm9, 4Eh
        vpblendd ymm1, ymm8, ymm12, 0CCh
        vshufps ymm8, ymm11, ymm0, 4Eh
        vpunpckhdq ymm13, ymm2, ymm3
        vpblendd ymm2, ymm11, ymm8, 0CCh
        vpblendd ymm3, ymm12, ymm9, 0CCh
        vperm2i128 ymm12, ymm1, ymm2, 20h
        vmovdqu ymmword ptr [rsi], ymm12
        vpunpckhdq ymm14, ymm4, ymm5
        vpblendd ymm4, ymm8, ymm0, 0CCh
        vpunpckhdq ymm15, ymm6, ymm7
        vperm2i128 ymm7, ymm3, ymm4, 20h
        vmovdqu ymmword ptr [rsi+20h], ymm7
        vshufps ymm5, ymm10, ymm13, 4Eh
        vpblendd ymm6, ymm5, ymm13, 0CCh
        vshufps ymm13, ymm14, ymm15, 4Eh
        vpblendd ymm10, ymm10, ymm5, 0CCh
        vpblendd ymm14, ymm14, ymm13, 0CCh
        vperm2i128 ymm8, ymm10, ymm14, 20h
        vmovdqu ymmword ptr [rsi+40h], ymm8
        vpblendd ymm15, ymm13, ymm15, 0CCh
        vperm2i128 ymm13, ymm6, ymm15, 20h
        vmovdqu ymmword ptr [rsi+60h], ymm13
        vperm2i128 ymm9, ymm1, ymm2, 31h
        vmovdqu ymmword ptr [rsi+80h], ymm9
        cmp dl, 6h
        jb @F
        vperm2i128 ymm11, ymm3, ymm4, 31h
        vmovdqu ymmword ptr [rsi+0A0h], ymm11
        cmp dl, 7h
        jb @F
        vperm2i128 ymm14, ymm10, ymm14, 31h
        vmovdqu ymmword ptr [rsi+0C0h], ymm14
        cmp dl, 8h
        jb @F
        vperm2i128 ymm15, ymm6, ymm15, 31h
        vmovdqu ymmword ptr [rsi+0E0h], ymm15
@@:
        jmp unwind
final4blocks:
        mov rax, qword ptr [rsp+100h]
        mov rbx, qword ptr [rbp+90h]
        movzx esi, byte ptr [rbp+78h]
        movzx edi, byte ptr [rbp+88h]
        mov r8d, 0AAAAh
        kmovw k2, r8d
        mov r8d, 8888h
        kmovw k3, r8d
        cmp dl, 2h
        jbe final2blocks
        vbroadcasti32x4 zmm0, xmmword ptr [r9]
        vbroadcasti32x4 zmm1, xmmword ptr [r9+10h]
        vbroadcasti32x4 zmm4, xmmword ptr [BLAKE3_IV]
        mov r8d, 4444h
        kmovw k4, r8d
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+40h]
        vpbroadcastd zmm5, dword ptr [BLAKE3_BLOCK_LEN]
        vpunpckldq xmm8, xmm6, xmm7
        vpunpckhdq xmm7, xmm6, xmm7
        vinserti64x4 zmm8, zmm8, ymm7, 1h
        vpermq zmm8, zmm8, 0DCh
        vpblendmd zmm5 {k4}, zmm8, zmm5
        mov r8, qword ptr [rcx]
        mov r10, qword ptr [rcx+8h]
        mov r11, qword ptr [rcx+10h]
        cmp dl, 4h
        jb @F
        mov r12, qword ptr [rcx+18h]
@@:
        movzx r13d, byte ptr [rbp+80h]
        or r13d, esi
        xor r14d, r14d
innerloop4:
        movzx r15d, byte ptr [rbp+88h]
        or r15d, r13d
        add r14, 40h
        cmp r14, rax
        cmovz r13d, r15d
        mov dword ptr [rsp+80h], r13d
        vmovdqa32 zmm2, zmm4
        vpblendmd zmm3 {k3}, zmm5, dword bcst [rsp+80h]
        vmovdqu32 zmm10, zmmword ptr [r8+r14*1-40h]
        vmovdqu32 zmm11, zmmword ptr [r8+r14*1-30h]
        vmovdqu32 zmm12, zmmword ptr [r8+r14*1-20h]
        vmovdqu32 zmm13, zmmword ptr [r8+r14*1-10h]
        vinserti32x4 zmm10, zmm10, xmmword ptr [r10+r14*1-40h], 1h
        vinserti32x4 zmm11, zmm11, xmmword ptr [r10+r14*1-30h], 1h
        vinserti32x4 zmm12, zmm12, xmmword ptr [r10+r14*1-20h], 1h
        vinserti32x4 zmm13, zmm13, xmmword ptr [r10+r14*1-10h], 1h
        vinserti32x4 zmm10, zmm10, xmmword ptr [r11+r14*1-40h], 2h
        vinserti32x4 zmm11, zmm11, xmmword ptr [r11+r14*1-30h], 2h
        vinserti32x4 zmm12, zmm12, xmmword ptr [r11+r14*1-20h], 2h
        vinserti32x4 zmm13, zmm13, xmmword ptr [r11+r14*1-10h], 2h
        cmp dl, 4h
        jb @F
        vinserti32x4 zmm10, zmm10, xmmword ptr [r12+r14*1-40h], 3h
        vinserti32x4 zmm11, zmm11, xmmword ptr [r12+r14*1-30h], 3h
        vinserti32x4 zmm12, zmm12, xmmword ptr [r12+r14*1-20h], 3h
        vinserti32x4 zmm13, zmm13, xmmword ptr [r12+r14*1-10h], 3h
@@:
        vshufps zmm6, zmm10, zmm11, 88h
        vshufps zmm7, zmm10, zmm11, 0DDh
        vshufps zmm8, zmm12, zmm13, 88h
        vshufps zmm9, zmm12, zmm13, 0DDh
        vpshufd zmm8, zmm8, 93h
        vpshufd zmm9, zmm9, 93h
        mov r15b, 7h
@@:
        vpaddd zmm0, zmm0, zmm6
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 10h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0Ch
        vpaddd zmm0, zmm0, zmm7
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 8h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 7h
        vpshufd zmm0, zmm0, 93h
        vpshufd zmm3, zmm3, 4Eh
        vpshufd zmm2, zmm2, 39h
        vpaddd zmm0, zmm0, zmm8
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 10h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0Ch
        vpaddd zmm0, zmm0, zmm9
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 8h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 7h
        vpshufd zmm0, zmm0, 39h
        vpshufd zmm3, zmm3, 4Eh
        vpshufd zmm2, zmm2, 93h
        dec r15b
        jz @F
        vshufps zmm14, zmm6, zmm7, 0D6h
        vpshufd zmm15, zmm6, 0Fh
        vpshufd zmm6, zmm14, 39h
        vshufps zmm14, zmm8, zmm9, 0FAh
        vpblendmd zmm15 {k2}, zmm15, zmm14
        vpunpcklqdq zmm14, zmm9, zmm7
        vpblendmd zmm14 {k3}, zmm14, zmm8
        vpshufd zmm14, zmm14, 78h
        vpunpckhdq zmm7, zmm7, zmm9
        vpunpckldq zmm8, zmm8, zmm7
        vpshufd zmm9, zmm8, 1Eh
        vmovdqa32 zmm7, zmm15
        vmovdqa32 zmm8, zmm14
        jmp @B
@@:
        vpxord zmm0, zmm0, zmm2
        vpxord zmm1, zmm1, zmm3
        mov r13d, esi
        cmp r14, rax
        jb innerloop4
        vmovdqu xmmword ptr [rbx], xmm0
        vmovdqu xmmword ptr [rbx+10h], xmm1
        vextracti128 xmmword ptr [rbx+20h], ymm0, 1h
        vextracti128 xmmword ptr [rbx+30h], ymm1, 1h
        vextracti32x4 xmmword ptr [rbx+40h], zmm0, 2h
        vextracti32x4 xmmword ptr [rbx+50h], zmm1, 2h
        cmp dl, 4h
        jb @F
        vextracti32x4 xmmword ptr [rbx+60h], zmm0, 3h
        vextracti32x4 xmmword ptr [rbx+70h], zmm1, 3h
@@:
        jmp unwind
final2blocks:
        test dl, dl
        jz unwind
        vbroadcasti128 ymm0, xmmword ptr [r9]
        vbroadcasti128 ymm1, xmmword ptr [r9+10h]
        vbroadcasti128 ymm4, xmmword ptr [BLAKE3_IV]
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+40h]
        mov r8d, 40h
        vpbroadcastq ymm5, r8
        mov r8d, 55h
        kmovw k4, r8d
        vpunpckldq xmm8, xmm6, xmm7
        vpunpckhdq xmm7, xmm6, xmm7
        vinserti128 ymm8, ymm8, xmm7, 1h
        vpermq ymm5 {k4}, ymm8, 0DCh
        mov r8, qword ptr [rcx]
        cmp dl, 2h
        jb @F
        mov r10, qword ptr [rcx+8h]
@@:
        mov r11d, esi
        movzx r12d, byte ptr [rbp+80h]
        or r11d, r12d
        xor r12d, r12d
innerloop2:
        movzx r13d, byte ptr [rbp+88h]
        or r13d, r11d
        add r12, 40h
        cmp r12, rax
        cmovz r11d, r13d
        mov dword ptr [rsp+80h], r11d
        vmovdqa ymm2, ymm4
        vpblendmd ymm3 {k3}, ymm5, dword bcst [rsp+80h]
        vmovdqu ymm10, ymmword ptr [r8+r12*1-40h]
        vmovdqu ymm11, ymmword ptr [r8+r12*1-30h]
        vmovdqu ymm12, ymmword ptr [r8+r12*1-20h]
        vmovdqu ymm13, ymmword ptr [r8+r12*1-10h]
        cmp dl, 2h
        jb @F
        vinserti128 ymm10, ymm10, xmmword ptr [r10+r12*1-40h], 1h
        vinserti128 ymm11, ymm11, xmmword ptr [r10+r12*1-30h], 1h
        vinserti128 ymm12, ymm12, xmmword ptr [r10+r12*1-20h], 1h
        vinserti128 ymm13, ymm13, xmmword ptr [r10+r12*1-10h], 1h
@@:
        vshufps ymm6, ymm10, ymm11, 88h
        vshufps ymm7, ymm10, ymm11, 0DDh
        vshufps ymm8, ymm12, ymm13, 88h
        vshufps ymm9, ymm12, ymm13, 0DDh
        vpshufd ymm8, ymm8, 93h
        vpshufd ymm9, ymm9, 93h
        mov r13b, 7h
@@:
        vpaddd ymm0, ymm0, ymm6
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 10h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0Ch
        vpaddd ymm0, ymm0, ymm7
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 8h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 7h
        vpshufd ymm0, ymm0, 93h
        vpshufd ymm3, ymm3, 4Eh
        vpshufd ymm2, ymm2, 39h
        vpaddd ymm0, ymm0, ymm8
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 10h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0Ch
        vpaddd ymm0, ymm0, ymm9
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 8h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 7h
        vpshufd ymm0, ymm0, 39h
        vpshufd ymm3, ymm3, 4Eh
        vpshufd ymm2, ymm2, 93h
        dec r13b
        jz @F
        vshufps ymm10, ymm6, ymm7, 0D6h
        vpshufd ymm11, ymm6, 0Fh
        vpshufd ymm6, ymm10, 39h
        vshufps ymm10, ymm8, ymm9, 0FAh
        vpblendd ymm11, ymm11, ymm10, 0AAh
        vpunpcklqdq ymm10, ymm9, ymm7
        vpblendd ymm10, ymm10, ymm8, 88h
        vpshufd ymm10, ymm10, 78h
        vpunpckhdq ymm7, ymm7, ymm9
        vpunpckldq ymm8, ymm8, ymm7
        vpshufd ymm9, ymm8, 1Eh
        vmovdqa ymm7, ymm11
        vmovdqa ymm8, ymm10
        jmp @B
@@:
        vpxor ymm0, ymm0, ymm2
        vpxor ymm1, ymm1, ymm3
        mov r11d, esi
        cmp r12, rax
        jb innerloop2
        vmovdqu xmmword ptr [rbx], xmm0
        vmovdqu xmmword ptr [rbx+10h], xmm1
        cmp dl, 2h
        jb @F
        vextracti128 xmmword ptr [rbx+20h], ymm0, 1h
        vextracti128 xmmword ptr [rbx+30h], ymm1, 1h
@@:
        jmp unwind
_blake3_hash_many_avx512 ENDP
blake3_hash_many_avx512 ENDP

ALIGN 16
blake3_compress_in_place_avx512 PROC
_blake3_compress_in_place_avx512 PROC
        sub     rsp, 72
        vmovdqa xmmword ptr [rsp], xmm6
        vmovdqa xmmword ptr [rsp+10H], xmm7
        vmovdqa xmmword ptr [rsp+20H], xmm8
        vmovdqa xmmword ptr [rsp+30H], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+10H]
        movzx   eax, byte ptr [rsp+70H]
        movzx   r8d, r8b
        shl     rax, 32
        add     r8, rax
        vmovq   xmm3, r9
        vmovq   xmm4, r8
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV]
        vmovups xmm8, xmmword ptr [rdx]
        vmovups xmm9, xmmword ptr [rdx+10H]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rdx+20H]
        vmovups xmm9, xmmword ptr [rdx+30H]
        vshufps xmm6, xmm8, xmm9, 136
        vshufps xmm7, xmm8, xmm9, 221
        vpshufd xmm6, xmm6, 93H
        vpshufd xmm7, xmm7, 93H
        mov     al, 7
@@:
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
        vpshufd xmm0, xmm0, 93H
        vpshufd xmm3, xmm3, 4EH
        vpshufd xmm2, xmm2, 39H
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
        vpshufd xmm0, xmm0, 39H
        vpshufd xmm3, xmm3, 4EH
        vpshufd xmm2, xmm2, 93H
        dec     al
        jz      @F
        vshufps xmm8, xmm4, xmm5, 214
        vpshufd xmm9, xmm4, 0FH
        vpshufd xmm4, xmm8, 39H
        vshufps xmm8, xmm6, xmm7, 250
        vpblendd xmm9, xmm9, xmm8, 0AAH
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 88H
        vpshufd xmm8, xmm8, 78H
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 1EH
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp     @B
@@:
        vpxor   xmm0, xmm0, xmm2
        vpxor   xmm1, xmm1, xmm3
        vmovdqu xmmword ptr [rcx], xmm0
        vmovdqu xmmword ptr [rcx+10H], xmm1
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+10H]
        vmovdqa xmm8, xmmword ptr [rsp+20H]
        vmovdqa xmm9, xmmword ptr [rsp+30H]
        add     rsp, 72
        ret
_blake3_compress_in_place_avx512 ENDP
blake3_compress_in_place_avx512 ENDP

ALIGN 16
blake3_compress_xof_avx512 PROC
_blake3_compress_xof_avx512 PROC
        sub     rsp, 72
        vmovdqa xmmword ptr [rsp], xmm6
        vmovdqa xmmword ptr [rsp+10H], xmm7
        vmovdqa xmmword ptr [rsp+20H], xmm8
        vmovdqa xmmword ptr [rsp+30H], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+10H]
        movzx   eax, byte ptr [rsp+70H]
        movzx   r8d, r8b
        mov     r10, qword ptr [rsp+78H]
        shl     rax, 32
        add     r8, rax
        vmovq   xmm3, r9
        vmovq   xmm4, r8
        vpunpcklqdq xmm3, xmm3, xmm4
        vmovaps xmm2, xmmword ptr [BLAKE3_IV]
        vmovups xmm8, xmmword ptr [rdx]
        vmovups xmm9, xmmword ptr [rdx+10H]
        vshufps xmm4, xmm8, xmm9, 136
        vshufps xmm5, xmm8, xmm9, 221
        vmovups xmm8, xmmword ptr [rdx+20H]
        vmovups xmm9, xmmword ptr [rdx+30H]
        vshufps xmm6, xmm8, xmm9, 136
        vshufps xmm7, xmm8, xmm9, 221
        vpshufd xmm6, xmm6, 93H
        vpshufd xmm7, xmm7, 93H
        mov     al, 7
@@:
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
        vpshufd xmm0, xmm0, 93H
        vpshufd xmm3, xmm3, 4EH
        vpshufd xmm2, xmm2, 39H
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
        vpshufd xmm0, xmm0, 39H
        vpshufd xmm3, xmm3, 4EH
        vpshufd xmm2, xmm2, 93H
        dec     al
        jz      @F
        vshufps xmm8, xmm4, xmm5, 214
        vpshufd xmm9, xmm4, 0FH
        vpshufd xmm4, xmm8, 39H
        vshufps xmm8, xmm6, xmm7, 250
        vpblendd xmm9, xmm9, xmm8, 0AAH
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 88H
        vpshufd xmm8, xmm8, 78H
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 1EH
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp     @B
@@:
        vpxor   xmm0, xmm0, xmm2
        vpxor   xmm1, xmm1, xmm3
        vpxor   xmm2, xmm2, xmmword ptr [rcx]
        vpxor   xmm3, xmm3, xmmword ptr [rcx+10H]
        vmovdqu xmmword ptr [r10], xmm0
        vmovdqu xmmword ptr [r10+10H], xmm1
        vmovdqu xmmword ptr [r10+20H], xmm2
        vmovdqu xmmword ptr [r10+30H], xmm3
        vmovdqa xmm6, xmmword ptr [rsp]
        vmovdqa xmm7, xmmword ptr [rsp+10H]
        vmovdqa xmm8, xmmword ptr [rsp+20H]
        vmovdqa xmm9, xmmword ptr [rsp+30H]
        add     rsp, 72
        ret
_blake3_compress_xof_avx512 ENDP
blake3_compress_xof_avx512 ENDP


ALIGN 16
blake3_xof_many_avx512 PROC
_blake3_xof_many_avx512 PROC
        mov rax, qword ptr [rsp+38h]
        cmp rax, 1h
        jnbe slowpath
        sub rsp, 48h
        movaps xmmword ptr [rsp], xmm6
        movaps xmmword ptr [rsp+10h], xmm7
        movaps xmmword ptr [rsp+20h], xmm8
        movaps xmmword ptr [rsp+30h], xmm9
        vmovdqu xmm0, xmmword ptr [rcx]
        vmovdqu xmm1, xmmword ptr [rcx+10h]
        movzx r8d, r8b
        movzx r10d, byte ptr [rsp+70h]
        shl r10, 20h
        or r8, r10
        vmovq xmm2, r8
        vmovq xmm3, r9
        vpunpcklqdq xmm3, xmm3, xmm2
        vmovaps xmm2, xmmword ptr [BLAKE3_IV]
        vmovdqu xmm8, xmmword ptr [rdx]
        vmovdqu xmm9, xmmword ptr [rdx+10h]
        vshufps xmm4, xmm8, xmm9, 88h
        vshufps xmm5, xmm8, xmm9, 0DDh
        vmovdqu xmm8, xmmword ptr [rdx+20h]
        vmovdqu xmm9, xmmword ptr [rdx+30h]
        vshufps xmm6, xmm8, xmm9, 88h
        vshufps xmm7, xmm8, xmm9, 0DDh
        vpshufd xmm6, xmm6, 93h
        vpshufd xmm7, xmm7, 93h
        mov r8b, 7h
@@:
        vpaddd xmm0, xmm0, xmm4
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 10h
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0Ch
        vpaddd xmm0, xmm0, xmm5
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 8h
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 7h
        vpshufd xmm0, xmm0, 93h
        vpshufd xmm3, xmm3, 4Eh
        vpshufd xmm2, xmm2, 39h
        vpaddd xmm0, xmm0, xmm6
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 10h
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 0Ch
        vpaddd xmm0, xmm0, xmm7
        vpaddd xmm0, xmm0, xmm1
        vpxord xmm3, xmm3, xmm0
        vprord xmm3, xmm3, 8h
        vpaddd xmm2, xmm2, xmm3
        vpxord xmm1, xmm1, xmm2
        vprord xmm1, xmm1, 7h
        vpshufd xmm0, xmm0, 39h
        vpshufd xmm3, xmm3, 4Eh
        vpshufd xmm2, xmm2, 93h
        dec r8b
        jz @F
        vshufps xmm8, xmm4, xmm5, 0D6h
        vpshufd xmm9, xmm4, 0Fh
        vpshufd xmm4, xmm8, 39h
        vshufps xmm8, xmm6, xmm7, 0FAh
        vpblendd xmm9, xmm9, xmm8, 0AAh
        vpunpcklqdq xmm8, xmm7, xmm5
        vpblendd xmm8, xmm8, xmm6, 88h
        vpshufd xmm8, xmm8, 78h
        vpunpckhdq xmm5, xmm5, xmm7
        vpunpckldq xmm6, xmm6, xmm5
        vpshufd xmm7, xmm6, 1Eh
        vmovdqa xmm5, xmm9
        vmovdqa xmm6, xmm8
        jmp @B
@@:
        mov r8, qword ptr [rsp+78h]
        vpxor xmm0, xmm0, xmm2
        vpxor xmm1, xmm1, xmm3
        vpxor xmm2, xmm2, xmmword ptr [rcx]
        vpxor xmm3, xmm3, xmmword ptr [rcx+10h]
        vmovdqu xmmword ptr [r8], xmm0
        vmovdqu xmmword ptr [r8+10h], xmm1
        vmovdqu xmmword ptr [r8+20h], xmm2
        vmovdqu xmmword ptr [r8+30h], xmm3
        vzeroupper
        movaps xmm6, xmmword ptr [rsp]
        movaps xmm7, xmmword ptr [rsp+10h]
        movaps xmm8, xmmword ptr [rsp+20h]
        movaps xmm9, xmmword ptr [rsp+30h]
        add rsp, 48h
        ret
slowpath:
        push rbp
        mov rbp, rsp
        sub rsp, 1A0h
        movaps xmmword ptr [rbp-0A0h], xmm6
        movaps xmmword ptr [rbp-90h], xmm7
        movaps xmmword ptr [rbp-80h], xmm8
        movaps xmmword ptr [rbp-70h], xmm9
        movaps xmmword ptr [rbp-60h], xmm10
        movaps xmmword ptr [rbp-50h], xmm11
        movaps xmmword ptr [rbp-40h], xmm12
        movaps xmmword ptr [rbp-30h], xmm13
        movaps xmmword ptr [rbp-20h], xmm14
        movaps xmmword ptr [rbp-10h], xmm15
        and rsp, -40h
        vpbroadcastd zmm0, r9d
        shr r9, 20h
        vpbroadcastd zmm1, r9d
        vpaddd zmm2, zmm0, zmmword ptr [ADD0]
        vpcmpud k1, zmm2, zmm0, 1h
        vpaddd zmm1 {k1}, zmm1, dword bcst [ADD1]
        vmovdqa32 zmmword ptr [rsp], zmm2
        vmovdqa32 zmmword ptr [rsp+40h], zmm1
        mov r9, qword ptr [rbp+38h]
        movzx r8d, r8b
        movzx r10d, byte ptr [rbp+30h]
        cmp rax, 8h
        jbe final8blocks
ALIGN 16
innerloop16:
        vpbroadcastd zmm0, dword ptr [rdx]
        vpbroadcastd zmm1, dword ptr [rdx+4h]
        vpbroadcastd zmm2, dword ptr [rdx+8h]
        vpbroadcastd zmm3, dword ptr [rdx+0Ch]
        vpbroadcastd zmm4, dword ptr [rdx+10h]
        vpbroadcastd zmm5, dword ptr [rdx+14h]
        vpbroadcastd zmm6, dword ptr [rdx+18h]
        vpbroadcastd zmm7, dword ptr [rdx+1Ch]
        vpbroadcastd zmm8, dword ptr [rdx+20h]
        vpbroadcastd zmm9, dword ptr [rdx+24h]
        vpbroadcastd zmm10, dword ptr [rdx+28h]
        vpbroadcastd zmm11, dword ptr [rdx+2Ch]
        vpbroadcastd zmm12, dword ptr [rdx+30h]
        vpbroadcastd zmm13, dword ptr [rdx+34h]
        vpbroadcastd zmm14, dword ptr [rdx+38h]
        vpbroadcastd zmm15, dword ptr [rdx+3Ch]
        vpbroadcastd zmm16, dword ptr [rcx]
        vpbroadcastd zmm17, dword ptr [rcx+4h]
        vpbroadcastd zmm18, dword ptr [rcx+8h]
        vpbroadcastd zmm19, dword ptr [rcx+0Ch]
        vpbroadcastd zmm20, dword ptr [rcx+10h]
        vpbroadcastd zmm21, dword ptr [rcx+14h]
        vpbroadcastd zmm22, dword ptr [rcx+18h]
        vpbroadcastd zmm23, dword ptr [rcx+1Ch]
        vpbroadcastd zmm24, dword ptr [BLAKE3_IV_0]
        vpbroadcastd zmm25, dword ptr [BLAKE3_IV_1]
        vpbroadcastd zmm26, dword ptr [BLAKE3_IV_2]
        vpbroadcastd zmm27, dword ptr [BLAKE3_IV_3]
        vmovdqa32 zmm28, zmmword ptr [rsp]
        vmovdqa32 zmm29, zmmword ptr [rsp+40h]
        vpbroadcastd zmm30, r8d
        vpbroadcastd zmm31, r10d
        mov r11b, 7h
@@:
        vpaddd zmm16, zmm16, zmm0
        vpaddd zmm17, zmm17, zmm2
        vpaddd zmm18, zmm18, zmm4
        vpaddd zmm19, zmm19, zmm6
        vmovdqa32 zmmword ptr [rsp+80h], zmm0
        vmovdqa32 zmmword ptr [rsp+0C0h], zmm6
        vpaddd zmm16, zmm16, zmm20
        vpaddd zmm17, zmm17, zmm21
        vpaddd zmm18, zmm18, zmm22
        vpaddd zmm19, zmm19, zmm23
        vpxord zmm28, zmm28, zmm16
        vmovdqa32 zmm0, zmm2
        vpxord zmm29, zmm29, zmm17
        vpxord zmm30, zmm30, zmm18
        vpxord zmm31, zmm31, zmm19
        vprord zmm28, zmm28, 10h
        vprord zmm29, zmm29, 10h
        vprord zmm30, zmm30, 10h
        vprord zmm31, zmm31, 10h
        vpaddd zmm24, zmm24, zmm28
        vmovdqa32 zmm6, zmm4
        vpaddd zmm25, zmm25, zmm29
        vpaddd zmm26, zmm26, zmm30
        vpaddd zmm27, zmm27, zmm31
        vpxord zmm20, zmm20, zmm24
        vpxord zmm21, zmm21, zmm25
        vpxord zmm22, zmm22, zmm26
        vpxord zmm23, zmm23, zmm27
        vprord zmm20, zmm20, 0Ch
        vprord zmm21, zmm21, 0Ch
        vprord zmm22, zmm22, 0Ch
        vprord zmm23, zmm23, 0Ch
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
        vprord zmm28, zmm28, 8h
        vmovdqa32 zmm4, zmm7
        vprord zmm29, zmm29, 8h
        vprord zmm30, zmm30, 8h
        vprord zmm31, zmm31, 8h
        vpaddd zmm24, zmm24, zmm28
        vpaddd zmm25, zmm25, zmm29
        vpaddd zmm26, zmm26, zmm30
        vpaddd zmm27, zmm27, zmm31
        vpxord zmm20, zmm20, zmm24
        vmovdqa32 zmm3, zmm10
        vpxord zmm21, zmm21, zmm25
        vpxord zmm22, zmm22, zmm26
        vpxord zmm23, zmm23, zmm27
        vprord zmm20, zmm20, 7h
        vprord zmm21, zmm21, 7h
        vprord zmm22, zmm22, 7h
        vprord zmm23, zmm23, 7h
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
        vprord zmm31, zmm31, 10h
        vprord zmm28, zmm28, 10h
        vprord zmm29, zmm29, 10h
        vprord zmm30, zmm30, 10h
        vpaddd zmm26, zmm26, zmm31
        vmovdqa32 zmm12, zmm9
        vpaddd zmm27, zmm27, zmm28
        vpaddd zmm24, zmm24, zmm29
        vpaddd zmm25, zmm25, zmm30
        vpxord zmm21, zmm21, zmm26
        vpxord zmm22, zmm22, zmm27
        vpxord zmm23, zmm23, zmm24
        vpxord zmm20, zmm20, zmm25
        vprord zmm21, zmm21, 0Ch
        vprord zmm22, zmm22, 0Ch
        vprord zmm23, zmm23, 0Ch
        vprord zmm20, zmm20, 0Ch
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
        vprord zmm31, zmm31, 8h
        vmovdqa32 zmm11, zmm5
        vprord zmm28, zmm28, 8h
        vprord zmm29, zmm29, 8h
        vprord zmm30, zmm30, 8h
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
        vmovdqa32 zmm5, zmmword ptr [rsp+80h]
        vprord zmm21, zmm21, 7h
        vprord zmm22, zmm22, 7h
        vprord zmm23, zmm23, 7h
        vprord zmm20, zmm20, 7h
        vmovdqa32 zmm1, zmmword ptr [rsp+0C0h]
        dec r11b
        jnz @B
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
        vmovdqa32 zmm8, zmmword ptr [INDEX0]
        vmovdqa32 zmm9, zmmword ptr [INDEX1]
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
        vextracti64x4 ymmword ptr [r9], zmm0, 0h
        vextracti64x4 ymmword ptr [r9+20h], zmm1, 0h
        vextracti64x4 ymmword ptr [r9+40h], zmm2, 0h
        vextracti64x4 ymmword ptr [r9+60h], zmm3, 0h
        vextracti64x4 ymmword ptr [r9+80h], zmm4, 0h
        vextracti64x4 ymmword ptr [r9+0A0h], zmm5, 0h
        vextracti64x4 ymmword ptr [r9+0C0h], zmm6, 0h
        vextracti64x4 ymmword ptr [r9+0E0h], zmm7, 0h
        vextracti64x4 ymmword ptr [r9+100h], zmm16, 0h
        vextracti64x4 ymmword ptr [r9+120h], zmm24, 0h
        vextracti64x4 ymmword ptr [r9+140h], zmm17, 0h
        vextracti64x4 ymmword ptr [r9+160h], zmm25, 0h
        vextracti64x4 ymmword ptr [r9+180h], zmm18, 0h
        vextracti64x4 ymmword ptr [r9+1A0h], zmm26, 0h
        vextracti64x4 ymmword ptr [r9+1C0h], zmm19, 0h
        vextracti64x4 ymmword ptr [r9+1E0h], zmm27, 0h
        vextracti64x4 ymmword ptr [r9+200h], zmm0, 1h
        vextracti64x4 ymmword ptr [r9+220h], zmm1, 1h
        cmp rax, 0Ah
        jb unwind
        vextracti64x4 ymmword ptr [r9+240h], zmm2, 1h
        vextracti64x4 ymmword ptr [r9+260h], zmm3, 1h
        cmp rax, 0Bh
        jb unwind
        vextracti64x4 ymmword ptr [r9+280h], zmm4, 1h
        vextracti64x4 ymmword ptr [r9+2A0h], zmm5, 1h
        cmp rax, 0Ch
        jb unwind
        vextracti64x4 ymmword ptr [r9+2C0h], zmm6, 1h
        vextracti64x4 ymmword ptr [r9+2E0h], zmm7, 1h
        cmp rax, 0Dh
        jb unwind
        vextracti64x4 ymmword ptr [r9+300h], zmm16, 1h
        vextracti64x4 ymmword ptr [r9+320h], zmm24, 1h
        cmp rax, 0Eh
        jb unwind
        vextracti64x4 ymmword ptr [r9+340h], zmm17, 1h
        vextracti64x4 ymmword ptr [r9+360h], zmm25, 1h
        cmp rax, 0Fh
        jb unwind
        vextracti64x4 ymmword ptr [r9+380h], zmm18, 1h
        vextracti64x4 ymmword ptr [r9+3A0h], zmm26, 1h
        cmp rax, 10h
        jb unwind
        vextracti64x4 ymmword ptr [r9+3C0h], zmm19, 1h
        vextracti64x4 ymmword ptr [r9+3E0h], zmm27, 1h
        vmovdqa32 zmm0, zmmword ptr [rsp]
        vmovdqa32 zmm1, zmmword ptr [rsp+40h]
        vpaddd zmm2, zmm0, dword bcst [ADD16]
        vpcmpud k1, zmm2, zmm0, 1h
        vpaddd zmm1 {k1}, zmm1, dword bcst [ADD1]
        vmovdqa32 zmmword ptr [rsp], zmm2
        vmovdqa32 zmmword ptr [rsp+40h], zmm1
        add r9, 400h
        sub rax, 10h
        cmp rax, 8h
        jnbe innerloop16
        test al, al
        jnz final8blocks
unwind:
        vzeroupper
        movaps xmm6, xmmword ptr [rbp-0A0h]
        movaps xmm7, xmmword ptr [rbp-90h]
        movaps xmm8, xmmword ptr [rbp-80h]
        movaps xmm9, xmmword ptr [rbp-70h]
        movaps xmm10, xmmword ptr [rbp-60h]
        movaps xmm11, xmmword ptr [rbp-50h]
        movaps xmm12, xmmword ptr [rbp-40h]
        movaps xmm13, xmmword ptr [rbp-30h]
        movaps xmm14, xmmword ptr [rbp-20h]
        movaps xmm15, xmmword ptr [rbp-10h]
        mov rsp, rbp
        pop rbp
        ret
final8blocks:
        cmp al, 4h
        jbe final4blocks
        vpbroadcastd ymm16, dword ptr [rdx]
        vpbroadcastd ymm17, dword ptr [rdx+4h]
        vpbroadcastd ymm18, dword ptr [rdx+8h]
        vpbroadcastd ymm19, dword ptr [rdx+0Ch]
        vpbroadcastd ymm20, dword ptr [rdx+10h]
        vpbroadcastd ymm21, dword ptr [rdx+14h]
        vpbroadcastd ymm22, dword ptr [rdx+18h]
        vpbroadcastd ymm23, dword ptr [rdx+1Ch]
        vpbroadcastd ymm24, dword ptr [rdx+20h]
        vpbroadcastd ymm25, dword ptr [rdx+24h]
        vpbroadcastd ymm26, dword ptr [rdx+28h]
        vpbroadcastd ymm27, dword ptr [rdx+2Ch]
        vpbroadcastd ymm28, dword ptr [rdx+30h]
        vpbroadcastd ymm29, dword ptr [rdx+34h]
        vpbroadcastd ymm30, dword ptr [rdx+38h]
        vpbroadcastd ymm31, dword ptr [rdx+3Ch]
        vpbroadcastd ymm0, dword ptr [rcx]
        vpbroadcastd ymm1, dword ptr [rcx+4h]
        vpbroadcastd ymm2, dword ptr [rcx+8h]
        vpbroadcastd ymm3, dword ptr [rcx+0Ch]
        vpbroadcastd ymm4, dword ptr [rcx+10h]
        vpbroadcastd ymm5, dword ptr [rcx+14h]
        vpbroadcastd ymm6, dword ptr [rcx+18h]
        vpbroadcastd ymm7, dword ptr [rcx+1Ch]
        vpbroadcastd ymm8, dword ptr [BLAKE3_IV_0]
        vpbroadcastd ymm9, dword ptr [BLAKE3_IV_1]
        vpbroadcastd ymm10, dword ptr [BLAKE3_IV_2]
        vpbroadcastd ymm11, dword ptr [BLAKE3_IV_3]
        vmovdqa ymm12, ymmword ptr [rsp]
        vmovdqa ymm13, ymmword ptr [rsp+40h]
        vpbroadcastd ymm14, r8d
        vpbroadcastd ymm15, r10d
        mov r11b, 7h
@@:
        vpaddd ymm0, ymm0, ymm16
        vpaddd ymm1, ymm1, ymm18
        vpaddd ymm2, ymm2, ymm20
        vpaddd ymm3, ymm3, ymm22
        vmovdqa32 ymmword ptr [rsp+80h], ymm16
        vmovdqa32 ymmword ptr [rsp+0C0h], ymm22
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm1, ymm1, ymm5
        vpaddd ymm2, ymm2, ymm6
        vpaddd ymm3, ymm3, ymm7
        vpxord ymm12, ymm12, ymm0
        vmovdqa32 ymm16, ymm18
        vpxord ymm13, ymm13, ymm1
        vpxord ymm14, ymm14, ymm2
        vpxord ymm15, ymm15, ymm3
        vprord ymm12, ymm12, 10h
        vprord ymm13, ymm13, 10h
        vprord ymm14, ymm14, 10h
        vprord ymm15, ymm15, 10h
        vpaddd ymm8, ymm8, ymm12
        vmovdqa32 ymm22, ymm20
        vpaddd ymm9, ymm9, ymm13
        vpaddd ymm10, ymm10, ymm14
        vpaddd ymm11, ymm11, ymm15
        vpxord ymm4, ymm4, ymm8
        vpxord ymm5, ymm5, ymm9
        vpxord ymm6, ymm6, ymm10
        vpxord ymm7, ymm7, ymm11
        vprord ymm4, ymm4, 0Ch
        vprord ymm5, ymm5, 0Ch
        vprord ymm6, ymm6, 0Ch
        vprord ymm7, ymm7, 0Ch
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
        vprord ymm12, ymm12, 8h
        vmovdqa32 ymm20, ymm23
        vprord ymm13, ymm13, 8h
        vprord ymm14, ymm14, 8h
        vprord ymm15, ymm15, 8h
        vpaddd ymm8, ymm8, ymm12
        vpaddd ymm9, ymm9, ymm13
        vpaddd ymm10, ymm10, ymm14
        vpaddd ymm11, ymm11, ymm15
        vpxord ymm4, ymm4, ymm8
        vmovdqa32 ymm19, ymm26
        vpxord ymm5, ymm5, ymm9
        vpxord ymm6, ymm6, ymm10
        vpxord ymm7, ymm7, ymm11
        vprord ymm4, ymm4, 7h
        vprord ymm5, ymm5, 7h
        vprord ymm6, ymm6, 7h
        vprord ymm7, ymm7, 7h
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
        vprord ymm15, ymm15, 10h
        vprord ymm12, ymm12, 10h
        vprord ymm13, ymm13, 10h
        vprord ymm14, ymm14, 10h
        vpaddd ymm10, ymm10, ymm15
        vmovdqa32 ymm28, ymm25
        vpaddd ymm11, ymm11, ymm12
        vpaddd ymm8, ymm8, ymm13
        vpaddd ymm9, ymm9, ymm14
        vpxord ymm5, ymm5, ymm10
        vpxord ymm6, ymm6, ymm11
        vpxord ymm7, ymm7, ymm8
        vpxord ymm4, ymm4, ymm9
        vprord ymm5, ymm5, 0Ch
        vprord ymm6, ymm6, 0Ch
        vprord ymm7, ymm7, 0Ch
        vprord ymm4, ymm4, 0Ch
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
        vprord ymm15, ymm15, 8h
        vmovdqa32 ymm27, ymm21
        vprord ymm12, ymm12, 8h
        vprord ymm13, ymm13, 8h
        vprord ymm14, ymm14, 8h
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
        vmovdqa32 ymm21, ymmword ptr [rsp+80h]
        vprord ymm5, ymm5, 7h
        vprord ymm6, ymm6, 7h
        vprord ymm7, ymm7, 7h
        vprord ymm4, ymm4, 7h
        vmovdqa32 ymm17, ymmword ptr [rsp+0C0h]
        dec r11b
        jnz @B
        vpxord ymm0, ymm0, ymm8
        vpxord ymm8, ymm8, dword bcst [rcx]
        vpxord ymm1, ymm1, ymm9
        vpxord ymm9, ymm9, dword bcst [rcx+4h]
        vpxord ymm2, ymm2, ymm10
        vpxord ymm10, ymm10, dword bcst [rcx+8h]
        vpxord ymm3, ymm3, ymm11
        vpxord ymm11, ymm11, dword bcst [rcx+0Ch]
        vpxord ymm4, ymm4, ymm12
        vpxord ymm12, ymm12, dword bcst [rcx+10h]
        vpxord ymm5, ymm5, ymm13
        vpxord ymm13, ymm13, dword bcst [rcx+14h]
        vpxord ymm6, ymm6, ymm14
        vpxord ymm14, ymm14, dword bcst [rcx+18h]
        vpxord ymm7, ymm7, ymm15
        vpxord ymm15, ymm15, dword bcst [rcx+1Ch]
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
        vshufi32x4 ymm16, ymm0, ymm4, 0h
        vshufi32x4 ymm17, ymm8, ymm12, 0h
        vshufi32x4 ymm18, ymm1, ymm5, 0h
        vshufi32x4 ymm19, ymm9, ymm13, 0h
        vshufi32x4 ymm20, ymm2, ymm6, 0h
        vshufi32x4 ymm21, ymm10, ymm14, 0h
        vshufi32x4 ymm22, ymm3, ymm7, 0h
        vshufi32x4 ymm23, ymm11, ymm15, 0h
        vshufi32x4 ymm24, ymm0, ymm4, 3h
        vshufi32x4 ymm25, ymm8, ymm12, 3h
        vshufi32x4 ymm26, ymm1, ymm5, 3h
        vshufi32x4 ymm27, ymm9, ymm13, 3h
        vshufi32x4 ymm28, ymm2, ymm6, 3h
        vshufi32x4 ymm29, ymm10, ymm14, 3h
        vshufi32x4 ymm30, ymm3, ymm7, 3h
        vshufi32x4 ymm31, ymm11, ymm15, 3h
        vmovdqu32 ymmword ptr [r9], ymm16
        vmovdqu32 ymmword ptr [r9+20h], ymm17
        vmovdqu32 ymmword ptr [r9+40h], ymm18
        vmovdqu32 ymmword ptr [r9+60h], ymm19
        vmovdqu32 ymmword ptr [r9+80h], ymm20
        vmovdqu32 ymmword ptr [r9+0A0h], ymm21
        vmovdqu32 ymmword ptr [r9+0C0h], ymm22
        vmovdqu32 ymmword ptr [r9+0E0h], ymm23
        vmovdqu32 ymmword ptr [r9+100h], ymm24
        vmovdqu32 ymmword ptr [r9+120h], ymm25
        cmp al, 6h
        jb @F
        vmovdqu32 ymmword ptr [r9+140h], ymm26
        vmovdqu32 ymmword ptr [r9+160h], ymm27
        cmp al, 7h
        jb @F
        vmovdqu32 ymmword ptr [r9+180h], ymm28
        vmovdqu32 ymmword ptr [r9+1A0h], ymm29
        cmp al, 8h
        jb @F
        vmovdqu32 ymmword ptr [r9+1C0h], ymm30
        vmovdqu32 ymmword ptr [r9+1E0h], ymm31
@@:
        jmp unwind
final4blocks:
        mov r11d, 0AAAAh
        kmovw k1, r11d
        mov r11d, 8888h
        kmovw k2, r11d
        mov r11d, 55h
        kmovw k3, r11d
        mov r11d, r10d
        shl r11, 20h
        or r11, r8
        cmp al, 2h
        jbe final2blocks
        vbroadcasti32x4 zmm0, xmmword ptr [rcx]
        vbroadcasti32x4 zmm1, xmmword ptr [rcx+10h]
        vbroadcasti32x4 zmm2, xmmword ptr [BLAKE3_IV]
        vmovdqa32 xmm4, xmmword ptr [rsp]
        vmovdqa32 xmm5, xmmword ptr [rsp+40h]
        vpbroadcastq zmm3, r11
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti64x4 zmm6, zmm6, ymm5, 1h
        vpermq zmm3 {k3}, zmm6, 0DCh
        vbroadcasti32x4 zmm8, xmmword ptr [rdx]
        vbroadcasti32x4 zmm9, xmmword ptr [rdx+10h]
        vshufps zmm4, zmm8, zmm9, 88h
        vshufps zmm5, zmm8, zmm9, 0DDh
        vbroadcasti32x4 zmm8, xmmword ptr [rdx+20h]
        vbroadcasti32x4 zmm9, xmmword ptr [rdx+30h]
        vshufps zmm6, zmm8, zmm9, 88h
        vshufps zmm7, zmm8, zmm9, 0DDh
        vpshufd zmm6, zmm6, 93h
        vpshufd zmm7, zmm7, 93h
        mov r8b, 7h
@@:
        vpaddd zmm0, zmm0, zmm4
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 10h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0Ch
        vpaddd zmm0, zmm0, zmm5
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 8h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 7h
        vpshufd zmm0, zmm0, 93h
        vpshufd zmm3, zmm3, 4Eh
        vpshufd zmm2, zmm2, 39h
        vpaddd zmm0, zmm0, zmm6
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 10h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 0Ch
        vpaddd zmm0, zmm0, zmm7
        vpaddd zmm0, zmm0, zmm1
        vpxord zmm3, zmm3, zmm0
        vprord zmm3, zmm3, 8h
        vpaddd zmm2, zmm2, zmm3
        vpxord zmm1, zmm1, zmm2
        vprord zmm1, zmm1, 7h
        vpshufd zmm0, zmm0, 39h
        vpshufd zmm3, zmm3, 4Eh
        vpshufd zmm2, zmm2, 93h
        dec r8b
        jz @F
        vshufps zmm8, zmm4, zmm5, 0D6h
        vpshufd zmm9, zmm4, 0Fh
        vpshufd zmm4, zmm8, 39h
        vshufps zmm8, zmm6, zmm7, 0FAh
        vpblendmd zmm9 {k1}, zmm9, zmm8
        vpunpcklqdq zmm8, zmm7, zmm5
        vpblendmd zmm8 {k2}, zmm8, zmm6
        vpshufd zmm8, zmm8, 78h
        vpunpckhdq zmm5, zmm5, zmm7
        vpunpckldq zmm6, zmm6, zmm5
        vpshufd zmm7, zmm6, 1Eh
        vmovdqa32 zmm5, zmm9
        vmovdqa32 zmm6, zmm8
        jmp @B
@@:
        vpxord zmm0, zmm0, zmm2
        vpxord zmm1, zmm1, zmm3
        vbroadcasti32x4 zmm4, xmmword ptr [rcx]
        vbroadcasti32x4 zmm5, xmmword ptr [rcx+10h]
        vpxord zmm2, zmm2, zmm4
        vpxord zmm3, zmm3, zmm5
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+10h], xmm1
        vmovdqu xmmword ptr [r9+20h], xmm2
        vmovdqu xmmword ptr [r9+30h], xmm3
        vextracti128 xmmword ptr [r9+40h], ymm0, 1h
        vextracti128 xmmword ptr [r9+50h], ymm1, 1h
        vextracti128 xmmword ptr [r9+60h], ymm2, 1h
        vextracti128 xmmword ptr [r9+70h], ymm3, 1h
        vextracti32x4 xmmword ptr [r9+80h], zmm0, 2h
        vextracti32x4 xmmword ptr [r9+90h], zmm1, 2h
        vextracti32x4 xmmword ptr [r9+0A0h], zmm2, 2h
        vextracti32x4 xmmword ptr [r9+0B0h], zmm3, 2h
        cmp al, 4h
        jb @F
        vextracti32x4 xmmword ptr [r9+0C0h], zmm0, 3h
        vextracti32x4 xmmword ptr [r9+0D0h], zmm1, 3h
        vextracti32x4 xmmword ptr [r9+0E0h], zmm2, 3h
        vextracti32x4 xmmword ptr [r9+0F0h], zmm3, 3h
@@:
        jmp unwind
final2blocks:
        test al, al
        jz unwind
        vbroadcasti32x4 ymm0, xmmword ptr [rcx]
        vbroadcasti32x4 ymm1, xmmword ptr [rcx+10h]
        vbroadcasti32x4 ymm2, xmmword ptr [BLAKE3_IV]
        vmovdqa xmm4, xmmword ptr [rsp]
        vmovdqa xmm5, xmmword ptr [rsp+40h]
        vpbroadcastq ymm3, r11
        vpunpckldq xmm6, xmm4, xmm5
        vpunpckhdq xmm5, xmm4, xmm5
        vinserti128 ymm6, ymm6, xmm5, 1h
        vpermq ymm3 {k3}, ymm6, 0DCh
        vbroadcasti32x4 ymm8, xmmword ptr [rdx]
        vbroadcasti32x4 ymm9, xmmword ptr [rdx+10h]
        vshufps ymm4, ymm8, ymm9, 88h
        vshufps ymm5, ymm8, ymm9, 0DDh
        vbroadcasti32x4 ymm8, xmmword ptr [rdx+20h]
        vbroadcasti32x4 ymm9, xmmword ptr [rdx+30h]
        vshufps ymm6, ymm8, ymm9, 88h
        vshufps ymm7, ymm8, ymm9, 0DDh
        vpshufd ymm6, ymm6, 93h
        vpshufd ymm7, ymm7, 93h
        mov r8b, 7h
@@:
        vpaddd ymm0, ymm0, ymm4
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 10h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0Ch
        vpaddd ymm0, ymm0, ymm5
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 8h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 7h
        vpshufd ymm0, ymm0, 93h
        vpshufd ymm3, ymm3, 4Eh
        vpshufd ymm2, ymm2, 39h
        vpaddd ymm0, ymm0, ymm6
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 10h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 0Ch
        vpaddd ymm0, ymm0, ymm7
        vpaddd ymm0, ymm0, ymm1
        vpxord ymm3, ymm3, ymm0
        vprord ymm3, ymm3, 8h
        vpaddd ymm2, ymm2, ymm3
        vpxord ymm1, ymm1, ymm2
        vprord ymm1, ymm1, 7h
        vpshufd ymm0, ymm0, 39h
        vpshufd ymm3, ymm3, 4Eh
        vpshufd ymm2, ymm2, 93h
        dec r8b
        jz @F
        vshufps ymm8, ymm4, ymm5, 0D6h
        vpshufd ymm9, ymm4, 0Fh
        vpshufd ymm4, ymm8, 39h
        vshufps ymm8, ymm6, ymm7, 0FAh
        vpblendd ymm9, ymm9, ymm8, 0AAh
        vpunpcklqdq ymm8, ymm7, ymm5
        vpblendd ymm8, ymm8, ymm6, 88h
        vpshufd ymm8, ymm8, 78h
        vpunpckhdq ymm5, ymm5, ymm7
        vpunpckldq ymm6, ymm6, ymm5
        vpshufd ymm7, ymm6, 1Eh
        vmovdqa ymm5, ymm9
        vmovdqa ymm6, ymm8
        jmp @B
@@:
        vpxord ymm0, ymm0, ymm2
        vpxord ymm1, ymm1, ymm3
        vbroadcasti128 ymm4, xmmword ptr [rcx]
        vbroadcasti128 ymm5, xmmword ptr [rcx+10h]
        vpxord ymm2, ymm2, ymm4
        vpxord ymm3, ymm3, ymm5
        vmovdqu xmmword ptr [r9], xmm0
        vmovdqu xmmword ptr [r9+10h], xmm1
        vmovdqu xmmword ptr [r9+20h], xmm2
        vmovdqu xmmword ptr [r9+30h], xmm3
        cmp al, 2h
        jb @F
        vextracti128 xmmword ptr [r9+40h], ymm0, 1h
        vextracti128 xmmword ptr [r9+50h], ymm1, 1h
        vextracti128 xmmword ptr [r9+60h], ymm2, 1h
        vextracti128 xmmword ptr [r9+70h], ymm3, 1h
@@:
        jmp unwind
_blake3_xof_many_avx512 ENDP
blake3_xof_many_avx512 ENDP

_TEXT ENDS

_RDATA SEGMENT READONLY PAGE ALIAS(".rdata") 'CONST'
ALIGN   64
ADD0:
        dd    0,  1,  2,  3,  4,  5,  6,  7
        dd    8,  9, 10, 11, 12, 13, 14, 15
INDEX0:
        dd    0,  1,  2,  3, 16, 17, 18, 19
        dd    8,  9, 10, 11, 24, 25, 26, 27
INDEX1:
        dd    4,  5,  6,  7, 20, 21, 22, 23
        dd   12, 13, 14, 15, 28, 29, 30, 31
BLAKE3_IV:
BLAKE3_IV_0:
        dd   06A09E667H
BLAKE3_IV_1:
        dd   0BB67AE85H
BLAKE3_IV_2:
        dd   03C6EF372H
BLAKE3_IV_3:
        dd   0A54FF53AH
ADD1:   
        dd    1
ADD16:  
        dd   16
BLAKE3_BLOCK_LEN:
        dd   64

_RDATA ENDS
END
