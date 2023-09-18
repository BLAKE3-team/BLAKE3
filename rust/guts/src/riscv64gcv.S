// This implementation currently assumes riscv64gcv_zbb_zvbb. Zvbb in particular ("Vector
// Bit-manipulation used in Cryptography") is a bleeding-edge extension that was only frozen a few
// weeks ago at the time I'm writing this comment. Compiling and testing this code currently
// requires quite a lot of effort, including building Clang from master and building QEMU from a
// custom branch. Please don't expect this code to be usable on real hardware for some time.

#define IV0 0x6A09E667
#define IV1 0xBB67AE85
#define IV2 0x3C6EF372
#define IV3 0xA54FF53A

// NOTE: Keep this in sync with the same constant in Rust.
#define MAX_SIMD_DEGREE 16

#define TRANSPOSED_STRIDE_BYTES 2 * MAX_SIMD_DEGREE * 4

#define CHUNK_START         (1 << 0)
#define CHUNK_END           (1 << 1)
#define PARENT              (1 << 2)
#define ROOT                (1 << 3)
#define KEYED_HASH          (1 << 4)
#define DERIVE_KEY_CONTEXT  (1 << 5)
#define DERIVE_KEY_MATERIAL (1 << 6)

.section .text

.p2align 2
IV_VEC:
        .word    IV0, IV1, IV2, IV3
ROR1:
        .word    3, 0, 1, 2
ROR2:
        .word    2, 3, 0, 1
ROR3:
        .word    1, 2, 3, 0

# The bottom half of the load permutation is tweaked to account for the fact that
# we hold the second row fixed during diagonalization.
MSG_LOAD:
        .short   0, 2, 4, 6, 1, 3, 5, 7, 14, 8, 10, 12, 15, 9, 11, 13

# The message permutation as given in the in the BLAKE3 spec would be the correct
# permutation to use if the load order above was 0, 1, 2, 3... However, since
# we're using a tricky load order, we need to adjust the permutation accordingly.
# The following Python snippet reproduces the permutation we're using here:
#
#     load_order = [0, 2, 4, 6, 1, 3, 5, 7, 14, 8, 10, 12, 15, 9, 11, 13]
#     original_permutation = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]
#     retargeted_permutation = [load_order.index(x) for x in original_permutation]
#     shuffled_permutation = [retargeted_permutation[i] for i in load_order]
#     print(shuffled_permutation)
MSG_PERMUTE:
        .short   1, 5, 7, 2, 3, 10, 0, 15, 12, 4, 11, 13, 9, 14, 6, 8

// a0: block (zero-padded to 64 bytes)
// a1: block_len
// a2: cv_bytes
// a3: counter
// a4: flags
// a5: out_ptr
.global blake3_guts_riscv64gcv_compress
blake3_guts_riscv64gcv_compress:
        // Load the message load and message permutation indexes.
        vsetivli zero, 16, e16, m2, ta, ma
        la t0, MSG_LOAD
        vle16.v v8, (t0)
        la t0, MSG_PERMUTE
        vle16.v v10, (t0)
        // Load the CV into v0-v1.
        vsetivli zero, 16, e8, m1, ta, ma
        vle8.v v0, (a2)
        addi a2, a2, 16
        vle8.v v1, (a2)
        // Set LMUL=4 and load the message block temporarily into scratch
        // space. Apply the MSG_LOAD permutation, and then move the permuted
        // message words into v4-v7.
        // TODO: Do this with less register movement?
        li t0, 64
        vsetvli zero, t0, e8, m4, ta, ma
        vle8.v v20, (a0)
        vsetivli zero, 16, e32, m4, ta, ma
        vrgatherei16.vv v16, v20, v8
        vsetivli zero, 4, e32, m4, ta, ma
        vslidedown.vi v20, v16, 4
        vslidedown.vi v24, v16, 8
        vslidedown.vi v28, v16, 12
        vsetivli zero, 4, e32, m1, ta, ma
        vmv.v.v v4, v16
        vmv.v.v v5, v20
        vmv.v.v v6, v24
        vmv.v.v v7, v28
        // Load the diagonalization gather indexes.
        la t0, ROR1
        vle32.v v12, (t0)
        la t0, ROR2
        vle32.v v13, (t0)
        la t0, ROR3
        vle32.v v14, (t0)
        // Load the IV words.
        la t0, IV_VEC
        vle32.v v2, (t0)
        // Load the counter, block_len, and flags.
        vsetivli zero, 4, e32, m1, ta, ma
        vslide1down.vx v3, v3, a3
        srli a3, a3, 32
        vslide1down.vx v3, v3, a3
        vslide1down.vx v3, v3, a1
        vslide1down.vx v3, v3, a4
        li t0, 7  // round counter
blake3_guts_riscv64gcv_compress_round_loop:
        vadd.vv  v0, v0, v4
        vadd.vv  v0, v0, v1
        vxor.vv  v3, v3, v0
        vror.vi  v3, v3, 16
        vadd.vv  v2, v2, v3
        vxor.vv  v1, v1, v2
        vror.vi  v1, v1, 12
        vadd.vv  v0, v0, v5
        vadd.vv  v0, v0, v1
        vxor.vv  v3, v3, v0
        vror.vi  v3, v3, 8
        vadd.vv  v2, v2, v3
        vxor.vv  v1, v1, v2
        vror.vi  v1, v1, 7
        // Gathers can't overlap a source register, so use v20/v22/v23 in place
        // of v0/v2/v3 for this section.
        vrgather.vv v20, v0, v12
        vrgather.vv v23, v3, v13
        vrgather.vv v22, v2, v14
        vadd.vv  v20, v20, v6
        vadd.vv  v20, v20, v1
        vxor.vv  v23, v23, v20
        vror.vi  v23, v23, 16
        vadd.vv  v22, v22, v23
        vxor.vv  v1, v1, v22
        vror.vi  v1, v1, 12
        vadd.vv  v20, v20, v7
        vadd.vv  v20, v20, v1
        vxor.vv  v23, v23, v20
        vror.vi  v23, v23, 8
        vadd.vv  v22, v22, v23
        vxor.vv  v1, v1, v22
        vror.vi  v1, v1, 7
        vrgather.vv v0, v20, v14
        vrgather.vv v3, v23, v13
        vrgather.vv v2, v22, v12
        addi t0, t0, -1
        beqz t0, blake3_guts_riscv64gcv_compress_end
        // Shuffle message words.
        // TODO: Find a way to do this without so much movement?
        vmv.v.v v16, v4
        vmv.v.v v20, v5
        vmv.v.v v24, v6
        vmv.v.v v28, v7
        vsetivli zero, 16, e32, m4, ta, ma
        vslideup.vi v16, v20, 4
        vslideup.vi v16, v24, 8
        vslideup.vi v16, v28, 12
        vrgatherei16.vv v28, v16, v10
        vsetivli zero, 4, e32, m4, ta, ma
        vslidedown.vi v16, v28, 4
        vslidedown.vi v20, v28, 8
        vslidedown.vi v24, v28, 12
        vsetivli zero, 4, e32, m1, ta, ma
        vmv.v.v v4, v28
        vmv.v.v v5, v16
        vmv.v.v v6, v20
        vmv.v.v v7, v24
        j blake3_guts_riscv64gcv_compress_round_loop
blake3_guts_riscv64gcv_compress_end:
        vxor.vv v0, v0, v2
        vxor.vv v1, v1, v3
        vsetivli zero, 16, e8, m1, ta, ma
        vse8.v v0, (a5)
        addi a5, a5, 16
        vse8.v v1, (a5)
        ret


.global blake3_guts_riscv64gcv_degree
blake3_guts_riscv64gcv_degree:
        csrr t0, vlenb
        srli t0, t0, 2
        li t1, MAX_SIMD_DEGREE
        minu a0, t0, t1
        ret

// clobbers: t0
blake3_guts_riscv64gcv_kernel:
        li t0, IV0
        vmv.v.x v8, t0
        li t0, IV1
        vmv.v.x v9, t0
        li t0, IV2
        vmv.v.x v10, t0
        li t0, IV3
        vmv.v.x v11, t0
        vadd.vv  v0, v0, v16
        vadd.vv  v1, v1, v18
        vadd.vv  v2, v2, v20
        vadd.vv  v3, v3, v22
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v17
        vadd.vv  v1, v1, v19
        vadd.vv  v2, v2, v21
        vadd.vv  v3, v3, v23
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v24
        vadd.vv  v1, v1, v26
        vadd.vv  v2, v2, v28
        vadd.vv  v3, v3, v30
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v25
        vadd.vv  v1, v1, v27
        vadd.vv  v2, v2, v29
        vadd.vv  v3, v3, v31
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v18
        vadd.vv  v1, v1, v19
        vadd.vv  v2, v2, v23
        vadd.vv  v3, v3, v20
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v22
        vadd.vv  v1, v1, v26
        vadd.vv  v2, v2, v16
        vadd.vv  v3, v3, v29
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v17
        vadd.vv  v1, v1, v28
        vadd.vv  v2, v2, v25
        vadd.vv  v3, v3, v31
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v27
        vadd.vv  v1, v1, v21
        vadd.vv  v2, v2, v30
        vadd.vv  v3, v3, v24
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v19
        vadd.vv  v1, v1, v26
        vadd.vv  v2, v2, v29
        vadd.vv  v3, v3, v23
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v20
        vadd.vv  v1, v1, v28
        vadd.vv  v2, v2, v18
        vadd.vv  v3, v3, v30
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v22
        vadd.vv  v1, v1, v25
        vadd.vv  v2, v2, v27
        vadd.vv  v3, v3, v24
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v21
        vadd.vv  v1, v1, v16
        vadd.vv  v2, v2, v31
        vadd.vv  v3, v3, v17
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v26
        vadd.vv  v1, v1, v28
        vadd.vv  v2, v2, v30
        vadd.vv  v3, v3, v29
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v23
        vadd.vv  v1, v1, v25
        vadd.vv  v2, v2, v19
        vadd.vv  v3, v3, v31
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v20
        vadd.vv  v1, v1, v27
        vadd.vv  v2, v2, v21
        vadd.vv  v3, v3, v17
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v16
        vadd.vv  v1, v1, v18
        vadd.vv  v2, v2, v24
        vadd.vv  v3, v3, v22
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v28
        vadd.vv  v1, v1, v25
        vadd.vv  v2, v2, v31
        vadd.vv  v3, v3, v30
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v29
        vadd.vv  v1, v1, v27
        vadd.vv  v2, v2, v26
        vadd.vv  v3, v3, v24
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v23
        vadd.vv  v1, v1, v21
        vadd.vv  v2, v2, v16
        vadd.vv  v3, v3, v22
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v18
        vadd.vv  v1, v1, v19
        vadd.vv  v2, v2, v17
        vadd.vv  v3, v3, v20
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v25
        vadd.vv  v1, v1, v27
        vadd.vv  v2, v2, v24
        vadd.vv  v3, v3, v31
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v30
        vadd.vv  v1, v1, v21
        vadd.vv  v2, v2, v28
        vadd.vv  v3, v3, v17
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v29
        vadd.vv  v1, v1, v16
        vadd.vv  v2, v2, v18
        vadd.vv  v3, v3, v20
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v19
        vadd.vv  v1, v1, v26
        vadd.vv  v2, v2, v22
        vadd.vv  v3, v3, v23
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        vadd.vv  v0, v0, v27
        vadd.vv  v1, v1, v21
        vadd.vv  v2, v2, v17
        vadd.vv  v3, v3, v24
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vror.vi  v15, v15, 16
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 12
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vadd.vv  v0, v0, v31
        vadd.vv  v1, v1, v16
        vadd.vv  v2, v2, v25
        vadd.vv  v3, v3, v22
        vadd.vv  v0, v0, v4
        vadd.vv  v1, v1, v5
        vadd.vv  v2, v2, v6
        vadd.vv  v3, v3, v7
        vxor.vv  v12, v12, v0
        vxor.vv  v13, v13, v1
        vxor.vv  v14, v14, v2
        vxor.vv  v15, v15, v3
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vror.vi  v15, v15, 8
        vadd.vv  v8, v8, v12
        vadd.vv  v9, v9, v13
        vadd.vv  v10, v10, v14
        vadd.vv  v11, v11, v15
        vxor.vv  v4, v4, v8
        vxor.vv  v5, v5, v9
        vxor.vv  v6, v6, v10
        vxor.vv  v7, v7, v11
        vror.vi  v4, v4, 7
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vadd.vv  v0, v0, v30
        vadd.vv  v1, v1, v18
        vadd.vv  v2, v2, v19
        vadd.vv  v3, v3, v23
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 16
        vror.vi  v12, v12, 16
        vror.vi  v13, v13, 16
        vror.vi  v14, v14, 16
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 12
        vror.vi  v6, v6, 12
        vror.vi  v7, v7, 12
        vror.vi  v4, v4, 12
        vadd.vv  v0, v0, v26
        vadd.vv  v1, v1, v28
        vadd.vv  v2, v2, v20
        vadd.vv  v3, v3, v29
        vadd.vv  v0, v0, v5
        vadd.vv  v1, v1, v6
        vadd.vv  v2, v2, v7
        vadd.vv  v3, v3, v4
        vxor.vv  v15, v15, v0
        vxor.vv  v12, v12, v1
        vxor.vv  v13, v13, v2
        vxor.vv  v14, v14, v3
        vror.vi  v15, v15, 8
        vror.vi  v12, v12, 8
        vror.vi  v13, v13, 8
        vror.vi  v14, v14, 8
        vadd.vv  v10, v10, v15
        vadd.vv  v11, v11, v12
        vadd.vv  v8, v8, v13
        vadd.vv  v9, v9, v14
        vxor.vv  v5, v5, v10
        vxor.vv  v6, v6, v11
        vxor.vv  v7, v7, v8
        vxor.vv  v4, v4, v9
        vror.vi  v5, v5, 7
        vror.vi  v6, v6, 7
        vror.vi  v7, v7, 7
        vror.vi  v4, v4, 7
        ret

// arguments from hash_chunks
// a0: input [adjusted by 64]
// a1: input_len [adjusted by -64]
// a2: key [unused]
// a3: counter
// a4: flags
// a5: aligned+transposed output [unused]
// a6: total chunks [unused]
// a7: remaining_bytes_in_last_chunk
blake3_guts_riscv64gcv_hash_blocks:
        // t0 := full_blocks := (input_len + 1024 - 64) / 1024
        addi t0, a1, 1024 - 64
        srli t0, t0, 10
        // Load and transpose full message blocks. These are "strided segment
        // loads". Each vlsseg8e32 instruction transposes 8 words from multiple
        // message blocks into 8 registers, so we need two vlsseg8e32
        // instructions (with the second offset by 32 bytes) to load full
        // 64-byte blocks. The 1024-byte stride represents the spacing between
        // two blocks in the same position in adjacent chunks.
        // NOTE: If the final chunk is short, this could be 1 less than the
        // total number of chunks, in which case this setup code and the kernel
        // will leave a CV word undisturbed in each of v0-v7.
        // NOTE: These loads could be misaligned. As far as I know, the Linux
        // RISC-V ABI allows misaligned loads and stores. If we need to support
        // an environment that doesn't allow them (or where they're
        // unacceptably slow), we could add a fallback here.
        vsetvli zero, t0, e32, m1, ta, ma
        li t1, 1024
        addi t2, a0, 32
        vlsseg8e32.v v16, (a0), t1
        vlsseg8e32.v v24, (t2), t1
        // If remaining_bytes_in_last_chunk in 1..=63, there's a partial block
        // at the end. Handle it out-of-line. If we take this branch, it will
        // increment t0 by 1.
        addi t1, a7, -1
        li t2, 63
        bltu t1, t2, handle_partial_block
partial_block_finished:
        // load the counter
        vsetvli zero, t0, e64, m2, ta, ma
        vmv.v.x v8, a3
        vid.v v10
        vadd.vv v8, v8, v10
        // This is the mode setting that the kernel will use. If the final
        // chunk is short, this iteration might have fewer blocks than an
        // earlier iteration, so we need the tail undisturbed (tu).
        vsetvli zero, t0, e32, m1, tu, ma
        vncvt.x.x.w v12, v8
        li t1, 32
        vnsrl.wx v13, v8, t1
        // Broadcast the block length, then overwrite the last block's length
        // to be ((min(64, remaining_bytes_in_last_chunk) - 1) % 64) + 1. That
        // is: 64 if remaining_bytes_in_last_chunk >= 64
        //     else 64 if remaining_bytes_in_last_chunk is 0
        //     else remaining_bytes_in_last_chunk
        li t1, 64
        vmv.v.x v14, t1
        minu t1, t1, a7
        addi t1, t1, -1
        andi t1, t1, 63
        addi t1, t1, 1
        vslide1down.vx v14, v14, t1
        // Broadcast the flags, then set CHUNK_END in the last block's flags if
        // remaining_bytes_in_last_chunk is in 1..=64.
        vmv.v.x v15, a4
        addi t1, a7, -1
        sltiu t1, t1, 64
        slli t1, t1, 1  // CHUNK_END = 2
        or t1, t1, a4
        vslide1down.vx v15, v15, t1
        // execute the kernel
        mv t6, ra
        call blake3_guts_riscv64gcv_kernel
        mv ra, t6
        // xor the two halves of the state
        vxor.vv v0, v0, v8
        vxor.vv v1, v1, v9
        vxor.vv v2, v2, v10
        vxor.vv v3, v3, v11
        vxor.vv v4, v4, v12
        vxor.vv v5, v5, v13
        vxor.vv v6, v6, v14
        vxor.vv v7, v7, v15
        // Increment the input pointer, input_len, and
        // remaining_bytes_in_last_chunk (which cannot go below zero).
        addi a0, a0, 64
        addi a1, a1, -64
        addi a7, a7, -64
        max a7, a7, zero
        ret
handle_partial_block:
        // The minimum VLEN is 128 bits, so we're guaranteed to be able to fit
        // the block in v8-v11 with LMUL=4. Clear 64 zero bytes before the
        // load, to make sure the partial block is zero-padded.
        li t1, 64
        vsetvli zero, t1, e8, m4, ta, ma
        vmv.v.i v8, 0
        add t2, a0, a1
        sub t2, t2, a7
        vsetvli zero, a7, e8, m4, ta, ma
        vle8.v v8, (t2)
        // If VLEN is longer than 128 bits (16 bytes), then half or all of the
        // block bytes will be in v8. Make sure they're split evenly across
        // v8-v11.
        csrr t1, vlenb
        li t2, 64
        bltu t1, t2, vlenb_less_than_64
        vsetivli zero, 8, e32, m1, ta, ma
        vslidedown.vi v9, v8, 8
vlenb_less_than_64:
        li t2, 32
        bltu t1, t2, vlenb_less_than_32
        vsetivli zero, 4, e32, m1, ta, ma
        vmv.v.v v10, v9
        vslidedown.vi v11, v9, 4
        vslidedown.vi v9, v8, 4
vlenb_less_than_32:
        // Shift each of the words of the padded partial block to the end of
        // the corresponding message vector. t0 was previously the number of
        // full blocks. Now we increment it, so that it's the number of all
        // blocks (both full and partial).
        mv t1, t0
        addi t0, t0, 1
        // Set vl to at least 4, because v8-v11 each have 4 message words.
        // Setting vl shorter will make vslide1down clobber those words.
        li t2, 4
        maxu t2, t0, t2
        vsetvli zero, t2, e32, m1, ta, ma
        vslideup.vx v16, v8, t1
        vslide1down.vx v8, v8, zero
        vslideup.vx v17, v8, t1
        vslide1down.vx v8, v8, zero
        vslideup.vx v18, v8, t1
        vslide1down.vx v8, v8, zero
        vslideup.vx v19, v8, t1
        vslideup.vx v20, v9, t1
        vslide1down.vx v9, v9, zero
        vslideup.vx v21, v9, t1
        vslide1down.vx v9, v9, zero
        vslideup.vx v22, v9, t1
        vslide1down.vx v9, v9, zero
        vslideup.vx v23, v9, t1
        vslideup.vx v24, v10, t1
        vslide1down.vx v10, v10, zero
        vslideup.vx v25, v10, t1
        vslide1down.vx v10, v10, zero
        vslideup.vx v26, v10, t1
        vslide1down.vx v10, v10, zero
        vslideup.vx v27, v10, t1
        vslideup.vx v28, v11, t1
        vslide1down.vx v11, v11, zero
        vslideup.vx v29, v11, t1
        vslide1down.vx v11, v11, zero
        vslideup.vx v30, v11, t1
        vslide1down.vx v11, v11, zero
        vslideup.vx v31, v11, t1
        j partial_block_finished

// a0: input
// a1: input_len
// a2: key
// a3: counter
// a4: flags
// a5: aligned+transposed output
.global blake3_guts_riscv64gcv_hash_chunks
blake3_guts_riscv64gcv_hash_chunks:
        // Save the original num_chunks = (input_len+1023)/1024 in a6.
        addi a6, a1, 1023
        srli a6, a6, 10
        // Track the bytes remaining in the last chunk in a7. The initial value
        // of this is ((input_len - 1) % 1024) + 1. (The input to this function
        // is never empty.) It decrements by 64 with each call to
        // blake3_guts_riscv64gcv_hash_chunks, but not below 0.
        addi a7, a1, -1
        andi a7, a7, 1023
        addi a7, a7, 1
        // broadcast the key to v0-7
        vsetvli zero, a6, e32, m1, ta, ma
        lw t0, 0(a2)
        vmv.v.x v0, t0
        lw t0, 4(a2)
        vmv.v.x v1, t0
        lw t0, 8(a2)
        vmv.v.x v2, t0
        lw t0, 12(a2)
        vmv.v.x v3, t0
        lw t0, 16(a2)
        vmv.v.x v4, t0
        lw t0, 20(a2)
        vmv.v.x v5, t0
        lw t0, 24(a2)
        vmv.v.x v6, t0
        lw t0, 28(a2)
        vmv.v.x v7, t0
        // sixteen blocks (TODO: partial chunks)
        // Note that hash_blocks increments the input pointer and decrements
        // the input length.
        mv t5, ra
        ori a4, a4, 1    // set CHUNK_START
        call blake3_guts_riscv64gcv_hash_blocks
        andi a4, a4, -2  // unset CHUNK_START
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        call blake3_guts_riscv64gcv_hash_blocks
        ori a4, a4, 2    // set CHUNK_END
        call blake3_guts_riscv64gcv_hash_blocks
        mv ra, t5
        // If the final chunk is short, we need to set vl back to the total
        // number of chunks.
        vsetvli zero, a6, e32, m1, ta, ma
        // write aligned+transposed outputs with a stride of 2*MAX_SIMD_DEGREE words
        vse32.v v0, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v1, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v2, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v3, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v4, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v5, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v6, (a5)
        addi a5, a5, TRANSPOSED_STRIDE_BYTES
        vse32.v v7, (a5)
        ret

// a0: aligned+transposed input
// a1: num_parents
// a2: key
// a3: flags
// a4: out pointer
.global blake3_guts_riscv64gcv_hash_parents
blake3_guts_riscv64gcv_hash_parents:
        // load the transposed CVs and split alternating words into the low and
        // high halves of the input vectors
        vsetvli zero, a1, e32, m1, ta, ma
        vlseg2e32.v v16, (a0)
        vmv.v.v v24, v17
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v17, (a0)
        vmv.v.v v25, v18
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v18, (a0)
        vmv.v.v v26, v19
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v19, (a0)
        vmv.v.v v27, v20
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v20, (a0)
        vmv.v.v v28, v21
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v21, (a0)
        vmv.v.v v29, v22
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v22, (a0)
        vmv.v.v v30, v23
        addi a0, a0, TRANSPOSED_STRIDE_BYTES
        vlseg2e32.v v14, (a0)  // use v14-15 as scratch space to avoid overwriting v24
        vmv.v.v v23, v14
        vmv.v.v v31, v15
        // broadcast the key to v0-7
        lw t0, 0(a2)
        vmv.v.x v0, t0
        lw t0, 4(a2)
        vmv.v.x v1, t0
        lw t0, 8(a2)
        vmv.v.x v2, t0
        lw t0, 12(a2)
        vmv.v.x v3, t0
        lw t0, 16(a2)
        vmv.v.x v4, t0
        lw t0, 20(a2)
        vmv.v.x v5, t0
        lw t0, 24(a2)
        vmv.v.x v6, t0
        lw t0, 28(a2)
        vmv.v.x v7, t0
        // zero the counter
        vmv.v.i v12, 0
        vmv.v.i v13, 0
        // broadcast the block length
        li t0, 64
        vmv.v.x v14, t0
        // broadcast the flags
        vmv.v.x v15, a3

        // execute the kernel
        mv t6, ra
        call blake3_guts_riscv64gcv_kernel
        mv ra, t6

        // xor the two halves of the state
        vxor.vv v0, v0, v8
        vxor.vv v1, v1, v9
        vxor.vv v2, v2, v10
        vxor.vv v3, v3, v11
        vxor.vv v4, v4, v12
        vxor.vv v5, v5, v13
        vxor.vv v6, v6, v14
        vxor.vv v7, v7, v15
        // write aligned+transposed outputs with a stride of 2*MAX_SIMD_DEGREE words
        vse32.v v0, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v1, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v2, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v3, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v4, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v5, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v6, (a4)
        addi a4, a4, TRANSPOSED_STRIDE_BYTES
        vse32.v v7, (a4)
        ret

// a0: 64 zero-padded block bytes
// a1: block_len
// a2: cv
// a3: counter
// a4: flags
// a5: out_ptr
// a6: out_len
blake3_guts_riscv64gcv_xof_inner:
        // t1 := total_blocks := (out_len + 63) / 64
        addi t1, a6, 63
        srli t1, t1, 6
        // t2 := full_blocks := out_len / 64
        srli t2, a6, 6
        // broadcast the CV to v0-7
        vsetvli zero, t1, e32, m1, ta, ma
        lw t3, 0(a2)
        vmv.v.x v0, t3
        lw t3, 4(a2)
        vmv.v.x v1, t3
        lw t3, 8(a2)
        vmv.v.x v2, t3
        lw t3, 12(a2)
        vmv.v.x v3, t3
        lw t3, 16(a2)
        vmv.v.x v4, t3
        lw t3, 20(a2)
        vmv.v.x v5, t3
        lw t3, 24(a2)
        vmv.v.x v6, t3
        lw t3, 28(a2)
        vmv.v.x v7, t3
        // broadcast the block_words to v16-31
        lw t3, 0(a0)
        vmv.v.x v16, t3
        lw t3, 4(a0)
        vmv.v.x v17, t3
        lw t3, 8(a0)
        vmv.v.x v18, t3
        lw t3, 12(a0)
        vmv.v.x v19, t3
        lw t3, 16(a0)
        vmv.v.x v20, t3
        lw t3, 20(a0)
        vmv.v.x v21, t3
        lw t3, 24(a0)
        vmv.v.x v22, t3
        lw t3, 28(a0)
        vmv.v.x v23, t3
        lw t3, 32(a0)
        vmv.v.x v24, t3
        lw t3, 36(a0)
        vmv.v.x v25, t3
        lw t3, 40(a0)
        vmv.v.x v26, t3
        lw t3, 44(a0)
        vmv.v.x v27, t3
        lw t3, 48(a0)
        vmv.v.x v28, t3
        lw t3, 52(a0)
        vmv.v.x v29, t3
        lw t3, 56(a0)
        vmv.v.x v30, t3
        lw t3, 60(a0)
        vmv.v.x v31, t3
        // load the counter
        vsetvli zero, t1, e64, m2, ta, ma
        vmv.v.x v8, a3
        vid.v v10
        vadd.vv v8, v8, v10
        vsetvli zero, t1, e32, m1, ta, ma
        vncvt.x.x.w v12, v8
        li t3, 32
        vnsrl.wx v13, v8, t3
        // broadcast the block length
        vmv.v.x v14, a1
        // broadcast the flags
        vmv.v.x v15, a4

        // execute the kernel
        mv t6, ra
        call blake3_guts_riscv64gcv_kernel
        mv ra, t6

        // reload the CV, this time into v16-23
        lw t3, 0(a2)
        vmv.v.x v16, t3
        lw t3, 4(a2)
        vmv.v.x v17, t3
        lw t3, 8(a2)
        vmv.v.x v18, t3
        lw t3, 12(a2)
        vmv.v.x v19, t3
        lw t3, 16(a2)
        vmv.v.x v20, t3
        lw t3, 20(a2)
        vmv.v.x v21, t3
        lw t3, 24(a2)
        vmv.v.x v22, t3
        lw t3, 28(a2)
        vmv.v.x v23, t3
        // xor the two halves of the state and feed-forward the CV
        vxor.vv v0, v0, v8
        vxor.vv v1, v1, v9
        vxor.vv v2, v2, v10
        vxor.vv v3, v3, v11
        vxor.vv v4, v4, v12
        vxor.vv v5, v5, v13
        vxor.vv v6, v6, v14
        vxor.vv v7, v7, v15
        vxor.vv v8, v8, v16
        vxor.vv v9, v9, v17
        vxor.vv v10, v10, v18
        vxor.vv v11, v11, v19
        vxor.vv v12, v12, v20
        vxor.vv v13, v13, v21
        vxor.vv v14, v14, v22
        vxor.vv v15, v15, v23
        ret

// a0: 64 zero-padded block bytes
// a1: block_len
// a2: cv
// a3: counter
// a4: flags
// a5: out_ptr
// a6: out_len
.global blake3_guts_riscv64gcv_xof
blake3_guts_riscv64gcv_xof:
        mv t5, ra
        call blake3_guts_riscv64gcv_xof_inner
        mv ra, t5

        // t1 is now total_blocks, and t2 is full_blocks. Set vl to t2 and the
        // tail policy to undisturbed. We'll handle full blocks with segmented
        // stores, and then we'll use a separate branch for a partial final
        // block, if any.
        vsetvli zero, t2, e32, m1, tu, ma

        // Transpose and store full output blocks. These are "strided segment
        // stores". Each vssseg8e32 instruction transposes 8 words from
        // adjacent registers into 32 bytes of contiguous output, so we need
        // two vssseg8e32 instructions to store full 64-byte blocks. We offset
        // the second store by 32 bytes and use a 64-byte stride.
        // NOTE: These stores might be misaligned.
        li t0, 64
        addi t3, a5, 32
        vssseg8e32.v v0, (a5), t0
        vssseg8e32.v v8, (t3), t0

        // If full_blocks != partial_blocks, we need to handle the final
        // partial block. Otherwise, we're done.
        bne t1, t2, blake3_guts_riscv64gcv_xof_partial_block
        ret
blake3_guts_riscv64gcv_xof_partial_block:
        // Collect groups of 4 words in v0, v4, v8, and v12.
        vsetivli zero, 4, e32, m1, ta, ma
        vslidedown.vx v0, v0, t2
        vslidedown.vx v1, v1, t2
        vslideup.vi v0, v1, 1
        vslidedown.vx v2, v2, t2
        vslideup.vi v0, v2, 2
        vslidedown.vx v3, v3, t2
        vslideup.vi v0, v3, 3
        vslidedown.vx v4, v4, t2
        vslidedown.vx v5, v5, t2
        vslideup.vi v4, v5, 1
        vslidedown.vx v6, v6, t2
        vslideup.vi v4, v6, 2
        vslidedown.vx v7, v7, t2
        vslideup.vi v4, v7, 3
        vslidedown.vx v8, v8, t2
        vslidedown.vx v9, v9, t2
        vslideup.vi v8, v9, 1
        vslidedown.vx v10, v10, t2
        vslideup.vi v8, v10, 2
        vslidedown.vx v11, v11, t2
        vslideup.vi v8, v11, 3
        vslidedown.vx v12, v12, t2
        vslidedown.vx v13, v13, t2
        vslideup.vi v12, v13, 1
        vslidedown.vx v14, v14, t2
        vslideup.vi v12, v14, 2
        vslidedown.vx v15, v15, t2
        vslideup.vi v12, v15, 3
        // Use LMUL=4 to guarantee that one vector register group can hold 16
        // words, and collect all 16 words in the v0 group.
        vsetivli zero, 16, e32, m4, ta, ma
        vslideup.vi v0, v4, 4
        vslideup.vi v0, v8, 8
        vslideup.vi v0, v12, 12
        // Switch to bytes and write the output.
        andi t3, a6, 63
        add a5, a5, a6
        sub a5, a5, t3
        vsetvli zero, t3, e8, m4, ta, ma
        vse8.v v0, (a5)
        ret

// a0: 64 zero-padded block bytes
// a1: block_len
// a2: cv
// a3: counter
// a4: flags
// a5: out_ptr
// a6: out_len
.global blake3_guts_riscv64gcv_xof_xor
blake3_guts_riscv64gcv_xof_xor:
        mv t5, ra
        call blake3_guts_riscv64gcv_xof_inner
        mv ra, t5

        // t1 is now total_blocks, and t2 is full_blocks. Set vl to t2 and the
        // tail policy to undisturbed. We'll handle full blocks with segmented
        // stores, and then we'll use a separate branch for a partial final
        // block, if any.
        vsetvli zero, t2, e32, m1, tu, ma

        // Do a transposed load of the caller's buffer, xor that with the state
        // words, and do a transposed store. These are "strided segment"
        // loads/stores. Each vlsseg8e32/vssseg8e32 instruction works with
        // groups of 8 words or 32 bytes, so we need pairs of these
        // instructions to handle full 64-byte blocks. We offset the second by
        // 32 bytes and use a 64-byte stride.
        // NOTE: These accesses might be misaligned.
        li t0, 64
        addi t3, a5, 32
        vlsseg8e32.v v16, (a5), t0
        vlsseg8e32.v v24, (t3), t0
        vxor.vv v0, v0, v16
        vxor.vv v1, v1, v17
        vxor.vv v2, v2, v18
        vxor.vv v3, v3, v19
        vxor.vv v4, v4, v20
        vxor.vv v5, v5, v21
        vxor.vv v6, v6, v22
        vxor.vv v7, v7, v23
        vxor.vv v8, v8, v24
        vxor.vv v9, v9, v25
        vxor.vv v10, v10, v26
        vxor.vv v11, v11, v27
        vxor.vv v12, v12, v28
        vxor.vv v13, v13, v29
        vxor.vv v14, v14, v30
        vxor.vv v15, v15, v31
        vssseg8e32.v v0, (a5), t0
        vssseg8e32.v v8, (t3), t0

        // If full_blocks != partial_blocks, we need to handle the final
        // partial block. Otherwise, we're done.
        bne t1, t2, blake3_guts_riscv64gcv_xof_xor_partial_block
        ret
blake3_guts_riscv64gcv_xof_xor_partial_block:
        // Collect groups of 4 words in v0, v4, v8, and v12.
        vsetivli zero, 4, e32, m1, ta, ma
        vslidedown.vx v0, v0, t2
        vslidedown.vx v1, v1, t2
        vslideup.vi v0, v1, 1
        vslidedown.vx v2, v2, t2
        vslideup.vi v0, v2, 2
        vslidedown.vx v3, v3, t2
        vslideup.vi v0, v3, 3
        vslidedown.vx v4, v4, t2
        vslidedown.vx v5, v5, t2
        vslideup.vi v4, v5, 1
        vslidedown.vx v6, v6, t2
        vslideup.vi v4, v6, 2
        vslidedown.vx v7, v7, t2
        vslideup.vi v4, v7, 3
        vslidedown.vx v8, v8, t2
        vslidedown.vx v9, v9, t2
        vslideup.vi v8, v9, 1
        vslidedown.vx v10, v10, t2
        vslideup.vi v8, v10, 2
        vslidedown.vx v11, v11, t2
        vslideup.vi v8, v11, 3
        vslidedown.vx v12, v12, t2
        vslidedown.vx v13, v13, t2
        vslideup.vi v12, v13, 1
        vslidedown.vx v14, v14, t2
        vslideup.vi v12, v14, 2
        vslidedown.vx v15, v15, t2
        vslideup.vi v12, v15, 3
        // Use LMUL=4 to guarantee that one vector register group can hold 16
        // words, and collect all 16 words in the v0 group.
        vsetivli zero, 16, e32, m4, ta, ma
        vslideup.vi v0, v4, 4
        vslideup.vi v0, v8, 8
        vslideup.vi v0, v12, 12
        // Switch to bytes and read/xor/write the output.
        andi t3, a6, 63
        add a5, a5, a6
        sub a5, a5, t3
        vsetvli zero, t3, e8, m4, ta, ma
        vle8.v v4, (a5)
        vxor.vv v0, v0, v4
        vse8.v v0, (a5)
        ret

// a0: input_ptr
// a1: input_len
// a2: key
// a3: counter
// a4: out_ptr
.global blake3_guts_riscv64gcv_universal_hash
blake3_guts_riscv64gcv_universal_hash:
        // t0 := full_blocks := input_len / 64
        // TODO: handle the partial block at the end
        srli t0, a1, 6
        // Load the counter.
        vsetvli zero, t0, e64, m2, ta, ma
        vmv.v.x v8, a3
        vid.v v10
        vadd.vv v8, v8, v10
        vsetvli zero, t0, e32, m1, ta, ma
        vncvt.x.x.w v12, v8
        li t1, 32
        vnsrl.wx v13, v8, t1
        // Load and transpose full message blocks. These are "strided segment
        // loads". Each vlsseg8e32 instruction transposes 8 words from multiple
        // message blocks into 8 registers, so we need two vlsseg8e32
        // instructions (with the second offset by 32 bytes) to load full
        // 64-byte blocks. The 64-byte stride equals the block size, because in
        // this case (unlike hash_blocks) the blocks are adjacent.
        // NOTE: These loads could be misaligned. As far as I know, the Linux
        // RISC-V ABI allows misaligned loads and stores. If we need to support
        // an environment that doesn't allow them (or where they're
        // unacceptably slow), we could add a fallback here.
        li t1, 64
        addi t2, a0, 32
        vlsseg8e32.v v16, (a0), t1
        vlsseg8e32.v v24, (t2), t1
        // Broadcast the key to v0-7.
        lw t0, 0(a2)
        vmv.v.x v0, t0
        lw t0, 4(a2)
        vmv.v.x v1, t0
        lw t0, 8(a2)
        vmv.v.x v2, t0
        lw t0, 12(a2)
        vmv.v.x v3, t0
        lw t0, 16(a2)
        vmv.v.x v4, t0
        lw t0, 20(a2)
        vmv.v.x v5, t0
        lw t0, 24(a2)
        vmv.v.x v6, t0
        lw t0, 28(a2)
        vmv.v.x v7, t0
        // Broadcast the block length.
        li t1, 64
        vmv.v.x v14, t1
        // Broadcast the flags.
        li t1, CHUNK_START | CHUNK_END | ROOT | KEYED_HASH
        vmv.v.x v15, t1
        // Execute the kernel.
        mv t6, ra
        call blake3_guts_riscv64gcv_kernel
        mv ra, t6
        // XOR the first four words. The rest are dropped.
        vxor.vv v0, v0, v8
        vxor.vv v1, v1, v9
        vxor.vv v2, v2, v10
        vxor.vv v3, v3, v11
        // XOR-reduce each vector.
        vmv.v.i v4, 0
        vredxor.vs v0, v0, v4
        vredxor.vs v1, v1, v4
        vredxor.vs v2, v2, v4
        vredxor.vs v3, v3, v4
        // Write the output.
        vmv.x.s t0, v0
        sw t0,  0(a4)
        vmv.x.s t0, v1
        sw t0,  4(a4)
        vmv.x.s t0, v2
        sw t0,  8(a4)
        vmv.x.s t0, v3
        sw t0, 12(a4)

        ret
