use std::env;

fn defined(var: &str) -> bool {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var_os(var).is_some()
}

fn is_pure() -> bool {
    defined("CARGO_FEATURE_PURE")
}

fn target_components() -> Vec<String> {
    let target = env::var("TARGET").unwrap();
    target.split("-").map(|s| s.to_string()).collect()
}

fn is_riscv64gc() -> bool {
    target_components()[0] == "riscv64gc"
}

fn new_build() -> cc::Build {
    let build = cc::Build::new();
    build
}

fn build_riscv_rva23u64_assembly() {
    println!("cargo:rustc-cfg=blake3_riscv_rva23u64_ffi");
    let mut build = new_build();
    let asm_path = "src/riscv_rva23u64.S";
    build.file(asm_path);
    build.flag("--target=riscv64");
    build.flag("-march=rv64gcv_zbb_zvbb1p0");
    build.flag("-menable-experimental-extensions");
    build.compile("blake3_riscv_rva23u64_assembly");
    println!("cargo:rerun-if-changed={asm_path}");
}

fn main() {
    // TODO: This implementation assumes some bleeding-edge extensions, and it should probably be
    // gated by a Cargo feature.
    if is_riscv64gc() && !is_pure() {
        build_riscv_rva23u64_assembly();
    }

    // The `cc` crate doesn't automatically emit rerun-if directives for the
    // environment variables it supports, in particular for $CC. We expect to
    // do a lot of benchmarking across different compilers, so we explicitly
    // add the variables that we're likely to need.
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");

    // Ditto for source files, though these shouldn't change as often.
    for file in std::fs::read_dir("../../c").unwrap() {
        println!(
            "cargo:rerun-if-changed={}",
            file.unwrap().path().to_str().expect("utf-8")
        );
    }
}
