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

.section .text

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
blake3_guts_riscv64gcv_hash_blocks:
        // t0 = full_blocks = (input_len + 1024 - 64) / 1024
        // NOTE: If the final chunk is short, this could be 1 less than the
        // total number of chunks, in which case this setup code and the kernel
        // will leave a CV word undisturbed in each of v0-v7.
        addi t0, a1, 1024 - 64
        srli t0, t0, 10
        vsetvli zero, t0, e32, m1, ta, ma
        li t1, 1024
        // NOTE: These loads could be misaligned. As far as I know, the Linux
        // RISC-V ABI allows misaligned loads and stores. If we need to support
        // an environment that doesn't allow them (or where they're
        // unacceptably slow), we could add a fallback here.
        vlsseg8e32.v v16, (a0), t1
        addi a0, a0, 32
        vlsseg8e32.v v24, (a0), t1
        addi a0, a0, 32
        addi a1, a1, -64
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
        // broadcast the block length
        // TODO: handle partial blocks
        li t1, 64
        vmv.v.x v14, t1
        // broadcast the flags
        vmv.v.x v15, a4
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
        ret

// a0: input
// a1: input_len
// a2: key
// a3: counter
// a4: flags
// a5: aligned+transposed output
.global blake3_guts_riscv64gcv_hash_chunks
blake3_guts_riscv64gcv_hash_chunks:
        // keep the original num_chunks = (input_len+1023)/1024 in a6
        addi a6, a1, 1023
        srli a6, a6, 10
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
