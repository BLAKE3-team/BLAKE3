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
    # x86_64
    arch: str  # "x86_64"

    # sse2, sse41, avx2, avx512, neon
    extension: str  # "sse41"

    # unix, windows_msvc, windows_gnu
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


def add_row(t, o, dest, src):
    assert t.arch == X86_64
    if t.extension == AVX512:
        o.append(f"vpaddd xmm{dest}, xmm{dest}, xmm{src}")
    elif t.extension in (SSE41, SSE2):
        o.append(f"paddd xmm{dest}, xmm{src}")
    else:
        raise NotImplementedError


def xor_row(t, o, dest, src):
    assert t.arch == X86_64
    if t.extension == AVX512:
        o.append(f"vpxord xmm{dest}, xmm{dest}, xmm{src}")
    elif t.extension in (SSE41, SSE2):
        o.append(f"pxor xmm{dest}, xmm{src}")
    else:
        raise NotImplementedError


# This is the >>> operation in G, not state diagonalization or message permutation.
def bitrotate_row(t, o, reg, bits):
    assert t.arch == X86_64
    if t.extension == AVX512:
        o.append(f"vprord xmm{reg}, xmm{reg}, {bits}")
    elif t.extension == SSE41:
        if bits == 16:
            # xmm15 is initialized at the top of kernel_1.
            o.append(f"pshufb xmm{reg}, xmm15")
        elif bits == 8:
            # xmm14 is initialized at the top of kernel_1.
            o.append(f"pshufb xmm{reg}, xmm14")
        else:
            # Do two bitshifts, using register 11 as temp.
            o.append(f"movdqa xmm11, xmm{reg}")
            o.append(f"pslld xmm{reg}, {32 - bits}")
            o.append(f"psrld xmm11, {bits}")
            o.append(f"por xmm{reg}, xmm11")
    elif t.extension == SSE2:
        if bits == 16:
            o.append(f"pshuflw xmm{reg}, xmm{reg}, 0xB1")
            o.append(f"pshufhw xmm{reg}, xmm{reg}, 0xB1")
        else:
            # Do two bitshifts, using register 11 as temp.
            o.append(f"movdqa xmm11, xmm{reg}")
            o.append(f"pslld xmm{reg}, {32 - bits}")
            o.append(f"psrld xmm11, {bits}")
            o.append(f"por xmm{reg}, xmm11")
    else:
        raise NotImplementedError


def diagonalize_state_rows(t, o):
    if t.extension == AVX512:
        o.append("vpshufd xmm0, xmm0, 0x93")
        o.append("vpshufd xmm3, xmm3, 0x4E")
        o.append("vpshufd xmm2, xmm2, 0x39")
    elif t.extension in (SSE41, SSE2):
        o.append("pshufd xmm0, xmm0, 0x93")
        o.append("pshufd xmm3, xmm3, 0x4E")
        o.append("pshufd xmm2, xmm2, 0x39")
    else:
        raise NotImplementedError


def undiagonalize_state_rows(t, o):
    if t.extension == AVX512:
        o.append("vpshufd xmm0, xmm0, 0x39")
        o.append("vpshufd xmm3, xmm3, 0x4E")
        o.append("vpshufd xmm2, xmm2, 0x93")
    elif t.extension in (SSE41, SSE2):
        o.append("pshufd xmm0, xmm0, 0x39")
        o.append("pshufd xmm3, xmm3, 0x4E")
        o.append("pshufd xmm2, xmm2, 0x93")
    else:
        raise NotImplementedError


def permute_message_rows(t, o):
    if t.extension == AVX512:
        o.append("vshufps xmm8, xmm4, xmm5, 214")
        o.append("vpshufd xmm9, xmm4, 0x0F")
        o.append("vpshufd xmm4, xmm8, 0x39")
        o.append("vshufps xmm8, xmm6, xmm7, 250")
        o.append("vpblendd xmm9, xmm9, xmm8, 0xAA")
        o.append("vpunpcklqdq xmm8, xmm7, xmm5")
        o.append("vpblendd xmm8, xmm8, xmm6, 0x88")
        o.append("vpshufd xmm8, xmm8, 0x78")
        o.append("vpunpckhdq xmm5, xmm5, xmm7")
        o.append("vpunpckldq xmm6, xmm6, xmm5")
        o.append("vpshufd xmm7, xmm6, 0x1E")
        o.append("vmovdqa xmm5, xmm9")
        o.append("vmovdqa xmm6, xmm8")
    elif t.extension == SSE41:
        o.append("movdqa xmm8, xmm4")
        o.append("shufps xmm8, xmm5, 214")
        o.append("pshufd xmm9, xmm4, 0x0F")
        o.append("pshufd xmm4, xmm8, 0x39")
        o.append("movdqa xmm8, xmm6")
        o.append("shufps xmm8, xmm7, 250")
        o.append("pblendw xmm9, xmm8, 0xCC")
        o.append("movdqa xmm8, xmm7")
        o.append("punpcklqdq xmm8, xmm5")
        o.append("pblendw xmm8, xmm6, 0xC0")
        o.append("pshufd xmm8, xmm8, 0x78")
        o.append("punpckhdq xmm5, xmm7")
        o.append("punpckldq xmm6, xmm5")
        o.append("pshufd xmm7, xmm6, 0x1E")
        o.append("movdqa xmm5, xmm9")
        o.append("movdqa xmm6, xmm8")
    elif t.extension == SSE2:
        o.append("movdqa xmm8, xmm4")
        o.append("shufps xmm8, xmm5, 214")
        o.append("pshufd xmm9, xmm4, 0x0F")
        o.append("pshufd xmm4, xmm8, 0x39")
        o.append("movdqa xmm8, xmm6")
        o.append("shufps xmm8, xmm7, 250")
        o.append("pand xmm9, xmmword ptr [PBLENDW_0x33_MASK+rip]")
        o.append("pand xmm8, xmmword ptr [PBLENDW_0xCC_MASK+rip]")
        o.append("por xmm9, xmm8")
        o.append("movdqa xmm8, xmm7")
        o.append("punpcklqdq xmm8, xmm5")
        o.append("movdqa xmm10, xmm6")
        o.append("pand xmm8, xmmword ptr [PBLENDW_0x3F_MASK+rip]")
        o.append("pand xmm10, xmmword ptr [PBLENDW_0xC0_MASK+rip]")
        o.append("por xmm8, xmm10")
        o.append("pshufd xmm8, xmm8, 0x78")
        o.append("punpckhdq xmm5, xmm7")
        o.append("punpckldq xmm6, xmm5")
        o.append("pshufd xmm7, xmm6, 0x1E")
        o.append("movdqa xmm5, xmm9")
        o.append("movdqa xmm6, xmm8")
    else:
        raise NotImplementedError


def kernel_1(t, o):
    o.append(f"blake3_{t.extension}_kernel_1:")
    if t.extension == SSE41:
        o.append(f"movaps  xmm14, xmmword ptr [ROT8+rip]")
        o.append(f"movaps  xmm15, xmmword ptr [ROT16+rip]")
    for round_number in range(7):
        if round_number > 0:
            # Un-diagonalize and permute before each round except the first.
            # compress_finish() will also partially un-diagonalize.
            permute_message_rows(t, o)
        add_row(t, o, dest=0, src=4)
        add_row(t, o, dest=0, src=1)
        xor_row(t, o, dest=3, src=0)
        bitrotate_row(t, o, reg=3, bits=16)
        add_row(t, o, dest=2, src=3)
        xor_row(t, o, dest=1, src=2)
        bitrotate_row(t, o, reg=1, bits=12)
        add_row(t, o, dest=0, src=5)
        add_row(t, o, dest=0, src=1)
        xor_row(t, o, dest=3, src=0)
        bitrotate_row(t, o, reg=3, bits=8)
        add_row(t, o, dest=2, src=3)
        xor_row(t, o, dest=1, src=2)
        bitrotate_row(t, o, reg=1, bits=7)
        diagonalize_state_rows(t, o)
        add_row(t, o, dest=0, src=6)
        add_row(t, o, dest=0, src=1)
        xor_row(t, o, dest=3, src=0)
        bitrotate_row(t, o, reg=3, bits=16)
        add_row(t, o, dest=2, src=3)
        xor_row(t, o, dest=1, src=2)
        bitrotate_row(t, o, reg=1, bits=12)
        add_row(t, o, dest=0, src=7)
        add_row(t, o, dest=0, src=1)
        xor_row(t, o, dest=3, src=0)
        bitrotate_row(t, o, reg=3, bits=8)
        add_row(t, o, dest=2, src=3)
        xor_row(t, o, dest=1, src=2)
        bitrotate_row(t, o, reg=1, bits=7)
        undiagonalize_state_rows(t, o)
    # Xor the last two rows into the first two, but don't do the feed forward
    # here. That's only done in the XOF case.
    xor_row(t, o, dest=0, src=2)
    xor_row(t, o, dest=1, src=3)
    o.append(t.ret())


def compress_setup(t, o):
    if t.extension == AVX512:
        o.append(f"vmovdqu xmm0, xmmword ptr [{t.arg64(0)}]")
        o.append(f"vmovdqu xmm1, xmmword ptr [{t.arg64(0)}+0x10]")
        o.append(f"shl {t.arg64(4)}, 32")
        o.append(f"mov {t.arg32(3)}, {t.arg32(3)}")
        o.append(f"or {t.arg64(3)}, {t.arg64(4)}")
        o.append(f"vmovq xmm3, {t.arg64(2)}")
        o.append(f"vmovq xmm4, {t.arg64(3)}")
        o.append(f"vpunpcklqdq xmm3, xmm3, xmm4")
        o.append(f"vmovaps xmm2, xmmword ptr [BLAKE3_IV+rip]")
        o.append(f"vmovups xmm8, xmmword ptr [{t.arg64(1)}]")
        o.append(f"vmovups xmm9, xmmword ptr [{t.arg64(1)}+0x10]")
        o.append(f"vshufps xmm4, xmm8, xmm9, 136")
        o.append(f"vshufps xmm5, xmm8, xmm9, 221")
        o.append(f"vmovups xmm8, xmmword ptr [{t.arg64(1)}+0x20]")
        o.append(f"vmovups xmm9, xmmword ptr [{t.arg64(1)}+0x30]")
        o.append(f"vshufps xmm6, xmm8, xmm9, 136")
        o.append(f"vshufps xmm7, xmm8, xmm9, 221")
        o.append(f"vpshufd xmm6, xmm6, 0x93")
        o.append(f"vpshufd xmm7, xmm7, 0x93")
    elif t.extension in (SSE41, SSE2):
        o.append(f"movups  xmm0, xmmword ptr [{t.arg64(0)}]")
        o.append(f"movups  xmm1, xmmword ptr [{t.arg64(0)}+0x10]")
        o.append(f"movaps  xmm2, xmmword ptr [BLAKE3_IV+rip]")
        o.append(f"shl {t.arg64(4)}, 32")
        o.append(f"mov {t.arg32(3)}, {t.arg32(3)}")
        o.append(f"or {t.arg64(3)}, {t.arg64(4)}")
        o.append(f"vmovq xmm3, {t.arg64(2)}")
        o.append(f"vmovq xmm4, {t.arg64(3)}")
        o.append(f"punpcklqdq xmm3, xmm4")
        o.append(f"movups  xmm4, xmmword ptr [{t.arg64(1)}]")
        o.append(f"movups  xmm5, xmmword ptr [{t.arg64(1)}+0x10]")
        o.append(f"movaps  xmm8, xmm4")
        o.append(f"shufps  xmm4, xmm5, 136")
        o.append(f"shufps  xmm8, xmm5, 221")
        o.append(f"movaps  xmm5, xmm8")
        o.append(f"movups  xmm6, xmmword ptr [{t.arg64(1)}+0x20]")
        o.append(f"movups  xmm7, xmmword ptr [{t.arg64(1)}+0x30]")
        o.append(f"movaps  xmm8, xmm6")
        o.append(f"shufps  xmm6, xmm7, 136")
        o.append(f"pshufd  xmm6, xmm6, 0x93")
        o.append(f"shufps  xmm8, xmm7, 221")
        o.append(f"pshufd  xmm7, xmm8, 0x93")
    else:
        raise NotImplementedError


def compress_finish(t, o):
    if t.extension == AVX512:
        o.append(f"vmovdqu xmmword ptr [{t.arg64(0)}], xmm0")
        o.append(f"vmovdqu xmmword ptr [{t.arg64(0)}+0x10], xmm1")
    elif t.extension in (SSE41, SSE2):
        o.append(f"movups xmmword ptr [{t.arg64(0)}], xmm0")
        o.append(f"movups xmmword ptr [{t.arg64(0)}+0x10], xmm1")
    else:
        raise NotImplementedError


def compress(t, o):
    name = f"blake3_{t.extension}_compress"
    o.append(f".global {name}")
    o.append(f"{name}:")
    compress_setup(t, o)
    o.append(f"call blake3_{t.extension}_kernel_1")
    compress_finish(t, o)
    o.append(t.ret())


def emit_prelude(t, o):
    # o.append(".intel_syntax noprefix")
    pass


def emit_sse2(t, o):
    t = replace(t, extension=SSE2)
    kernel_1(t, o)
    compress(t, o)
    o.append(".balign 16")
    o.append("PBLENDW_0x33_MASK:")
    o.append(".long 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000")
    o.append("PBLENDW_0xCC_MASK:")
    o.append(".long 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF")
    o.append("PBLENDW_0x3F_MASK:")
    o.append(".long 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000")
    o.append("PBLENDW_0xC0_MASK:")
    o.append(".long 0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF")


def emit_sse41(t, o):
    t = replace(t, extension=SSE41)
    kernel_1(t, o)
    compress(t, o)
    o.append(".balign 16")
    o.append("ROT16:")
    o.append(".byte  2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13")
    o.append("ROT8:")
    o.append(".byte  1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12")


def emit_avx2(t, o):
    t = replace(t, extension=AVX2)


def emit_avx512(t, o):
    t = replace(t, extension=AVX512)
    kernel_1(t, o)
    compress(t, o)


def emit_footer(t, o):
    o.append(".balign 16")
    o.append("BLAKE3_IV:")
    o.append("BLAKE3_IV_0:")
    o.append(".long 0x6A09E667")
    o.append("BLAKE3_IV_1:")
    o.append(".long 0xBB67AE85")
    o.append("BLAKE3_IV_2:")
    o.append(".long 0x3C6EF372")
    o.append("BLAKE3_IV_3:")
    o.append(".long 0xA54FF53A")


def format(output):
    print("# DO NOT EDIT")
    print("# This file is generated by asm.py.")
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
