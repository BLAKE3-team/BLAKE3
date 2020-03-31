use std::env;

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

fn is_ci() -> bool {
    defined("BLAKE3_CI")
}

fn warn(warning: &str) {
    assert!(!warning.contains("\n"));
    println!("cargo:warning={}", warning);
    if is_ci() {
        println!("cargo:warning=Warnings in CI are treated as errors. Build failed.");
        std::process::exit(1);
    }
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

fn c_compiler_exists_and_supports_avx512() -> bool {
    let build = new_build();
    let flags_checked;
    let support_result: Result<bool, _> = if is_windows_msvc() {
        flags_checked = "/arch:AVX512";
        build.is_flag_supported("/arch:AVX512")
    } else {
        // Check for both of the flags we use. If -mavx512f works, then -mavx512vl
        // will probably always work too, but we might as well be thorough.
        flags_checked = "-mavx512f and -mavx512vl";
        match build.is_flag_supported("-mavx512f") {
            Ok(true) => build.is_flag_supported("-mavx512vl"),
            false_or_error => false_or_error,
        }
    };
    match support_result {
        Ok(true) => true,
        Ok(false) => {
            warn(&format!(
                "The C compiler {:?} does not support {}.",
                build.get_compiler().path(),
                flags_checked,
            ));
            false
        }
        Err(e) => {
            println!("{:?}", e);
            warn(&format!(
                "No C compiler {:?} detected.",
                build.get_compiler().path()
            ));
            false
        }
    }
}

fn build_x86_pure() {
    // A pure Rust build, so nothing to compile here. Enable the Rust SSE4.1
    // and AVX2 intrinsics builds. Do not enable AVX-512 in general.
    println!("cargo:rustc-cfg=blake3_sse41_rust");
    println!("cargo:rustc-cfg=blake3_avx2_rust");
}

fn build_x86_intrinsics() {
    // Enable the Rust SSE4.1 and AVX2 intrinsics builds, and also the C
    // AVX-512 intrinsics build. This is required on 32-bit x86 targets, since
    // the assembly implementations don't support those.
    assert!(c_compiler_exists_and_supports_avx512());
    assert!(!is_pure());
    println!("cargo:rustc-cfg=blake3_sse41_rust");
    println!("cargo:rustc-cfg=blake3_avx2_rust");
    println!("cargo:rustc-cfg=blake3_avx512_ffi");
    let mut build = new_build();
    build.file("c/blake3_avx512.c");
    if is_windows_msvc() {
        build.flag("/arch:AVX512");
    } else {
        build.flag("-mavx512f");
        build.flag("-mavx512vl");
    }
    if is_windows_gnu() {
        // Workaround for https://gcc.gnu.org/bugzilla/show_bug.cgi?id=65782.
        build.flag("-fno-asynchronous-unwind-tables");
    }
    build.compile("blake3_ffi");
}

fn build_x86_asm() {
    // Enable the assembly builds for SSE4.1, AVX2, and AVX-512. This is the
    // preferred build configuration by default, but it only supports x86_64.
    assert!(is_x86_64());
    assert!(c_compiler_exists_and_supports_avx512());
    assert!(!is_pure());
    assert!(!should_prefer_intrinsics());
    println!("cargo:rustc-cfg=blake3_sse41_ffi");
    println!("cargo:rustc-cfg=blake3_avx2_ffi");
    println!("cargo:rustc-cfg=blake3_avx512_ffi");
    let mut build = new_build();
    if is_windows_msvc() {
        build.file("c/blake3_sse41_x86-64_windows_msvc.asm");
        build.file("c/blake3_avx2_x86-64_windows_msvc.asm");
        build.file("c/blake3_avx512_x86-64_windows_msvc.asm");
    } else if is_windows_gnu() {
        build.file("c/blake3_sse41_x86-64_windows_gnu.S");
        build.file("c/blake3_avx2_x86-64_windows_gnu.S");
        build.file("c/blake3_avx512_x86-64_windows_gnu.S");
    } else {
        // All non-Windows implementations are assumed to support
        // Linux-style assembly. These files do contain a small
        // explicit workaround for macOS also.
        build.file("c/blake3_sse41_x86-64_unix.S");
        build.file("c/blake3_avx2_x86-64_unix.S");
        build.file("c/blake3_avx512_x86-64_unix.S");
    }
    build.compile("blake3_ffi");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if is_pure() && is_neon() {
        panic!("It doesn't make sense to enable both \"pure\" and \"neon\".");
    }

    if is_x86_64() || is_x86_32() {
        if is_pure() {
            build_x86_pure();
        } else if !c_compiler_exists_and_supports_avx512() {
            warn("Falling back to the pure Rust implementation, with reduced performance.");
            build_x86_pure();
        } else if is_x86_32() || should_prefer_intrinsics() {
            build_x86_intrinsics();
        } else {
            build_x86_asm();
        }
    }

    if is_neon() {
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
