#! /usr/bin/env python3

# Generate asm!

from dataclasses import dataclass, replace

X86_64 = "x86_64"
AVX512 = "avx512"
AVX2 = "avx2"
SSE41 = "sse41"
SSE2 = "sse2"
LINUX = "linux"


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
        elif degree == 2:
            output.append(f"vpaddd ymm{dest}, ymm{dest}, ymm{src}")
        elif degree == 4:
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
        elif degree == 2:
            output.append(f"vpxord ymm{dest}, ymm{dest}, ymm{src}")
        elif degree == 4:
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
        elif degree == 2:
            output.append(f"vprord ymm{reg}, ymm{reg}, {bits}")
        elif degree == 4:
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


# See the comments above kernel2d().
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


# See the comments above kernel2d().
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


# See the comments above kernel2d().
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


def kernel2d_name(target, degree):
    return f"blake3_{target.extension}_kernel2d_{degree}"


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
def kernel2d(target, output, degree):
    label = kernel2d_name(target, degree)
    output.append(f"{label}:")
    # vpshufb indexes
    if target.extension == SSE41:
        output.append(f"movaps xmm14, xmmword ptr [ROT8+rip]")
        output.append(f"movaps xmm15, xmmword ptr [ROT16+rip]")
    if target.extension == AVX2:
        output.append(f"vbroadcasti128 ymm14, xmmword ptr [ROT16+rip]")
        output.append(f"vbroadcasti128 ymm15, xmmword ptr [ROT8+rip]")
    if target.extension == AVX512:
        if degree == 4:
            output.append(f"mov {target.scratch32(0)}, 43690")
            output.append(f"kmovw k3, {target.scratch32(0)}")
            output.append(f"mov {target.scratch32(0)}, 34952")
            output.append(f"kmovw k4, {target.scratch32(0)}")
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
    output.append(f"call {kernel2d_name(target, 1)}")
    compress_finish(target, output)
    output.append(target.ret())


def xof_setup2d(target, output, degree):
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


def xof_stream_finish2d(target, output, degree):
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
            output.append(
                f"vextracti128 xmmword ptr [{target.arg64(5)} + 4 * 16], ymm0, 1"
            )
            output.append(
                f"vextracti128 xmmword ptr [{target.arg64(5)} + 5 * 16], ymm1, 1"
            )
            output.append(
                f"vextracti128 xmmword ptr [{target.arg64(5)} + 6 * 16], ymm2, 1"
            )
            output.append(
                f"vextracti128 xmmword ptr [{target.arg64(5)} + 7 * 16], ymm3, 1"
            )
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


def xof_stream(target, output, degree):
    label = f"blake3_{target.extension}_xof_stream_{degree}"
    output.append(f".global {label}")
    output.append(f"{label}:")
    if target.extension == AVX512:
        if degree in (1, 2, 4):
            xof_setup2d(target, output, degree)
            output.append(f"call {kernel2d_name(target, degree)}")
            xof_stream_finish2d(target, output, degree)
        else:
            raise NotImplementedError
    elif target.extension == AVX2:
        assert degree == 2
        xof_setup2d(target, output, degree)
        output.append(f"call {kernel2d_name(target, degree)}")
        xof_stream_finish2d(target, output, degree)
    elif target.extension in (SSE41, SSE2):
        assert degree == 1
        xof_setup2d(target, output, degree)
        output.append(f"call {kernel2d_name(target, degree)}")
        xof_stream_finish2d(target, output, degree)
    else:
        raise NotImplementedError
    output.append(target.ret())


def emit_prelude(target, output):
    # output.append(".intel_syntax noprefix")
    pass


def emit_sse2(target, output):
    target = replace(target, extension=SSE2)
    kernel2d(target, output, 1)
    compress(target, output)
    xof_stream(target, output, 1)
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
    kernel2d(target, output, 1)
    compress(target, output)
    xof_stream(target, output, 1)


def emit_avx2(target, output):
    target = replace(target, extension=AVX2)
    kernel2d(target, output, 2)
    xof_stream(target, output, 2)


def emit_avx512(target, output):
    target = replace(target, extension=AVX512)
    kernel2d(target, output, 1)
    kernel2d(target, output, 2)
    kernel2d(target, output, 4)
    compress(target, output)
    xof_stream(target, output, 1)
    xof_stream(target, output, 2)
    xof_stream(target, output, 4)


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
