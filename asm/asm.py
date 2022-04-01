#! /usr/bin/env python3

# Generate asm!
#
# TODOs:
# - vzeroupper
# - CET
# - prefetches

from dataclasses import dataclass, replace

X86_64 = "x86_64"
AVX512 = "avx512"
AVX2 = "avx2"
SSE41 = "sse41"
SSE2 = "sse2"
LINUX = "linux"

MESSAGE_SCHEDULE = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8],
    [3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1],
    [10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6],
    [12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4],
    [9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7],
    [11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13],
]


@dataclass
class Target:
    arch: str
    extension: str
    os: str

    def arg32(self, index):
        system_v_args_32 = ["edi", "esi", "edx", "ecx", "r8d", "r9d"]
        return system_v_args_32[index]

    def arg64(self, index):
        system_v_args_64 = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"]
        return system_v_args_64[index]

    def scratch32(self, index):
        system_v_scratch_32 = ["eax", "r10d", "r11d"]
        return system_v_scratch_32[index]

    def scratch64(self, index):
        system_v_scratch_64 = ["rax", "r10", "r11"]
        return system_v_scratch_64[index]

    def reg128(self, index):
        assert self.arch == X86_64
        return "xmm" + str(index)

    def ret(self):
        return "ret"


def add_row(target, output, degree, dest, src):
    assert target.arch == X86_64
    if target.extension == AVX512:
        if degree == 1:
            output.append(f"vpaddd xmm{dest}, xmm{dest}, xmm{src}")
        elif degree in (2, 8):
            output.append(f"vpaddd ymm{dest}, ymm{dest}, ymm{src}")
        elif degree in (4, 16):
            output.append(f"vpaddd zmm{dest}, zmm{dest}, zmm{src}")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        output.append(f"vpaddd ymm{dest}, ymm{dest}, ymm{src}")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append(f"paddd xmm{dest}, xmm{src}")
    else:
        raise NotImplementedError


def xor_row(target, output, degree, dest, src):
    assert target.arch == X86_64
    if target.extension == AVX512:
        if degree == 1:
            output.append(f"vpxord xmm{dest}, xmm{dest}, xmm{src}")
        elif degree in (2, 8):
            output.append(f"vpxord ymm{dest}, ymm{dest}, ymm{src}")
        elif degree in (4, 16):
            output.append(f"vpxord zmm{dest}, zmm{dest}, zmm{src}")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        output.append(f"vpxord ymm{dest}, ymm{dest}, ymm{src}")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append(f"pxor xmm{dest}, xmm{src}")
    else:
        raise NotImplementedError


# This is the >>> operation in G, not state diagonalization or message permutation.
def bitrotate_row(target, output, degree, reg, bits):
    assert target.arch == X86_64
    if target.extension == AVX512:
        if degree == 1:
            output.append(f"vprord xmm{reg}, xmm{reg}, {bits}")
        elif degree in (2, 8):
            output.append(f"vprord ymm{reg}, ymm{reg}, {bits}")
        elif degree in (4, 16):
            output.append(f"vprord zmm{reg}, zmm{reg}, {bits}")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        if bits == 16:
            # ymm14 is initialized at the start of the kernel
            output.append(f"vpshufb ymm{reg}, ymm{reg}, ymm14")
        elif bits == 8:
            # ymm15 is initialized at the start of the kernel
            output.append(f"vpshufb ymm{reg}, ymm{reg}, ymm15")
        else:
            # Do two bitshifts, using register 8 as temp.
            output.append(f"vpsrld ymm8, ymm{reg}, {bits}")
            output.append(f"vpslld ymm{reg}, ymm{reg}, {32 - bits}")
            output.append(f"vpor ymm{reg}, ymm{reg}, ymm8")
    elif target.extension == SSE41:
        assert degree == 1
        if bits == 16:
            # xmm15 is initialized at the start of the kernel
            output.append(f"pshufb xmm{reg}, xmm15")
        elif bits == 8:
            # xmm14 is initialized at the start of the kernel
            output.append(f"pshufb xmm{reg}, xmm14")
        else:
            # Do two bitshifts, using register 11 as temp.
            output.append(f"movdqa xmm11, xmm{reg}")
            output.append(f"pslld xmm{reg}, {32 - bits}")
            output.append(f"psrld xmm11, {bits}")
            output.append(f"por xmm{reg}, xmm11")
    elif target.extension == SSE2:
        assert degree == 1
        if bits == 16:
            output.append(f"pshuflw xmm{reg}, xmm{reg}, 0xB1")
            output.append(f"pshufhw xmm{reg}, xmm{reg}, 0xB1")
        else:
            # Do two bitshifts, using register 11 as temp.
            output.append(f"movdqa xmm11, xmm{reg}")
            output.append(f"pslld xmm{reg}, {32 - bits}")
            output.append(f"psrld xmm11, {bits}")
            output.append(f"por xmm{reg}, xmm11")
    else:
        raise NotImplementedError


# See the comments above kernel_2d().
def diagonalize_state_rows(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            output.append("vpshufd xmm0, xmm0, 0x93")  # 3 0 1 2
            output.append("vpshufd xmm3, xmm3, 0x4E")  # 2 3 0 1
            output.append("vpshufd xmm2, xmm2, 0x39")  # 1 2 3 0
        elif degree == 2:
            output.append("vpshufd ymm0, ymm0, 0x93")
            output.append("vpshufd ymm3, ymm3, 0x4E")
            output.append("vpshufd ymm2, ymm2, 0x39")
        elif degree == 4:
            output.append("vpshufd zmm0, zmm0, 0x93")
            output.append("vpshufd zmm3, zmm3, 0x4E")
            output.append("vpshufd zmm2, zmm2, 0x39")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        output.append("vpshufd ymm0, ymm0, 0x93")
        output.append("vpshufd ymm3, ymm3, 0x4E")
        output.append("vpshufd ymm2, ymm2, 0x39")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append("pshufd xmm0, xmm0, 0x93")
        output.append("pshufd xmm3, xmm3, 0x4E")
        output.append("pshufd xmm2, xmm2, 0x39")
    else:
        raise NotImplementedError


# See the comments above kernel_2d().
def undiagonalize_state_rows(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            output.append("vpshufd xmm0, xmm0, 0x39")  # 1 2 3 0
            output.append("vpshufd xmm3, xmm3, 0x4E")  # 2 3 0 1
            output.append("vpshufd xmm2, xmm2, 0x93")  # 3 0 1 2
        elif degree == 2:
            output.append("vpshufd ymm0, ymm0, 0x39")
            output.append("vpshufd ymm3, ymm3, 0x4E")
            output.append("vpshufd ymm2, ymm2, 0x93")
        elif degree == 4:
            output.append("vpshufd zmm0, zmm0, 0x39")
            output.append("vpshufd zmm3, zmm3, 0x4E")
            output.append("vpshufd zmm2, zmm2, 0x93")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        output.append("vpshufd ymm0, ymm0, 0x39")
        output.append("vpshufd ymm3, ymm3, 0x4E")
        output.append("vpshufd ymm2, ymm2, 0x93")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append("pshufd xmm0, xmm0, 0x39")
        output.append("pshufd xmm3, xmm3, 0x4E")
        output.append("pshufd xmm2, xmm2, 0x93")
    else:
        raise NotImplementedError


# See the comments above kernel_2d().
def permute_message_rows(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            output.append("vshufps xmm8, xmm4, xmm5, 214")
            output.append("vpshufd xmm9, xmm4, 0x0F")
            output.append("vpshufd xmm4, xmm8, 0x39")
            output.append("vshufps xmm8, xmm6, xmm7, 250")
            output.append("vpblendd xmm9, xmm9, xmm8, 0xAA")
            output.append("vpunpcklqdq xmm8, xmm7, xmm5")
            output.append("vpblendd xmm8, xmm8, xmm6, 0x88")
            output.append("vpshufd xmm8, xmm8, 0x78")
            output.append("vpunpckhdq xmm5, xmm5, xmm7")
            output.append("vpunpckldq xmm6, xmm6, xmm5")
            output.append("vpshufd xmm7, xmm6, 0x1E")
            output.append("vmovdqa xmm5, xmm9")
            output.append("vmovdqa xmm6, xmm8")
        elif degree == 2:
            output.append("vshufps ymm8, ymm4, ymm5, 214")
            output.append("vpshufd ymm9, ymm4, 0x0F")
            output.append("vpshufd ymm4, ymm8, 0x39")
            output.append("vshufps ymm8, ymm6, ymm7, 250")
            output.append("vpblendd ymm9, ymm9, ymm8, 0xAA")
            output.append("vpunpcklqdq ymm8, ymm7, ymm5")
            output.append("vpblendd ymm8, ymm8, ymm6, 0x88")
            output.append("vpshufd ymm8, ymm8, 0x78")
            output.append("vpunpckhdq ymm5, ymm5, ymm7")
            output.append("vpunpckldq ymm6, ymm6, ymm5")
            output.append("vpshufd ymm7, ymm6, 0x1E")
            output.append("vmovdqa ymm5, ymm9")
            output.append("vmovdqa ymm6, ymm8")
        elif degree == 4:
            output.append("vshufps zmm8, zmm4, zmm5, 214")
            output.append("vpshufd zmm9, zmm4, 0x0F")
            output.append("vpshufd zmm4, zmm8, 0x39")
            output.append("vshufps zmm8, zmm6, zmm7, 250")
            # k3 is initialized at the start of the kernel
            output.append("vpblendmd zmm9 {k3}, zmm9, zmm8")
            output.append("vpunpcklqdq zmm8, zmm7, zmm5")
            # k4 is initialized at the start of the kernel
            output.append("vpblendmd zmm8 {k4}, zmm8, zmm6")
            output.append("vpshufd zmm8, zmm8, 0x78")
            output.append("vpunpckhdq zmm5, zmm5, zmm7")
            output.append("vpunpckldq zmm6, zmm6, zmm5")
            output.append("vpshufd zmm7, zmm6, 0x1E")
            output.append("vmovdqa32 zmm5, zmm9")
            output.append("vmovdqa32 zmm6, zmm8")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        output.append("vshufps ymm8, ymm4, ymm5, 214")
        output.append("vpshufd ymm9, ymm4, 0x0F")
        output.append("vpshufd ymm4, ymm8, 0x39")
        output.append("vshufps ymm8, ymm6, ymm7, 250")
        output.append("vpblendd ymm9, ymm9, ymm8, 0xAA")
        output.append("vpunpcklqdq ymm8, ymm7, ymm5")
        output.append("vpblendd ymm8, ymm8, ymm6, 0x88")
        output.append("vpshufd ymm8, ymm8, 0x78")
        output.append("vpunpckhdq ymm5, ymm5, ymm7")
        output.append("vpunpckldq ymm6, ymm6, ymm5")
        output.append("vpshufd ymm7, ymm6, 0x1E")
        output.append("vmovdqa ymm5, ymm9")
        output.append("vmovdqa ymm6, ymm8")
    elif target.extension == SSE41:
        assert degree == 1
        output.append("movdqa xmm8, xmm4")
        output.append("shufps xmm8, xmm5, 214")
        output.append("pshufd xmm9, xmm4, 0x0F")
        output.append("pshufd xmm4, xmm8, 0x39")
        output.append("movdqa xmm8, xmm6")
        output.append("shufps xmm8, xmm7, 250")
        output.append("pblendw xmm9, xmm8, 0xCC")
        output.append("movdqa xmm8, xmm7")
        output.append("punpcklqdq xmm8, xmm5")
        output.append("pblendw xmm8, xmm6, 0xC0")
        output.append("pshufd xmm8, xmm8, 0x78")
        output.append("punpckhdq xmm5, xmm7")
        output.append("punpckldq xmm6, xmm5")
        output.append("pshufd xmm7, xmm6, 0x1E")
        output.append("movdqa xmm5, xmm9")
        output.append("movdqa xmm6, xmm8")
    elif target.extension == SSE2:
        assert degree == 1
        output.append("movdqa xmm8, xmm4")
        output.append("shufps xmm8, xmm5, 214")
        output.append("pshufd xmm9, xmm4, 0x0F")
        output.append("pshufd xmm4, xmm8, 0x39")
        output.append("movdqa xmm8, xmm6")
        output.append("shufps xmm8, xmm7, 250")
        output.append("pand xmm9, xmmword ptr [PBLENDW_0x33_MASK+rip]")
        output.append("pand xmm8, xmmword ptr [PBLENDW_0xCC_MASK+rip]")
        output.append("por xmm9, xmm8")
        output.append("movdqa xmm8, xmm7")
        output.append("punpcklqdq xmm8, xmm5")
        output.append("movdqa xmm10, xmm6")
        output.append("pand xmm8, xmmword ptr [PBLENDW_0x3F_MASK+rip]")
        output.append("pand xmm10, xmmword ptr [PBLENDW_0xC0_MASK+rip]")
        output.append("por xmm8, xmm10")
        output.append("pshufd xmm8, xmm8, 0x78")
        output.append("punpckhdq xmm5, xmm7")
        output.append("punpckldq xmm6, xmm5")
        output.append("pshufd xmm7, xmm6, 0x1E")
        output.append("movdqa xmm5, xmm9")
        output.append("movdqa xmm6, xmm8")
    else:
        raise NotImplementedError


def kernel_2d_name(target, degree):
    return f"blake3_{target.extension}_kernel_2d_{degree}"


# The two-dimensional kernel packs one or more *rows* of the state into a
# vector. For example, the AVX2 version of this kernel computes two inputs in
# parallel, and the caller needs to arrange their extended state words like
# this:
#
#     ymm0:  a0,  a1,  a2,  a3,  b0,  b1,  b2,  b3
#     ymm1:  a4,  a5,  a6,  a7,  b4,  b5,  b6,  b7
#     ymm2:  a8,  a9, a10, a11,  b8,  b9, b10, b11
#     ymm3: a12, a13, a14, a15, b12, b13, b14, b15
#
# In this arrangement, the rows need to be diagonalized and undiagonalized in
# each round. There's an important optimization for this, which applies to all
# ChaCha-derived functions. Intuitively, diagonalization in ChaCha looks like
# this:
#
#  0  1  2  3  ------------ no change ----------->   0  1  2  3
#  4  5  6  7  ---- rotate one position left ---->   5  6  7  4
#  8  9 10 11  ---- rotate two positions left --->  10 11  8  9
# 12 13 14 15  --- rotate three positions left -->  15 12 13 14
#
# However, there's a performance benefit to doing it this way instead:
#
#  0  1  2  3  --- rotate three positions left -->   3  0  1  2
#  4  5  6  7  ------------ no change ----------->   4  5  6  7
#  8  9 10 11  ---- rotate one position left ---->   9 10 11  8
# 12 13 14 15  ---- rotate two positions left --->  14 15 12 13
#
# That is, rather than keeping the first row fixed, keep the second row fixed.
# This gives the same columns, but in a different order. The second row is the
# last one touched by the G function, so leaving it unrotated saves latency.
# For more discussion, see: https://github.com/sneves/blake2-avx2/pull/4
#
# The message words need to be arranged to match the state words. Again using
# AVX2 as an example, the caller needs to initially arrange the message words
# like this:
#
#     ymm4:  a0,  a2,  a4,  a6,  b0,  b2,  b4,  b6
#     ymm5:  a1,  a3,  a5,  a7,  b1,  b3,  b5,  b7
#     ymm6: a14,  a8, a10, a12, b14,  b8, b10, b12
#     ymm7: a15,  a9, a11, a13, b15,  b9, b11, b13
def kernel_2d(target, output, degree):
    label = kernel_2d_name(target, degree)
    output.append(f"{label}:")
    # vpshufb indexes
    if target.extension == SSE2:
        assert degree == 1
    elif target.extension == SSE41:
        assert degree == 1
        output.append(f"movaps xmm14, xmmword ptr [ROT8+rip]")
        output.append(f"movaps xmm15, xmmword ptr [ROT16+rip]")
    elif target.extension == AVX2:
        assert degree == 2
        output.append(f"vbroadcasti128 ymm14, xmmword ptr [ROT16+rip]")
        output.append(f"vbroadcasti128 ymm15, xmmword ptr [ROT8+rip]")
    elif target.extension == AVX512:
        assert degree in (1, 2, 4)
        if degree == 4:
            output.append(f"mov {target.scratch32(0)}, 43690")
            output.append(f"kmovw k3, {target.scratch32(0)}")
            output.append(f"mov {target.scratch32(0)}, 34952")
            output.append(f"kmovw k4, {target.scratch32(0)}")
    else:
        raise NotImplementedError
    for round_number in range(7):
        if round_number > 0:
            # Un-diagonalize and permute before each round except the first.
            # compress_finish() will also partially un-diagonalize.
            permute_message_rows(target, output, degree)
        add_row(target, output, degree, dest=0, src=4)
        add_row(target, output, degree, dest=0, src=1)
        xor_row(target, output, degree, dest=3, src=0)
        bitrotate_row(target, output, degree, reg=3, bits=16)
        add_row(target, output, degree, dest=2, src=3)
        xor_row(target, output, degree, dest=1, src=2)
        bitrotate_row(target, output, degree, reg=1, bits=12)
        add_row(target, output, degree, dest=0, src=5)
        add_row(target, output, degree, dest=0, src=1)
        xor_row(target, output, degree, dest=3, src=0)
        bitrotate_row(target, output, degree, reg=3, bits=8)
        add_row(target, output, degree, dest=2, src=3)
        xor_row(target, output, degree, dest=1, src=2)
        bitrotate_row(target, output, degree, reg=1, bits=7)
        diagonalize_state_rows(target, output, degree)
        add_row(target, output, degree, dest=0, src=6)
        add_row(target, output, degree, dest=0, src=1)
        xor_row(target, output, degree, dest=3, src=0)
        bitrotate_row(target, output, degree, reg=3, bits=16)
        add_row(target, output, degree, dest=2, src=3)
        xor_row(target, output, degree, dest=1, src=2)
        bitrotate_row(target, output, degree, reg=1, bits=12)
        add_row(target, output, degree, dest=0, src=7)
        add_row(target, output, degree, dest=0, src=1)
        xor_row(target, output, degree, dest=3, src=0)
        bitrotate_row(target, output, degree, reg=3, bits=8)
        add_row(target, output, degree, dest=2, src=3)
        xor_row(target, output, degree, dest=1, src=2)
        bitrotate_row(target, output, degree, reg=1, bits=7)
        undiagonalize_state_rows(target, output, degree)
    # Xor the last two rows into the first two, but don't do the feed forward
    # here. That's only done in the XOF case.
    xor_row(target, output, degree, dest=0, src=2)
    xor_row(target, output, degree, dest=1, src=3)
    output.append(target.ret())


def compress_setup(target, output):
    if target.extension == AVX512:
        # state words
        output.append(f"vmovdqu xmm0, xmmword ptr [{target.arg64(0)}]")
        output.append(f"vmovdqu xmm1, xmmword ptr [{target.arg64(0)}+0x10]")
        # flags
        output.append(f"shl {target.arg64(4)}, 32")
        # block length
        output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
        output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
        # counter
        output.append(f"vmovq xmm3, {target.arg64(2)}")
        output.append(f"vmovq xmm4, {target.arg64(3)}")
        output.append(f"vpunpcklqdq xmm3, xmm3, xmm4")
        output.append(f"vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]")
        # message words
        # fmt: off
        output.append(f"vmovups xmm8, xmmword ptr [{target.arg64(1)}]")      # xmm8 = m0 m1 m2 m3
        output.append(f"vmovups xmm9, xmmword ptr [{target.arg64(1)}+0x10]") # xmm9 = m4 m5 m6 m7
        output.append(f"vshufps xmm4, xmm8, xmm9, 136")                      # xmm4 = m0 m2 m4 m6
        output.append(f"vshufps xmm5, xmm8, xmm9, 221")                      # xmm5 = m1 m3 m5 m7
        output.append(f"vmovups xmm8, xmmword ptr [{target.arg64(1)}+0x20]") # xmm8 = m8 m9 m10 m12
        output.append(f"vmovups xmm9, xmmword ptr [{target.arg64(1)}+0x30]") # xmm9 = m12 m13 m14 m15
        output.append(f"vshufps xmm6, xmm8, xmm9, 136")                      # xmm6 = m8 m10 m12 m14
        output.append(f"vshufps xmm7, xmm8, xmm9, 221")                      # xmm7 = m9 m11 m13 m15
        output.append(f"vpshufd xmm6, xmm6, 0x93")                           # xmm6 = m14 m8 m10 m12
        output.append(f"vpshufd xmm7, xmm7, 0x93")                           # xmm7 = m15 m9 m11 m13
        # fmt: on
    elif target.extension in (SSE41, SSE2):
        output.append(f"movups  xmm0, xmmword ptr [{target.arg64(0)}]")
        output.append(f"movups  xmm1, xmmword ptr [{target.arg64(0)}+0x10]")
        output.append(f"movaps  xmm2, xmmword ptr [BLAKE3_IV+rip]")
        output.append(f"shl {target.arg64(4)}, 32")
        output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
        output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
        output.append(f"vmovq xmm3, {target.arg64(2)}")
        output.append(f"vmovq xmm4, {target.arg64(3)}")
        output.append(f"punpcklqdq xmm3, xmm4")
        output.append(f"movups  xmm4, xmmword ptr [{target.arg64(1)}]")
        output.append(f"movups  xmm5, xmmword ptr [{target.arg64(1)}+0x10]")
        output.append(f"movaps  xmm8, xmm4")
        output.append(f"shufps  xmm4, xmm5, 136")
        output.append(f"shufps  xmm8, xmm5, 221")
        output.append(f"movaps  xmm5, xmm8")
        output.append(f"movups  xmm6, xmmword ptr [{target.arg64(1)}+0x20]")
        output.append(f"movups  xmm7, xmmword ptr [{target.arg64(1)}+0x30]")
        output.append(f"movaps  xmm8, xmm6")
        output.append(f"shufps  xmm6, xmm7, 136")
        output.append(f"pshufd  xmm6, xmm6, 0x93")
        output.append(f"shufps  xmm8, xmm7, 221")
        output.append(f"pshufd  xmm7, xmm8, 0x93")
    else:
        raise NotImplementedError


def compress_finish(target, output):
    if target.extension == AVX512:
        output.append(f"vmovdqu xmmword ptr [{target.arg64(0)}], xmm0")
        output.append(f"vmovdqu xmmword ptr [{target.arg64(0)}+0x10], xmm1")
    elif target.extension in (SSE41, SSE2):
        output.append(f"movups xmmword ptr [{target.arg64(0)}], xmm0")
        output.append(f"movups xmmword ptr [{target.arg64(0)}+0x10], xmm1")
    else:
        raise NotImplementedError


def compress(target, output):
    label = f"blake3_{target.extension}_compress"
    output.append(f".global {label}")
    output.append(f"{label}:")
    compress_setup(target, output)
    output.append(f"call {kernel_2d_name(target, 1)}")
    compress_finish(target, output)
    output.append(target.ret())


def xof_setup_2d(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            # state words
            output.append(f"vmovdqu xmm0, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vmovdqu xmm1, xmmword ptr [{target.arg64(0)}+0x10]")
            # flags
            output.append(f"shl {target.arg64(4)}, 32")
            # block length
            output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
            output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
            # counter
            output.append(f"vmovq xmm3, {target.arg64(2)}")
            output.append(f"vmovq xmm4, {target.arg64(3)}")
            output.append(f"vpunpcklqdq xmm3, xmm3, xmm4")
            output.append(f"vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]")
            # message words
            # fmt: off
            output.append(f"vmovups xmm8, xmmword ptr [{target.arg64(1)}]")      # xmm8 = m0 m1 m2 m3
            output.append(f"vmovups xmm9, xmmword ptr [{target.arg64(1)}+0x10]") # xmm9 = m4 m5 m6 m7
            output.append(f"vshufps xmm4, xmm8, xmm9, 136")                      # xmm4 = m0 m2 m4 m6
            output.append(f"vshufps xmm5, xmm8, xmm9, 221")                      # xmm5 = m1 m3 m5 m7
            output.append(f"vmovups xmm8, xmmword ptr [{target.arg64(1)}+0x20]") # xmm8 = m8 m9 m10 m12
            output.append(f"vmovups xmm9, xmmword ptr [{target.arg64(1)}+0x30]") # xmm9 = m12 m13 m14 m15
            output.append(f"vshufps xmm6, xmm8, xmm9, 136")                      # xmm6 = m8 m10 m12 m14
            output.append(f"vshufps xmm7, xmm8, xmm9, 221")                      # xmm7 = m9 m11 m13 m15
            output.append(f"vpshufd xmm6, xmm6, 0x93")                           # xmm6 = m14 m8 m10 m12
            output.append(f"vpshufd xmm7, xmm7, 0x93")                           # xmm7 = m15 m9 m11 m13
            # fmt: on
        elif degree == 2:
            # Load the state words.
            output.append(f"vbroadcasti128 ymm0, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vbroadcasti128 ymm1, xmmword ptr [{target.arg64(0)}+0x10]")
            # Load the counter increments.
            output.append(f"vmovdqa ymm4, ymmword ptr [INCREMENT_2D+rip]")
            # Load the IV constants.
            output.append(f"vbroadcasti128 ymm2, xmmword ptr [BLAKE3_IV+rip]")
            # Broadcast the counter.
            output.append(f"vpbroadcastq ymm5, {target.arg64(2)}")
            # Add the counter increments to the counter.
            output.append(f"vpaddq ymm6, ymm4, ymm5")
            # Combine the block length and flags into a 64-bit word.
            output.append(f"shl {target.arg64(4)}, 32")
            output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
            output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
            # Broadcast the block length and flags.
            output.append(f"vpbroadcastq ymm7, {target.arg64(3)}")
            # Blend the counter, block length, and flags.
            output.append(f"vpblendd ymm3, ymm6, ymm7, 0xCC")
            # Load and permute the message words.
            output.append(f"vbroadcasti128 ymm8, xmmword ptr [{target.arg64(1)}]")
            output.append(f"vbroadcasti128 ymm9, xmmword ptr [{target.arg64(1)}+0x10]")
            output.append(f"vshufps ymm4, ymm8, ymm9, 136")
            output.append(f"vshufps ymm5, ymm8, ymm9, 221")
            output.append(f"vbroadcasti128 ymm8, xmmword ptr [{target.arg64(1)}+0x20]")
            output.append(f"vbroadcasti128 ymm9, xmmword ptr [{target.arg64(1)}+0x30]")
            output.append(f"vshufps ymm6, ymm8, ymm9, 136")
            output.append(f"vshufps ymm7, ymm8, ymm9, 221")
            output.append(f"vpshufd ymm6, ymm6, 0x93")
            output.append(f"vpshufd ymm7, ymm7, 0x93")
        elif degree == 4:
            # Load the state words.
            output.append(f"vbroadcasti32x4 zmm0, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vbroadcasti32x4 zmm1, xmmword ptr [{target.arg64(0)}+0x10]")
            # Load the counter increments.
            output.append(f"vmovdqa32 zmm4, zmmword ptr [INCREMENT_2D+rip]")
            # Load the IV constants.
            output.append(f"vbroadcasti32x4 zmm2, xmmword ptr [BLAKE3_IV+rip]")
            # Broadcast the counter.
            output.append(f"vpbroadcastq zmm5, {target.arg64(2)}")
            # Add the counter increments to the counter.
            output.append(f"vpaddq zmm6, zmm4, zmm5")
            # Combine the block length and flags into a 64-bit word.
            output.append(f"shl {target.arg64(4)}, 32")
            output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
            output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
            # Broadcast the block length and flags.
            output.append(f"vpbroadcastq zmm7, {target.arg64(3)}")
            # Blend the counter, block length, and flags.
            output.append(f"mov {target.scratch32(0)}, 0xAA")
            output.append(f"kmovw k2, {target.scratch32(0)}")
            output.append(f"vpblendmq zmm3 {{k2}}, zmm6, zmm7")
            # Load and permute the message words.
            output.append(f"vbroadcasti32x4 zmm8, xmmword ptr [{target.arg64(1)}]")
            output.append(f"vbroadcasti32x4 zmm9, xmmword ptr [{target.arg64(1)}+0x10]")
            output.append(f"vshufps zmm4, zmm8, zmm9, 136")
            output.append(f"vshufps zmm5, zmm8, zmm9, 221")
            output.append(f"vbroadcasti32x4 zmm8, xmmword ptr [{target.arg64(1)}+0x20]")
            output.append(f"vbroadcasti32x4 zmm9, xmmword ptr [{target.arg64(1)}+0x30]")
            output.append(f"vshufps zmm6, zmm8, zmm9, 136")
            output.append(f"vshufps zmm7, zmm8, zmm9, 221")
            output.append(f"vpshufd zmm6, zmm6, 0x93")
            output.append(f"vpshufd zmm7, zmm7, 0x93")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        # Load the state words.
        output.append(f"vbroadcasti128 ymm0, xmmword ptr [{target.arg64(0)}]")
        output.append(f"vbroadcasti128 ymm1, xmmword ptr [{target.arg64(0)}+0x10]")
        # Load the counter increments.
        output.append(f"vmovdqa ymm4, ymmword ptr [INCREMENT_2D+rip]")
        # Load the IV constants.
        output.append(f"vbroadcasti128 ymm2, xmmword ptr [BLAKE3_IV+rip]")
        # Broadcast the counter.
        output.append(f"vpbroadcastq ymm5, {target.arg64(2)}")
        # Add the counter increments to the counter.
        output.append(f"vpaddq ymm6, ymm4, ymm5")
        # Combine the block length and flags into a 64-bit word.
        output.append(f"shl {target.arg64(4)}, 32")
        output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
        output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
        # Broadcast the block length and flags.
        output.append(f"vpbroadcastq ymm7, {target.arg64(3)}")
        # Blend the counter, block length, and flags.
        output.append(f"vpblendd ymm3, ymm6, ymm7, 0xCC")
        # Load and permute the message words.
        output.append(f"vbroadcasti128 ymm8, xmmword ptr [{target.arg64(1)}]")
        output.append(f"vbroadcasti128 ymm9, xmmword ptr [{target.arg64(1)}+0x10]")
        output.append(f"vshufps ymm4, ymm8, ymm9, 136")
        output.append(f"vshufps ymm5, ymm8, ymm9, 221")
        output.append(f"vbroadcasti128 ymm8, xmmword ptr [{target.arg64(1)}+0x20]")
        output.append(f"vbroadcasti128 ymm9, xmmword ptr [{target.arg64(1)}+0x30]")
        output.append(f"vshufps ymm6, ymm8, ymm9, 136")
        output.append(f"vshufps ymm7, ymm8, ymm9, 221")
        output.append(f"vpshufd ymm6, ymm6, 0x93")
        output.append(f"vpshufd ymm7, ymm7, 0x93")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append(f"movups  xmm0, xmmword ptr [{target.arg64(0)}]")
        output.append(f"movups  xmm1, xmmword ptr [{target.arg64(0)}+0x10]")
        output.append(f"movaps  xmm2, xmmword ptr [BLAKE3_IV+rip]")
        output.append(f"shl {target.arg64(4)}, 32")
        output.append(f"mov {target.arg32(3)}, {target.arg32(3)}")
        output.append(f"or {target.arg64(3)}, {target.arg64(4)}")
        output.append(f"vmovq xmm3, {target.arg64(2)}")
        output.append(f"vmovq xmm4, {target.arg64(3)}")
        output.append(f"punpcklqdq xmm3, xmm4")
        output.append(f"movups  xmm4, xmmword ptr [{target.arg64(1)}]")
        output.append(f"movups  xmm5, xmmword ptr [{target.arg64(1)}+0x10]")
        output.append(f"movaps  xmm8, xmm4")
        output.append(f"shufps  xmm4, xmm5, 136")
        output.append(f"shufps  xmm8, xmm5, 221")
        output.append(f"movaps  xmm5, xmm8")
        output.append(f"movups  xmm6, xmmword ptr [{target.arg64(1)}+0x20]")
        output.append(f"movups  xmm7, xmmword ptr [{target.arg64(1)}+0x30]")
        output.append(f"movaps  xmm8, xmm6")
        output.append(f"shufps  xmm6, xmm7, 136")
        output.append(f"pshufd  xmm6, xmm6, 0x93")
        output.append(f"shufps  xmm8, xmm7, 221")
        output.append(f"pshufd  xmm7, xmm8, 0x93")
    else:
        raise NotImplementedError


def xof_stream_finish_2d(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            output.append(f"vpxor xmm2, xmm2, [{target.arg64(0)}]")
            output.append(f"vpxor xmm3, xmm3, [{target.arg64(0)}+0x10]")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}], xmm0")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x10], xmm1")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x20], xmm2")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x30], xmm3")
        elif degree == 2:
            output.append(f"vbroadcasti128 ymm4, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vpxor ymm2, ymm2, ymm4")
            output.append(f"vbroadcasti128 ymm5, xmmword ptr [{target.arg64(0)} + 16]")
            output.append(f"vpxor ymm3, ymm3, ymm5")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 0 * 16], xmm0")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 1 * 16], xmm1")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 2 * 16], xmm2")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 3 * 16], xmm3")
            output.append(f"vextracti128 xmmword ptr [{target.arg64(5)}+4*16], ymm0, 1")
            output.append(f"vextracti128 xmmword ptr [{target.arg64(5)}+5*16], ymm1, 1")
            output.append(f"vextracti128 xmmword ptr [{target.arg64(5)}+6*16], ymm2, 1")
            output.append(f"vextracti128 xmmword ptr [{target.arg64(5)}+7*16], ymm3, 1")
        elif degree == 4:
            output.append(f"vbroadcasti32x4 zmm4, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vpxord zmm2, zmm2, zmm4")
            output.append(f"vbroadcasti32x4 zmm5, xmmword ptr [{target.arg64(0)} + 16]")
            output.append(f"vpxord zmm3, zmm3, zmm5")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 0 * 16], xmm0")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 1 * 16], xmm1")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 2 * 16], xmm2")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 3 * 16], xmm3")
            for i in range(1, 4):
                for reg in range(0, 4):
                    output.append(
                        f"vextracti32x4 xmmword ptr [{target.arg64(5)} + {4*i+reg} * 16], zmm{reg}, {i}"
                    )
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        output.append(f"vbroadcasti128 ymm4, xmmword ptr [{target.arg64(0)}]")
        output.append(f"vpxor ymm2, ymm2, ymm4")
        output.append(f"vbroadcasti128 ymm5, xmmword ptr [{target.arg64(0)} + 16]")
        output.append(f"vpxor ymm3, ymm3, ymm5")
        output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 0 * 16], xmm0")
        output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 1 * 16], xmm1")
        output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 2 * 16], xmm2")
        output.append(f"vmovdqu xmmword ptr [{target.arg64(5)} + 3 * 16], xmm3")
        output.append(f"vextracti128 xmmword ptr [{target.arg64(5)} + 4 * 16], ymm0, 1")
        output.append(f"vextracti128 xmmword ptr [{target.arg64(5)} + 5 * 16], ymm1, 1")
        output.append(f"vextracti128 xmmword ptr [{target.arg64(5)} + 6 * 16], ymm2, 1")
        output.append(f"vextracti128 xmmword ptr [{target.arg64(5)} + 7 * 16], ymm3, 1")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append(f"movdqu xmm4, xmmword ptr [{target.arg64(0)}]")
        output.append(f"movdqu xmm5, xmmword ptr [{target.arg64(0)}+0x10]")
        output.append(f"pxor xmm2, xmm4")
        output.append(f"pxor xmm3, xmm5")
        output.append(f"movups xmmword ptr [{target.arg64(5)}], xmm0")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x10], xmm1")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x20], xmm2")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x30], xmm3")
    else:
        raise NotImplementedError


def xof_xor_finish_2d(target, output, degree):
    if target.extension == AVX512:
        if degree == 1:
            output.append(f"vpxor xmm2, xmm2, [{target.arg64(0)}]")
            output.append(f"vpxor xmm3, xmm3, [{target.arg64(0)}+0x10]")
            output.append(f"vpxor xmm0, xmm0, [{target.arg64(5)}]")
            output.append(f"vpxor xmm1, xmm1, [{target.arg64(5)}+0x10]")
            output.append(f"vpxor xmm2, xmm2, [{target.arg64(5)}+0x20]")
            output.append(f"vpxor xmm3, xmm3, [{target.arg64(5)}+0x30]")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}], xmm0")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x10], xmm1")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x20], xmm2")
            output.append(f"vmovdqu xmmword ptr [{target.arg64(5)}+0x30], xmm3")
        elif degree == 2:
            output.append(f"vbroadcasti128 ymm4, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vpxor ymm2, ymm2, ymm4")
            output.append(f"vbroadcasti128 ymm5, xmmword ptr [{target.arg64(0)} + 16]")
            output.append(f"vpxor ymm3, ymm3, ymm5")
            # Each vector now holds rows from two different states:
            # ymm0:  a0,  a1,  a2,  a3,  b0,  b1,  b2,  b3
            # ymm1:  a4,  a5,  a6,  a7,  b4,  b5,  b6,  b7
            # ymm2:  a8,  a9, a10, a11,  b8,  b9, b10, b11
            # ymm3: a12, a13, a14, a15, b12, b13, b14, b15
            # We want to rearrange the 128-bit lanes like this, so we can load
            # destination bytes and XOR them in directly.
            # ymm4:  a0,  a1,  a2,  a3,  a4,  a5,  a6,  a7
            # ymm5:  a8,  a9, a10, a11, a12, a13, a14, a15
            # ymm6:  b0,  b1,  b2,  b3,  b4,  b5,  b6,  b7
            # ymm7:  b8,  b9, b10, b11, b12, b13, b14, b15
            output.append(f"vperm2f128 ymm4, ymm0, ymm1, {0b0010_0000}")  # lower 128
            output.append(f"vperm2f128 ymm5, ymm2, ymm3, {0b0010_0000}")
            output.append(f"vperm2f128 ymm6, ymm0, ymm1, {0b0011_0001}")  # upper 128
            output.append(f"vperm2f128 ymm7, ymm2, ymm3, {0b0011_0001}")
            # XOR in the bytes that are already in the destination.
            output.append(f"vpxor ymm4, ymm4, ymmword ptr [{target.arg64(5)} + 0 * 32]")
            output.append(f"vpxor ymm5, ymm5, ymmword ptr [{target.arg64(5)} + 1 * 32]")
            output.append(f"vpxor ymm6, ymm6, ymmword ptr [{target.arg64(5)} + 2 * 32]")
            output.append(f"vpxor ymm7, ymm7, ymmword ptr [{target.arg64(5)} + 3 * 32]")
            # Write out the XOR results.
            output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 0 * 32], ymm4")
            output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 1 * 32], ymm5")
            output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 2 * 32], ymm6")
            output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 3 * 32], ymm7")
        elif degree == 4:
            output.append(f"vbroadcasti32x4 zmm4, xmmword ptr [{target.arg64(0)}]")
            output.append(f"vpxord zmm2, zmm2, zmm4")
            output.append(f"vbroadcasti32x4 zmm5, xmmword ptr [{target.arg64(0)} + 16]")
            output.append(f"vpxord zmm3, zmm3, zmm5")
            # Each vector now holds rows from four different states:
            # zmm0:  a0,  a1,  a2,  a3,  b0,  b1,  b2,  b3,  c0,  c1,  c2,  c3,  d0,  d1,  d2,  d3
            # zmm1:  a4,  a5,  a6,  a7,  b4,  b5,  b6,  b7,  c4,  c5,  c6,  c7,  d4,  d5,  d6,  d7
            # zmm2:  a8,  a9, a10, a11,  b8,  b9, b10, b11,  c8,  c9, c10, c11,  d8,  d9, d10, d11
            # zmm3: a12, a13, a14, a15, b12, b13, b14, b15, c12, c13, c14, c15, d12, d13, d14, d15
            # We want to rearrange the 128-bit lanes like this, so we can load
            # destination bytes and XOR them in directly.
            # zmm0:  a0,  a1,  a2,  a3,  a4,  a5,  a6,  a7,  a8,  a9, a10, a11, a12, a13, a14, a15
            # zmm1:  b0,  b1,  b2,  b3,  b4,  b5,  b6,  b7,  b8,  b9, b10, b11, b12, b13, b14, b15
            # zmm2:  c0,  c1,  c2,  c3,  c4,  c5,  c6,  c7,  c8,  c9, c10, c11, c12, c13, c14, c15
            # zmm3:  d0,  d1,  d2,  d3,  d4,  d5,  d6,  d7,  d8,  d9, d10, d11, d12, d13, d14, d15
            #
            # This first interleaving of 256-bit lanes produces vectors like:
            # zmm4:  a0,  a1,  a2,  a3,  b0,  b1,  b2,  b3,  a4,  a5,  a6,  a7,  b4,  b5,  b6,  b7
            output.append(f"vshufi32x4 zmm4, zmm0, zmm1, {0b0100_0100}")  # low 256
            output.append(f"vshufi32x4 zmm5, zmm0, zmm1, {0b1110_1110}")  # high 256
            output.append(f"vshufi32x4 zmm6, zmm2, zmm3, {0b0100_0100}")
            output.append(f"vshufi32x4 zmm7, zmm2, zmm3, {0b1110_1110}")
            # And this second interleaving of 128-bit lanes within each 256-bit
            # lane produces the vectors we want.
            output.append(f"vshufi32x4 zmm0, zmm4, zmm6, {0b1000_1000}")  # low 128
            output.append(f"vshufi32x4 zmm1, zmm4, zmm6, {0b1101_1101}")  # high 128
            output.append(f"vshufi32x4 zmm2, zmm5, zmm7, {0b1000_1000}")
            output.append(f"vshufi32x4 zmm3, zmm5, zmm7, {0b1101_1101}")
            # XOR in the bytes that are already in the destination.
            output.append(f"vpxord zmm0, zmm0, zmmword ptr [{target.arg64(5)} + 0*64]")
            output.append(f"vpxord zmm1, zmm1, zmmword ptr [{target.arg64(5)} + 1*64]")
            output.append(f"vpxord zmm2, zmm2, zmmword ptr [{target.arg64(5)} + 2*64]")
            output.append(f"vpxord zmm3, zmm3, zmmword ptr [{target.arg64(5)} + 3*64]")
            # Write out the XOR results.
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 0*64], zmm0")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 1*64], zmm1")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 2*64], zmm2")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 3*64], zmm3")
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        output.append(f"vbroadcasti128 ymm4, xmmword ptr [{target.arg64(0)}]")
        output.append(f"vpxor ymm2, ymm2, ymm4")
        output.append(f"vbroadcasti128 ymm5, xmmword ptr [{target.arg64(0)} + 16]")
        output.append(f"vpxor ymm3, ymm3, ymm5")
        # Each vector now holds rows from two different states:
        # ymm0:  a0,  a1,  a2,  a3,  b0,  b1,  b2,  b3
        # ymm1:  a4,  a5,  a6,  a7,  b4,  b5,  b6,  b7
        # ymm2:  a8,  a9, a10, a11,  b8,  b9, b10, b11
        # ymm3: a12, a13, a14, a15, b12, b13, b14, b15
        # We want to rearrange the 128-bit lanes like this, so we can load
        # destination bytes and XOR them in directly.
        # ymm4:  a0,  a1,  a2,  a3,  a4,  a5,  a6,  a7
        # ymm5:  a8,  a9, a10, a11, a12, a13, a14, a15
        # ymm6:  b0,  b1,  b2,  b3,  b4,  b5,  b6,  b7
        # ymm7:  b8,  b9, b10, b11, b12, b13, b14, b15
        output.append(f"vperm2f128 ymm4, ymm0, ymm1, {0b0010_0000}")  # lower 128
        output.append(f"vperm2f128 ymm5, ymm2, ymm3, {0b0010_0000}")
        output.append(f"vperm2f128 ymm6, ymm0, ymm1, {0b0011_0001}")  # upper 128
        output.append(f"vperm2f128 ymm7, ymm2, ymm3, {0b0011_0001}")
        # XOR in the bytes that are already in the destination.
        output.append(f"vpxor ymm4, ymm4, ymmword ptr [{target.arg64(5)} + 0 * 32]")
        output.append(f"vpxor ymm5, ymm5, ymmword ptr [{target.arg64(5)} + 1 * 32]")
        output.append(f"vpxor ymm6, ymm6, ymmword ptr [{target.arg64(5)} + 2 * 32]")
        output.append(f"vpxor ymm7, ymm7, ymmword ptr [{target.arg64(5)} + 3 * 32]")
        # Write out the XOR results.
        output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 0 * 32], ymm4")
        output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 1 * 32], ymm5")
        output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 2 * 32], ymm6")
        output.append(f"vmovdqu ymmword ptr [{target.arg64(5)} + 3 * 32], ymm7")
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        output.append(f"movdqu xmm4, xmmword ptr [{target.arg64(0)}]")
        output.append(f"movdqu xmm5, xmmword ptr [{target.arg64(0)}+0x10]")
        output.append(f"pxor xmm2, xmm4")
        output.append(f"pxor xmm3, xmm5")
        output.append(f"movdqu xmm4, [{target.arg64(5)}]")
        output.append(f"movdqu xmm5, [{target.arg64(5)}+0x10]")
        output.append(f"movdqu xmm6, [{target.arg64(5)}+0x20]")
        output.append(f"movdqu xmm7, [{target.arg64(5)}+0x30]")
        output.append(f"pxor xmm0, xmm4")
        output.append(f"pxor xmm1, xmm5")
        output.append(f"pxor xmm2, xmm6")
        output.append(f"pxor xmm3, xmm7")
        output.append(f"movups xmmword ptr [{target.arg64(5)}], xmm0")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x10], xmm1")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x20], xmm2")
        output.append(f"movups xmmword ptr [{target.arg64(5)}+0x30], xmm3")
    else:
        raise NotImplementedError


def g_function_3d(target, output, degree, columns, msg_words1, msg_words2):
    if target.extension == SSE41:
        assert degree == 4
    elif target.extension == AVX2:
        assert degree == 8
    elif target.extension == AVX512:
        assert degree in (8, 16)
    else:
        raise NotImplementedError
    for (column, m1) in zip(columns, msg_words1):
        add_row(target, output, degree, dest=column[0], src=m1)
    for column in columns:
        add_row(target, output, degree, dest=column[0], src=column[1])
    for column in columns:
        xor_row(target, output, degree, dest=column[3], src=column[0])
    for column in columns:
        bitrotate_row(target, output, degree, reg=column[3], bits=16)
    for column in columns:
        add_row(target, output, degree, dest=column[2], src=column[3])
    for column in columns:
        xor_row(target, output, degree, dest=column[1], src=column[2])
    for column in columns:
        bitrotate_row(target, output, degree, reg=column[1], bits=12)
    for (column, m2) in zip(columns, msg_words2):
        add_row(target, output, degree, dest=column[0], src=m2)
    for column in columns:
        add_row(target, output, degree, dest=column[0], src=column[1])
    for column in columns:
        xor_row(target, output, degree, dest=column[3], src=column[0])
    for column in columns:
        bitrotate_row(target, output, degree, reg=column[3], bits=8)
    for column in columns:
        add_row(target, output, degree, dest=column[2], src=column[3])
    for column in columns:
        xor_row(target, output, degree, dest=column[1], src=column[2])
    for column in columns:
        bitrotate_row(target, output, degree, reg=column[1], bits=7)


def kernel_3d_name(target, degree):
    return f"blake3_{target.extension}_kernel_3d_{degree}"


def kernel_3d(target, output, degree):
    label = kernel_3d_name(target, degree)
    output.append(f"{label}:")
    if target.extension == SSE41:
        assert degree == 4
    elif target.extension == AVX2:
        assert degree == 8
    elif target.extension == AVX512:
        assert degree in (8, 16)
    else:
        raise NotImplementedError
    for round_number in range(7):
        straight_columns = [
            [0, 4, 8, 12],
            [1, 5, 9, 13],
            [2, 6, 10, 14],
            [3, 7, 11, 15],
        ]
        msg_words1 = [16 + MESSAGE_SCHEDULE[round_number][i] for i in [0, 2, 4, 6]]
        msg_words2 = [16 + MESSAGE_SCHEDULE[round_number][i] for i in [1, 3, 5, 7]]
        g_function_3d(target, output, degree, straight_columns, msg_words1, msg_words2)
        diagonal_columns = [
            [0, 5, 10, 15],
            [1, 6, 11, 12],
            [2, 7, 8, 13],
            [3, 4, 9, 14],
        ]
        msg_words1 = [16 + MESSAGE_SCHEDULE[round_number][i] for i in [8, 10, 12, 14]]
        msg_words2 = [16 + MESSAGE_SCHEDULE[round_number][i] for i in [9, 11, 13, 15]]
        g_function_3d(target, output, degree, diagonal_columns, msg_words1, msg_words2)
    # Xor the last two rows into the first two, but don't do the feed forward
    # here. That's only done in the XOF case.
    for dest in range(8):
        xor_row(target, output, degree, dest=dest, src=dest + 8)
    output.append(target.ret())


def xof_setup_3d(target, output, degree):
    if target.extension == AVX512:
        if degree == 16:
            # Load vpermi2d indexes into the counter registers.
            output.append(f"vmovdqa32 zmm12, zmmword ptr [rip + EVEN_INDEXES]")
            output.append(f"vmovdqa32 zmm13, zmmword ptr [rip + ODD_INDEXES]")
            # Load the state words.
            for i in range(8):
                output.append(
                    f"vpbroadcastd zmm{i}, dword ptr [{target.arg64(0)}+{4*i}]"
                )
            # Load the message words.
            for i in range(16):
                output.append(
                    f"vpbroadcastd zmm{i+16}, dword ptr [{target.arg64(1)}+{4*i}]"
                )
            # Load the 64-bit counter increments into a temporary register.
            output.append(f"vmovdqa64 zmm8, zmmword ptr [INCREMENT_3D+rip]")
            # Broadcast the counter and add it to the increments. This gives
            # the first 8 counter values.
            output.append(f"vpbroadcastq zmm9, {target.arg64(2)}")
            output.append(f"vpaddq zmm9, zmm9, zmm8")
            # Increment the counter and repeat that for the last 8 counter values.
            output.append(f"add {target.arg64(2)}, 8")
            output.append(f"vpbroadcastq zmm10, {target.arg64(2)}")
            output.append(f"vpaddq zmm10, zmm10, zmm8")
            # Extract the lower and upper halves of the counter words, using
            # the permutation tables loaded above.
            output.append(f"vpermi2d zmm12, zmm9, zmm10")
            output.append(f"vpermi2d zmm13, zmm9, zmm10")
            # Load the block length.
            output.append(f"vpbroadcastd zmm14, {target.arg32(3)}")
            # Load the domain flags.
            output.append(f"vpbroadcastd zmm15, {target.arg32(4)}")
            # Load the IV constants.
            for i in range(4):
                output.append(f"vpbroadcastd zmm{i+8}, dword ptr [BLAKE3_IV+rip+{4*i}]")
        else:
            raise NotImplementedError
    else:
        raise NotImplementedError


def xof_stream_finish_3d(target, output, degree):
    if target.extension == AVX512:
        if degree == 16:
            # Re-broadcast the input CV and feed it forward into the second half of the state.
            output.append(f"vpbroadcastd zmm16, dword ptr [{target.arg64(0)} + 0 * 4]")
            output.append(f"vpxord zmm8, zmm8, zmm16")
            output.append(f"vpbroadcastd zmm17, dword ptr [{target.arg64(0)} + 1 * 4]")
            output.append(f"vpxord zmm9, zmm9, zmm17")
            output.append(f"vpbroadcastd zmm18, dword ptr [{target.arg64(0)} + 2 * 4]")
            output.append(f"vpxord zmm10, zmm10, zmm18")
            output.append(f"vpbroadcastd zmm19, dword ptr [{target.arg64(0)} + 3 * 4]")
            output.append(f"vpxord zmm11, zmm11, zmm19")
            output.append(f"vpbroadcastd zmm20, dword ptr [{target.arg64(0)} + 4 * 4]")
            output.append(f"vpxord zmm12, zmm12, zmm20")
            output.append(f"vpbroadcastd zmm21, dword ptr [{target.arg64(0)} + 5 * 4]")
            output.append(f"vpxord zmm13, zmm13, zmm21")
            output.append(f"vpbroadcastd zmm22, dword ptr [{target.arg64(0)} + 6 * 4]")
            output.append(f"vpxord zmm14, zmm14, zmm22")
            output.append(f"vpbroadcastd zmm23, dword ptr [{target.arg64(0)} + 7 * 4]")
            output.append(f"vpxord zmm15, zmm15, zmm23")
            # zmm0-zmm15 now contain the final extended state vectors, transposed. We need to un-transpose
            # them before we write them out. As with blake3_avx512_blocks_16, we prefer to avoid expensive
            # operations across 128-bit lanes, so we do a couple of interleaving passes and then write out
            # 128 bits at a time.
            #
            # First, interleave 32-bit words. Use zmm16-zmm31 to hold the intermediate results. This
            # takes the input vectors like:
            #
            # a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
            #
            # And produces vectors like:
            #
            # a0, a1, b0, b1, e0, e1, g0, g1, i0, i1, k0, k1, m0, m1, o0, o1
            #
            # Then interleave 64-bit words back into zmm0-zmm15, producing vectors like:
            #
            # a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
            #
            # Finally, write out each 128-bit group, unaligned.
            output.append(f"vpunpckldq zmm16, zmm0, zmm1")
            output.append(f"vpunpckhdq zmm17, zmm0, zmm1")
            output.append(f"vpunpckldq zmm18, zmm2, zmm3")
            output.append(f"vpunpckhdq zmm19, zmm2, zmm3")
            output.append(f"vpunpcklqdq zmm0, zmm16, zmm18")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 0 * 16], xmm0")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 16 * 16], zmm0, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 32 * 16], zmm0, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 48 * 16], zmm0, 3"
            )
            output.append(f"vpunpckhqdq zmm1, zmm16, zmm18")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 4 * 16], xmm1")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 20 * 16], zmm1, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 36 * 16], zmm1, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 52 * 16], zmm1, 3"
            )
            output.append(f"vpunpcklqdq zmm2, zmm17, zmm19")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 8 * 16], xmm2")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 24 * 16], zmm2, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 40 * 16], zmm2, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 56 * 16], zmm2, 3"
            )
            output.append(f"vpunpckhqdq zmm3, zmm17, zmm19")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 12 * 16], xmm3")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 28 * 16], zmm3, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 44 * 16], zmm3, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 60 * 16], zmm3, 3"
            )
            output.append(f"vpunpckldq zmm20, zmm4, zmm5")
            output.append(f"vpunpckhdq zmm21, zmm4, zmm5")
            output.append(f"vpunpckldq zmm22, zmm6, zmm7")
            output.append(f"vpunpckhdq zmm23, zmm6, zmm7")
            output.append(f"vpunpcklqdq zmm4, zmm20, zmm22")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 1 * 16], xmm4")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 17 * 16], zmm4, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 33 * 16], zmm4, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 49 * 16], zmm4, 3"
            )
            output.append(f"vpunpckhqdq zmm5, zmm20, zmm22")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 5 * 16], xmm5")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 21 * 16], zmm5, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 37 * 16], zmm5, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 53 * 16], zmm5, 3"
            )
            output.append(f"vpunpcklqdq zmm6, zmm21, zmm23")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 9 * 16], xmm6")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 25 * 16], zmm6, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 41 * 16], zmm6, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 57 * 16], zmm6, 3"
            )
            output.append(f"vpunpckhqdq zmm7, zmm21, zmm23")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 13 * 16], xmm7")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 29 * 16], zmm7, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 45 * 16], zmm7, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 61 * 16], zmm7, 3"
            )
            output.append(f"vpunpckldq zmm24, zmm8, zmm9")
            output.append(f"vpunpckhdq zmm25, zmm8, zmm9")
            output.append(f"vpunpckldq zmm26, zmm10, zmm11")
            output.append(f"vpunpckhdq zmm27, zmm10, zmm11")
            output.append(f"vpunpcklqdq zmm8, zmm24, zmm26")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 2 * 16], xmm8")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 18 * 16], zmm8, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 34 * 16], zmm8, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 50 * 16], zmm8, 3"
            )
            output.append(f"vpunpckhqdq zmm9, zmm24, zmm26")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 6 * 16], xmm9")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 22 * 16], zmm9, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 38 * 16], zmm9, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 54 * 16], zmm9, 3"
            )
            output.append(f"vpunpcklqdq zmm10, zmm25, zmm27")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 10 * 16], xmm10")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 26 * 16], zmm10, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 42 * 16], zmm10, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 58 * 16], zmm10, 3"
            )
            output.append(f"vpunpckhqdq zmm11, zmm25, zmm27")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 14 * 16], xmm11")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 30 * 16], zmm11, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 46 * 16], zmm11, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 62 * 16], zmm11, 3"
            )
            output.append(f"vpunpckldq zmm28, zmm12, zmm13")
            output.append(f"vpunpckhdq zmm29, zmm12, zmm13")
            output.append(f"vpunpckldq zmm30, zmm14, zmm15")
            output.append(f"vpunpckhdq zmm31, zmm14, zmm15")
            output.append(f"vpunpcklqdq zmm12, zmm28, zmm30")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 3 * 16], xmm12")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 19 * 16], zmm12, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 35 * 16], zmm12, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 51 * 16], zmm12, 3"
            )
            output.append(f"vpunpckhqdq zmm13, zmm28, zmm30")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 7 * 16], xmm13")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 23 * 16], zmm13, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 39 * 16], zmm13, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 55 * 16], zmm13, 3"
            )
            output.append(f"vpunpcklqdq zmm14, zmm29, zmm31")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 11 * 16], xmm14")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 27 * 16], zmm14, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 43 * 16], zmm14, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 59 * 16], zmm14, 3"
            )
            output.append(f"vpunpckhqdq zmm15, zmm29, zmm31")
            output.append(f"vmovdqu32 xmmword ptr [{target.arg64(5)} + 15 * 16], xmm15")
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 31 * 16], zmm15, 1"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 47 * 16], zmm15, 2"
            )
            output.append(
                f"vextracti32x4 xmmword ptr [{target.arg64(5)} + 63 * 16], zmm15, 3"
            )
        else:
            raise NotImplementedError
    else:
        raise NotImplementedError


def xof_xor_finish_3d(target, output, degree):
    if target.extension == AVX512:
        if degree == 16:
            # Re-broadcast the input CV and feed it forward into the second half of the state.
            output.append(f"vpbroadcastd zmm16, dword ptr [{target.arg64(0)} + 0 * 4]")
            output.append(f"vpxord zmm8, zmm8, zmm16")
            output.append(f"vpbroadcastd zmm17, dword ptr [{target.arg64(0)} + 1 * 4]")
            output.append(f"vpxord zmm9, zmm9, zmm17")
            output.append(f"vpbroadcastd zmm18, dword ptr [{target.arg64(0)} + 2 * 4]")
            output.append(f"vpxord zmm10, zmm10, zmm18")
            output.append(f"vpbroadcastd zmm19, dword ptr [{target.arg64(0)} + 3 * 4]")
            output.append(f"vpxord zmm11, zmm11, zmm19")
            output.append(f"vpbroadcastd zmm20, dword ptr [{target.arg64(0)} + 4 * 4]")
            output.append(f"vpxord zmm12, zmm12, zmm20")
            output.append(f"vpbroadcastd zmm21, dword ptr [{target.arg64(0)} + 5 * 4]")
            output.append(f"vpxord zmm13, zmm13, zmm21")
            output.append(f"vpbroadcastd zmm22, dword ptr [{target.arg64(0)} + 6 * 4]")
            output.append(f"vpxord zmm14, zmm14, zmm22")
            output.append(f"vpbroadcastd zmm23, dword ptr [{target.arg64(0)} + 7 * 4]")
            output.append(f"vpxord zmm15, zmm15, zmm23")
            # zmm0-zmm15 now contain the final extended state vectors, transposed. We need to un-transpose
            # them before we write them out. Unlike blake3_avx512_xof_stream_16, we do a complete
            # un-transpose here, to make the xor step easier.
            #
            # First interleave 32-bit words. This takes vectors like:
            #
            # a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0
            #
            # And produces vectors like:
            #
            # a0, a1, b0, b1, e0, e1, g0, g1, i0, i1, k0, k1, m0, m1, o0, o1
            output.append(f"vpunpckldq zmm16, zmm0, zmm1")
            output.append(f"vpunpckhdq zmm17, zmm0, zmm1")
            output.append(f"vpunpckldq zmm18, zmm2, zmm3")
            output.append(f"vpunpckhdq zmm19, zmm2, zmm3")
            output.append(f"vpunpckldq zmm20, zmm4, zmm5")
            output.append(f"vpunpckhdq zmm21, zmm4, zmm5")
            output.append(f"vpunpckldq zmm22, zmm6, zmm7")
            output.append(f"vpunpckhdq zmm23, zmm6, zmm7")
            output.append(f"vpunpckldq zmm24, zmm8, zmm9")
            output.append(f"vpunpckhdq zmm25, zmm8, zmm9")
            output.append(f"vpunpckldq zmm26, zmm10, zmm11")
            output.append(f"vpunpckhdq zmm27, zmm10, zmm11")
            output.append(f"vpunpckldq zmm28, zmm12, zmm13")
            output.append(f"vpunpckhdq zmm29, zmm12, zmm13")
            output.append(f"vpunpckldq zmm30, zmm14, zmm15")
            output.append(f"vpunpckhdq zmm31, zmm14, zmm15")
            # Then interleave 64-bit words, producing vectors like:
            #
            # a0, a1, a2, a3, e0, e1, e2, e3, i0, i1, i2, i3, m0, m1, m2, m3
            output.append(f"vpunpcklqdq zmm0, zmm16, zmm18")
            output.append(f"vpunpckhqdq zmm1, zmm16, zmm18")
            output.append(f"vpunpcklqdq zmm2, zmm17, zmm19")
            output.append(f"vpunpckhqdq zmm3, zmm17, zmm19")
            output.append(f"vpunpcklqdq zmm4, zmm20, zmm22")
            output.append(f"vpunpckhqdq zmm5, zmm20, zmm22")
            output.append(f"vpunpcklqdq zmm6, zmm21, zmm23")
            output.append(f"vpunpckhqdq zmm7, zmm21, zmm23")
            output.append(f"vpunpcklqdq zmm8, zmm24, zmm26")
            output.append(f"vpunpckhqdq zmm9, zmm24, zmm26")
            output.append(f"vpunpcklqdq zmm10, zmm25, zmm27")
            output.append(f"vpunpckhqdq zmm11, zmm25, zmm27")
            output.append(f"vpunpcklqdq zmm12, zmm28, zmm30")
            output.append(f"vpunpckhqdq zmm13, zmm28, zmm30")
            output.append(f"vpunpcklqdq zmm14, zmm29, zmm31")
            output.append(f"vpunpckhqdq zmm15, zmm29, zmm31")
            # Then interleave 128-bit lanes, producing vectors like:
            #
            # a0, a1, a2, a3, i0, i1, i2, i3, a4, a5, a6, a7, i4, i5, i6, i7
            output.append(
                "vshufi32x4 zmm16, zmm0, zmm4, 0x88"
            )  # lo lanes: 0x88 = 0b10001000 = (0, 2, 0, 2)
            output.append(f"vshufi32x4 zmm17, zmm1, zmm5, 0x88")
            output.append(f"vshufi32x4 zmm18, zmm2, zmm6, 0x88")
            output.append(f"vshufi32x4 zmm19, zmm3, zmm7, 0x88")
            output.append(
                "vshufi32x4 zmm20, zmm0, zmm4, 0xdd"
            )  # hi lanes: 0xdd = 0b11011101 = (1, 3, 1, 3)
            output.append(f"vshufi32x4 zmm21, zmm1, zmm5, 0xdd")
            output.append(f"vshufi32x4 zmm22, zmm2, zmm6, 0xdd")
            output.append(f"vshufi32x4 zmm23, zmm3, zmm7, 0xdd")
            output.append(f"vshufi32x4 zmm24, zmm8, zmm12, 0x88")  # lo lanes
            output.append(f"vshufi32x4 zmm25, zmm9, zmm13, 0x88")
            output.append(f"vshufi32x4 zmm26, zmm10, zmm14, 0x88")
            output.append(f"vshufi32x4 zmm27, zmm11, zmm15, 0x88")
            output.append(f"vshufi32x4 zmm28, zmm8, zmm12, 0xdd")  # hi lanes
            output.append(f"vshufi32x4 zmm29, zmm9, zmm13, 0xdd")
            output.append(f"vshufi32x4 zmm30, zmm10, zmm14, 0xdd")
            output.append(f"vshufi32x4 zmm31, zmm11, zmm15, 0xdd")
            # Finally interleave 128-bit lanes again (the same permutation as the previous pass, but
            # different inputs), producing vectors like:
            #
            # a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15
            output.append(f"vshufi32x4 zmm0, zmm16, zmm24, 0x88")  # lo lanes
            output.append(f"vshufi32x4 zmm1, zmm17, zmm25, 0x88")
            output.append(f"vshufi32x4 zmm2, zmm18, zmm26, 0x88")
            output.append(f"vshufi32x4 zmm3, zmm19, zmm27, 0x88")
            output.append(f"vshufi32x4 zmm4, zmm20, zmm28, 0x88")
            output.append(f"vshufi32x4 zmm5, zmm21, zmm29, 0x88")
            output.append(f"vshufi32x4 zmm6, zmm22, zmm30, 0x88")
            output.append(f"vshufi32x4 zmm7, zmm23, zmm31, 0x88")
            output.append(f"vshufi32x4 zmm8, zmm16, zmm24, 0xdd")  # hi lanes
            output.append(f"vshufi32x4 zmm9, zmm17, zmm25, 0xdd")
            output.append(f"vshufi32x4 zmm10, zmm18, zmm26, 0xdd")
            output.append(f"vshufi32x4 zmm11, zmm19, zmm27, 0xdd")
            output.append(f"vshufi32x4 zmm12, zmm20, zmm28, 0xdd")
            output.append(f"vshufi32x4 zmm13, zmm21, zmm29, 0xdd")
            output.append(f"vshufi32x4 zmm14, zmm22, zmm30, 0xdd")
            output.append(f"vshufi32x4 zmm15, zmm23, zmm31, 0xdd")
            # zmm0-zmm15 now contain the fully un-transposed state words. Load each 64 block on input
            # (unaligned), perform the xor, and write out the result (again unaligned).
            output.append(f"vmovdqu32 zmm16, zmmword ptr [{target.arg64(5)} + 0 * 64]")
            output.append(f"vpxord zmm0, zmm0, zmm16")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 0 * 64], zmm0")
            output.append(f"vmovdqu32 zmm17, zmmword ptr [{target.arg64(5)} + 1 * 64]")
            output.append(f"vpxord zmm1, zmm1, zmm17")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 1 * 64], zmm1")
            output.append(f"vmovdqu32 zmm18, zmmword ptr [{target.arg64(5)} + 2 * 64]")
            output.append(f"vpxord zmm2, zmm2, zmm18")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 2 * 64], zmm2")
            output.append(f"vmovdqu32 zmm19, zmmword ptr [{target.arg64(5)} + 3 * 64]")
            output.append(f"vpxord zmm3, zmm3, zmm19")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 3 * 64], zmm3")
            output.append(f"vmovdqu32 zmm20, zmmword ptr [{target.arg64(5)} + 4 * 64]")
            output.append(f"vpxord zmm4, zmm4, zmm20")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 4 * 64], zmm4")
            output.append(f"vmovdqu32 zmm21, zmmword ptr [{target.arg64(5)} + 5 * 64]")
            output.append(f"vpxord zmm5, zmm5, zmm21")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 5 * 64], zmm5")
            output.append(f"vmovdqu32 zmm22, zmmword ptr [{target.arg64(5)} + 6 * 64]")
            output.append(f"vpxord zmm6, zmm6, zmm22")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 6 * 64], zmm6")
            output.append(f"vmovdqu32 zmm23, zmmword ptr [{target.arg64(5)} + 7 * 64]")
            output.append(f"vpxord zmm7, zmm7, zmm23")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 7 * 64], zmm7")
            output.append(f"vmovdqu32 zmm24, zmmword ptr [{target.arg64(5)} + 8 * 64]")
            output.append(f"vpxord zmm8, zmm8, zmm24")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 8 * 64], zmm8")
            output.append(f"vmovdqu32 zmm25, zmmword ptr [{target.arg64(5)} + 9 * 64]")
            output.append(f"vpxord zmm9, zmm9, zmm25")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 9 * 64], zmm9")
            output.append(f"vmovdqu32 zmm26, zmmword ptr [{target.arg64(5)} + 10 * 64]")
            output.append(f"vpxord zmm10, zmm10, zmm26")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 10 * 64], zmm10")
            output.append(f"vmovdqu32 zmm27, zmmword ptr [{target.arg64(5)} + 11 * 64]")
            output.append(f"vpxord zmm11, zmm11, zmm27")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 11 * 64], zmm11")
            output.append(f"vmovdqu32 zmm28, zmmword ptr [{target.arg64(5)} + 12 * 64]")
            output.append(f"vpxord zmm12, zmm12, zmm28")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 12 * 64], zmm12")
            output.append(f"vmovdqu32 zmm29, zmmword ptr [{target.arg64(5)} + 13 * 64]")
            output.append(f"vpxord zmm13, zmm13, zmm29")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 13 * 64], zmm13")
            output.append(f"vmovdqu32 zmm30, zmmword ptr [{target.arg64(5)} + 14 * 64]")
            output.append(f"vpxord zmm14, zmm14, zmm30")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 14 * 64], zmm14")
            output.append(f"vmovdqu32 zmm31, zmmword ptr [{target.arg64(5)} + 15 * 64]")
            output.append(f"vpxord zmm15, zmm15, zmm31")
            output.append(f"vmovdqu32 zmmword ptr [{target.arg64(5)} + 15 * 64], zmm15")
        else:
            raise NotImplementedError
    else:
        raise NotImplementedError


def xof_fn(target, output, degree, xor):
    variant = "xor" if xor else "stream"
    finish_fn_2d = xof_xor_finish_2d if xor else xof_stream_finish_2d
    finish_fn_3d = xof_xor_finish_3d if xor else xof_stream_finish_3d
    label = f"blake3_{target.extension}_xof_{variant}_{degree}"
    output.append(f".global {label}")
    output.append(f"{label}:")
    if target.extension == AVX512:
        if degree in (1, 2, 4):
            xof_setup_2d(target, output, degree)
            output.append(f"call {kernel_2d_name(target, degree)}")
            finish_fn_2d(target, output, degree)
        elif degree in (8, 16):
            xof_setup_3d(target, output, degree)
            output.append(f"call {kernel_3d_name(target, degree)}")
            finish_fn_3d(target, output, degree)
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        xof_setup_2d(target, output, degree)
        output.append(f"call {kernel_2d_name(target, degree)}")
        finish_fn_2d(target, output, degree)
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        xof_setup_2d(target, output, degree)
        output.append(f"call {kernel_2d_name(target, degree)}")
        finish_fn_2d(target, output, degree)
    else:
        raise NotImplementedError
    output.append(target.ret())


def emit_prelude(target, output):
    # output.append(".intel_syntax noprefix")
    pass


def emit_sse2(target, output):
    target = replace(target, extension=SSE2)
    kernel_2d(target, output, 1)
    compress(target, output)
    xof_fn(target, output, 1, xor=False)
    xof_fn(target, output, 1, xor=True)
    output.append(".balign 16")
    output.append("PBLENDW_0x33_MASK:")
    output.append(".long 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000")
    output.append("PBLENDW_0xCC_MASK:")
    output.append(".long 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF")
    output.append("PBLENDW_0x3F_MASK:")
    output.append(".long 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000")
    output.append("PBLENDW_0xC0_MASK:")
    output.append(".long 0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF")


def emit_sse41(target, output):
    target = replace(target, extension=SSE41)
    kernel_2d(target, output, 1)
    compress(target, output)
    xof_fn(target, output, 1, xor=False)
    xof_fn(target, output, 1, xor=True)


def emit_avx2(target, output):
    target = replace(target, extension=AVX2)
    kernel_2d(target, output, 2)
    xof_fn(target, output, 2, xor=False)
    xof_fn(target, output, 2, xor=True)


def emit_avx512(target, output):
    target = replace(target, extension=AVX512)

    # degree 1
    kernel_2d(target, output, 1)
    compress(target, output)
    xof_fn(target, output, 1, xor=False)
    xof_fn(target, output, 1, xor=True)

    # degree 2
    kernel_2d(target, output, 2)
    xof_fn(target, output, 2, xor=False)
    xof_fn(target, output, 2, xor=True)

    # degree 4
    kernel_2d(target, output, 4)
    xof_fn(target, output, 4, xor=False)
    xof_fn(target, output, 4, xor=True)

    # degree 16
    kernel_3d(target, output, 16)
    xof_fn(target, output, 16, xor=False)
    xof_fn(target, output, 16, xor=True)


def emit_footer(target, output):
    output.append(".balign 16")
    output.append("BLAKE3_IV:")
    output.append("BLAKE3_IV_0:")
    output.append(".long 0x6A09E667")
    output.append("BLAKE3_IV_1:")
    output.append(".long 0xBB67AE85")
    output.append("BLAKE3_IV_2:")
    output.append(".long 0x3C6EF372")
    output.append("BLAKE3_IV_3:")
    output.append(".long 0xA54FF53A")

    output.append(".balign 16")
    output.append("ROT16:")
    output.append(".byte  2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13")
    output.append("ROT8:")
    output.append(".byte  1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12")

    output.append(".balign 64")
    output.append("INCREMENT_2D:")
    output.append(".quad  0, 0, 1, 0, 2, 0, 3, 0")
    output.append("INCREMENT_3D:")
    output.append(".quad  0, 1, 2, 3, 4, 5, 6, 7")

    output.append(".balign 64")
    output.append("EVEN_INDEXES:")
    output.append(".long 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30")
    output.append("ODD_INDEXES:")
    output.append(".long 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31")


def format(output):
    print("# This file is generated by asm.py. Don't edit this file directly.")
    for item in output:
        if ":" in item or item[0] == ".":
            print(item)
        else:
            print(" " * 8 + item)


def main():
    target = Target(os=LINUX, arch=X86_64, extension=None)
    output = []

    emit_prelude(target, output)
    emit_sse2(target, output)
    emit_sse41(target, output)
    emit_avx2(target, output)
    emit_avx512(target, output)
    emit_footer(target, output)

    format(output)


if __name__ == "__main__":
    main()
