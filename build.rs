use std::env;

fn defined(var: &str) -> bool {
    env::var_os(var).is_some()
}

fn target_arch() -> String {
    let target = env::var("TARGET").unwrap();
    let target_components: Vec<&str> = target.split("-").collect();
    target_components[0].to_string()
}

fn target_os() -> String {
    let target = env::var("TARGET").unwrap();
    let target_components: Vec<&str> = target.split("-").collect();
    target_components[2].to_string()
}

fn is_windows() -> bool {
    target_os() == "windows"
}

fn is_x86_64() -> bool {
    target_arch() == "x86_64"
}

fn is_armv7() -> bool {
    target_arch() == "armv7"
}

fn new_build() -> cc::Build {
    let mut build = cc::Build::new();
    if !is_windows() {
        build.flag("-std=c11");
    }
    build
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // "c_avx512' is a no-op for non-x86_64 targets. It also participates in
    // dynamic CPU feature detection, so it's generally safe to enable.
    // However, it probably won't build in some older environments without
    // AVX-512 support in the C compiler, and it's disabled by default for that
    // reason.
    if defined("CARGO_FEATURE_C_AVX512") && is_x86_64() {
        let mut build = new_build();
        build.file("c/blake3_avx512.c");
        if is_windows() {
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
