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

fn new_build() -> cc::Build {
    let mut build = cc::Build::new();
    if !is_windows_msvc() {
        build.flag("-std=c11");
    }
    build
}

const WINDOWS_MSVC_ERROR: &str = r#"
The "c_avx512" feature is enabled, but your version of the MSVC C compiler does
not support the "/arch:AVX512" flag. If you are building the "b3sum" or
"bao_bin" crates, you can disable AVX-512 with Cargo's "--no-default-features"
flag. (Note that this also disables other default features like Rayon-based
multithreading, which you can re-enable with "--features=rayon".) Other crates
might or might not support this workaround.
"#;

const GNU_ERROR: &str = r#"
The "c_avx512" feature is enabled, but your C compiler does not support the
"-mavx512f" flag. If you are building the "b3sum" or "bao_bin" crates, you can
disable AVX-512 with Cargo's "--no-default-features" flag. (Note that this also
disables other default features like Rayon-based multithreading, which you can
re-enable with "--features=rayon".) Other crates might or might not support
this workaround.
"#;

fn check_for_avx512_compiler_support(build: &cc::Build) {
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
    // "c_avx512' is a no-op for non-x86_64 targets. It also participates in
    // dynamic CPU feature detection, so it's generally safe to enable.
    // However, it probably won't build in some older environments without
    // AVX-512 support in the C compiler, and it's disabled by default for that
    // reason.
    if defined("CARGO_FEATURE_C_AVX512") && is_x86_64() {
        let mut build = new_build();
        check_for_avx512_compiler_support(&build);
        build.file("c/blake3_avx512.c");
        if is_windows_msvc() {
            // Note that a lot of versions of MSVC don't support /arch:AVX512,
            // and they'll discard it with a warning, hopefully leading to a
            // build error.
            build.flag("/arch:AVX512");
        } else {
            build.flag("-mavx512f");
            build.flag("-mavx512vl");
        }
        build.compile("blake3_avx512");
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
