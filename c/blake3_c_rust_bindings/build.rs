use std::env;

fn defined(var: &str) -> bool {
    env::var_os(var).is_some()
}

fn target_components() -> Vec<String> {
    let target = env::var("TARGET").unwrap();
    target.split("-").map(|s| s.to_string()).collect()
}

// This is the full current list of x86 targets supported by Rustc. The C
// dispatch code uses
//   #if defined(__x86_64__) || defined(__i386__) || defined(_M_IX86) || defined(_M_X64)
// so this needs to be somewhat broad to match. These bindings are mainly for
// testing, so it's not the end of the world if this misses some obscure *86
// platform somehow.
fn is_x86() -> bool {
    target_components()[0] == "x86_64"
        || target_components()[0] == "i386"
        || target_components()[0] == "i586"
        || target_components()[0] == "i686"
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut base_build = new_build();
    base_build.file("../blake3.c");
    base_build.file("../blake3_dispatch.c");
    base_build.file("../blake3_portable.c");
    base_build.compile("blake3_c_base");

    if is_x86() {
        let mut sse41_build = new_build();
        sse41_build.file("../blake3_sse41.c");
        if is_windows_msvc() {
            // /arch:SSE2 is the default on x86 and undefined on x86_64:
            // https://docs.microsoft.com/en-us/cpp/build/reference/arch-x86
            // It also includes SSE4.1 intrisincs:
            // https://stackoverflow.com/a/32183222/823869
        } else {
            sse41_build.flag("-msse4.1");
        }
        sse41_build.compile("blake3_c_sse41");

        let mut avx2_build = new_build();
        avx2_build.file("../blake3_avx2.c");
        if is_windows_msvc() {
            avx2_build.flag("/arch:AVX2");
        } else {
            avx2_build.flag("-mavx2");
        }
        avx2_build.compile("blake3_c_avx2");

        let mut avx512_build = new_build();
        avx512_build.file("../blake3_avx512.c");
        if is_windows_msvc() {
            avx512_build.flag("/arch:AVX512");
        } else {
            avx512_build.flag("-mavx512f");
            avx512_build.flag("-mavx512vl");
        }
        avx512_build.compile("blake3_c_avx512");
    }

    // We only build NEON code here if 1) it's requested and 2) the root crate
    // is not already building it. The only time this will really happen is if
    // you build this crate by hand with the "neon" feature for some reason.
    if defined("CARGO_FEATURE_NEON") {
        let mut neon_build = new_build();
        neon_build.file("../blake3_neon.c");
        // ARMv7 platforms that support NEON generally need the following
        // flags. AArch64 supports NEON by default and does not support -mpfu.
        if is_armv7() {
            neon_build.flag("-mfpu=neon-vfpv4");
            neon_build.flag("-mfloat-abi=hard");
        }
        neon_build.compile("blake3_c_neon");
    }

    // The `cc` crate does not automatically emit rerun-if directives for the
    // environment variables it supports, in particular for $CC. We expect to
    // do a lot of benchmarking across different compilers, so we explicitly
    // add the variables that we're likely to need.
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");

    // Ditto for source files, though these shouldn't change as often.
    for file in std::fs::read_dir("..")? {
        println!(
            "cargo:rerun-if-changed={}",
            file?.path().to_str().expect("utf-8")
        );
    }

    Ok(())
}
