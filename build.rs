use std::env;

fn defined(var: &str) -> bool {
    env::var_os(var).is_some()
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

fn is_armv7() -> bool {
    target_components()[0] == "armv7"
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

fn is_windows_gnu() -> bool {
    // Some targets are only two components long, so check in steps.
    target_components()[1] == "pc"
        && target_components()[2] == "windows"
        && target_components()[3] == "gnu"
}

fn new_build() -> cc::Build {
    let mut build = cc::Build::new();
    if !is_windows_msvc() {
        build.flag("-std=c11");
    }
    build
}

const WINDOWS_MSVC_ERROR: &str = r#"
The "c" feature is enabled, but your version of the MSVC C compiler does not
support the "/arch:AVX512" flag. If you are building the "b3sum" or "bao_bin"
crates, you can disable AVX-512 with Cargo's "--no-default-features" flag.
(Note that this also disables other default features like Rayon-based
multithreading, which you can re-enable with "--features=rayon".) Other crates
might or might not support this workaround.
"#;

const GNU_ERROR: &str = r#"
The "c" feature is enabled, but your C compiler does not support the
"-mavx512f" flag. If you are building the "b3sum" or "bao_bin" crates, you can
disable AVX-512 with Cargo's "--no-default-features" flag. (Note that this also
disables other default features like Rayon-based multithreading, which you can
re-enable with "--features=rayon".) Other crates might or might not support
this workaround.
"#;

fn check_for_avx512_compiler_support() {
    let build = new_build();
    if is_windows_msvc() {
        if !build.is_flag_supported("/arch:AVX512").unwrap() {
            eprintln!("{}", WINDOWS_MSVC_ERROR.trim());
            std::process::exit(1);
        }
    } else {
        if !build.is_flag_supported("-mavx512f").unwrap() {
            eprintln!("{}", GNU_ERROR.trim());
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if defined("CARGO_FEATURE_C") {
        check_for_avx512_compiler_support();
        if is_x86_64() && !defined("CARGO_FEATURE_C_PREFER_INTRINSICS") {
            // On 64-bit, use the assembly implementations, unless the
            // "c_prefer_intrinsics" feature is enabled.
            if is_windows_msvc() {
                let mut build = new_build();
                build.file("c/blake3_sse41_x86-64_windows_msvc.asm");
                build.file("c/blake3_avx2_x86-64_windows_msvc.asm");
                build.file("c/blake3_avx512_x86-64_windows_msvc.asm");
                build.compile("blake3_asm");
            } else if is_windows_gnu() {
                let mut build = new_build();
                build.file("c/blake3_sse41_x86-64_windows_gnu.S");
                build.file("c/blake3_avx2_x86-64_windows_gnu.S");
                build.file("c/blake3_avx512_x86-64_windows_gnu.S");
                build.compile("blake3_asm");
            } else {
                // All non-Windows implementations are assumed to support
                // Linux-style assembly. These files do contain a small
                // explicit workaround for macOS also.
                let mut build = new_build();
                build.file("c/blake3_sse41_x86-64_unix.S");
                build.file("c/blake3_avx2_x86-64_unix.S");
                build.file("c/blake3_avx512_x86-64_unix.S");
                build.compile("blake3_asm");
            }
        } else if is_x86_64() || is_x86_32() {
            // Assembly implementations are only for 64-bit. On 32-bit, or if
            // the "c_prefer_intrinsics" feature is enabled, use the
            // intrinsics-based C implementations. These each need to be
            // compiled separately, with the corresponding instruction set
            // extension explicitly enabled in the compiler.

            let mut sse41_build = new_build();
            sse41_build.file("c/blake3_sse41.c");
            if is_windows_msvc() {
                // /arch:SSE2 is the default on x86 and undefined on x86_64:
                // https://docs.microsoft.com/en-us/cpp/build/reference/arch-x86
                // It also includes SSE4.1 intrisincs:
                // https://stackoverflow.com/a/32183222/823869
            } else {
                sse41_build.flag("-msse4.1");
            }
            sse41_build.compile("blake3_sse41");

            let mut avx2_build = new_build();
            avx2_build.file("c/blake3_avx2.c");
            if is_windows_msvc() {
                avx2_build.flag("/arch:AVX2");
            } else {
                avx2_build.flag("-mavx2");
            }
            avx2_build.compile("blake3_avx2");

            let mut avx512_build = new_build();
            avx512_build.file("c/blake3_avx512.c");
            if is_windows_msvc() {
                // Note that a lot of versions of MSVC don't support /arch:AVX512,
                // and they'll discard it with a warning, hopefully leading to a
                // build error.
                avx512_build.flag("/arch:AVX512");
            } else {
                avx512_build.flag("-mavx512f");
                avx512_build.flag("-mavx512vl");
            }
            avx512_build.compile("blake3_avx512");
        } else {
            // Currently no effect for non-x86 platforms.
        }
    }

    if defined("CARGO_FEATURE_C_NEON") {
        let mut build = new_build();
        // Note that blake3_neon.c normally depends on the blake3_portable.c
        // for the single-instance compression function, but we expose
        // portable.rs over FFI instead. See c_neon.rs.
        build.file("c/blake3_neon.c");
        // ARMv7 platforms that support NEON generally need the following
        // flags. AArch64 supports NEON by default and does not support -mpfu.
        if is_armv7() {
            build.flag("-mfpu=neon-vfpv4");
            build.flag("-mfloat-abi=hard");
        }
        build.compile("blake3_neon");
    }

    // The `cc` crate does not automatically emit rerun-if directives for the
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
