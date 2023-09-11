use std::env;
use std::path::{Path, PathBuf};

fn defined(var: &str) -> bool {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var_os(var).is_some()
}

fn is_pure() -> bool {
    defined("CARGO_FEATURE_PURE")
}

fn should_prefer_intrinsics() -> bool {
    defined("CARGO_FEATURE_PREFER_INTRINSICS")
}

fn is_neon() -> bool {
    defined("CARGO_FEATURE_NEON")
}

fn is_no_neon() -> bool {
    defined("CARGO_FEATURE_NO_NEON")
}

fn target_components() -> Vec<String> {
    let target = env::var("TARGET").unwrap();
    target.split("-").map(|s| s.to_string()).collect()
}

fn is_x86_64() -> bool {
    target_components()[0] == "x86_64"
}

fn is_x86_32() -> bool {
    let arch = &target_components()[0];
    arch == "i386" || arch == "i586" || arch == "i686"
}

fn is_apple() -> bool {
    let vendor = &target_components()[1];
    vendor == "apple"
}

fn is_arm() -> bool {
    is_armv7() || is_aarch64() || target_components()[0] == "arm"
}

fn is_aarch64() -> bool {
    target_components()[0] == "aarch64"
}

fn is_armv7() -> bool {
    target_components()[0] == "armv7"
}

fn out_dir() -> PathBuf {
    std::env::var("OUT_DIR").unwrap().into()
}

// Windows targets may be using the MSVC toolchain or the GNU toolchain. The
// right compiler flags to use depend on the toolchain. (And we don't want to
// use flag_if_supported, because we don't want features to be silently
// disabled by old compilers.)
fn is_windows_msvc() -> bool {
    // Some targets are only two components long, so check in steps.
    target_components()[1] == "pc"
        && target_components()[2] == "windows"
        && target_components()[3] == "msvc"
}

fn new_build() -> cc::Build {
    let mut build = cc::Build::new();
    if !is_windows_msvc() {
        build.flag("-std=c11");
    }
    build
}

fn build_asm<T: AsRef<Path>>(path: T, name: &str) {
    let mut asm = std::fs::read_to_string(path.as_ref()).unwrap();
    if asm.starts_with(".intel_syntax") {
        asm = asm[asm.find('\n').unwrap_or(asm.len())..].to_string()
    }
    if is_apple() {
        // Apple doesn't support .section .rodata/.section .text, and instead
        // has its own directives. Let's just rewrite them.
        asm = asm.replace(".section .rodata", ".static_data");
        asm = asm.replace(".section .text", ".text");
    }
    // Global_asm uses { and } for format string stuff, but they're used by the
    // x86_64 asm for AVX512 stuff.
    asm = asm.replace("{", "{{");
    asm = asm.replace("}", "}}");

    std::fs::write(out_dir().join(name), asm).unwrap();
}

fn build_sse2_sse41_avx2_rust_asm() {
    // No C code to compile here, global_asm will be used to include the asm.
    // Set the cfg flags that enable the SSE2, SSE4.1, and AVX2 asm modules.
    // The regular Cargo build will compile them.
    println!("cargo:rustc-cfg=blake3_sse2_ffi");
    println!("cargo:rustc-cfg=blake3_sse41_ffi");
    println!("cargo:rustc-cfg=blake3_avx2_ffi");
    println!("cargo:rustc-cfg=blake3_sse2_asm");
    println!("cargo:rustc-cfg=blake3_sse41_asm");
    println!("cargo:rustc-cfg=blake3_avx2_asm");

    build_asm("c/blake3_sse2_x86-64_windows_gnu.S", "blake3_sse2.S");
    build_asm("c/blake3_sse41_x86-64_windows_gnu.S", "blake3_sse41.S");
    build_asm("c/blake3_avx2_x86-64_windows_gnu.S", "blake3_avx2.S");

}

fn build_avx512_rust_asm() {
    // No C code to compile here, global_asm will be used to include the asm.
    // Set the cfg flags that enable the AVX512 asm modules.
    // The regular Cargo build will compile them.
    println!("cargo:rustc-cfg=blake3_avx512_ffi");
    println!("cargo:rustc-cfg=blake3_avx512_asm");

    build_asm("c/blake3_avx512_x86-64_windows_gnu.S", "blake3_avx512.S");
}

fn build_sse2_sse41_avx2_rust_intrinsics() {
    // No C code to compile here. Set the cfg flags that enable the Rust SSE2,
    // SSE4.1, and AVX2 intrinsics modules. The regular Cargo build will compile
    // them.
    println!("cargo:rustc-cfg=blake3_sse2_rust");
    println!("cargo:rustc-cfg=blake3_sse41_rust");
    println!("cargo:rustc-cfg=blake3_avx2_rust");
}

fn build_neon_c_intrinsics() {
    let mut build = new_build();
    // Note that blake3_neon.c normally depends on the blake3_portable.c
    // for the single-instance compression function, but we expose
    // portable.rs over FFI instead. See ffi_neon.rs.
    build.file("c/blake3_neon.c");
    // ARMv7 platforms that support NEON generally need the following
    // flags. AArch64 supports NEON by default and does not support -mpfu.
    if is_armv7() {
        build.flag("-mfpu=neon-vfpv4");
        build.flag("-mfloat-abi=hard");
    }
    build.compile("blake3_neon");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if is_pure() && is_neon() {
        panic!("It doesn't make sense to enable both \"pure\" and \"neon\".");
    }

    if is_no_neon() && is_neon() {
        panic!("It doesn't make sense to enable both \"no_neon\" and \"neon\".");
    }

    if is_x86_64() || is_x86_32() {
        if is_x86_32() || is_pure() || should_prefer_intrinsics() {
            build_sse2_sse41_avx2_rust_intrinsics();
        } else {
            build_sse2_sse41_avx2_rust_asm();
        }

        if is_x86_32() || is_pure() || should_prefer_intrinsics() {
            // The binary will not include any AVX-512 code.
        } else {
            build_avx512_rust_asm();
        }
    }

    if (is_arm() && is_neon()) || (!is_no_neon() && !is_pure() && is_aarch64()) {
        println!("cargo:rustc-cfg=blake3_neon");
        build_neon_c_intrinsics();
    }

    // The `cc` crate doesn't automatically emit rerun-if directives for the
    // environment variables it supports, in particular for $CC. We expect to
    // do a lot of benchmarking across different compilers, so we explicitly
    // add the variables that we're likely to need.
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");

    // Ditto for source files, though these shouldn't change as often.
    for file in std::fs::read_dir("c")? {
        println!(
            "cargo:rerun-if-changed={}",
            file?.path().to_str().expect("utf-8")
        );
    }

    Ok(())
}
