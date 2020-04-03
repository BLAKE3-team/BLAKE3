use anyhow::bail;
use std::path::Path;
use std::process::Command;

struct TestCommand {
    command: Command,
    repr: String,
}

impl TestCommand {
    fn new(args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::new_inner(args, None)
    }

    fn new_with_dir(args: impl IntoIterator<Item = impl AsRef<str>>, dir: &str) -> Self {
        Self::new_inner(args, Some(dir))
    }

    fn new_inner(args: impl IntoIterator<Item = impl AsRef<str>>, dir: Option<&str>) -> Self {
        let mut args = args.into_iter();
        let prog = args.next().unwrap().as_ref().to_string();
        let mut command = Command::new(&prog);
        let mut repr = prog;
        if let Some(dir) = dir {
            command.current_dir(dir);
            repr = format!("cd {} && {}", dir, repr);
        }
        for arg in args {
            let arg_str: &str = arg.as_ref();
            command.arg(arg_str);
            repr.push_str(" ");
            repr.push_str(arg_str);
        }
        Self { command, repr }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        println!("====================================================");
        println!("Command: {}", &self.repr);
        println!("====================================================");
        let status = self.command.status()?;
        if !status.success() {
            bail!("command failed: {}", &self.repr);
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let show = matches!(std::env::args().nth(1), Some(s) if s == "--show");

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mark = |b| if b { "YES ✔" } else { "no" };
        println!("x86 CPU features detected:");
        println!("  SSE4.1: {}", mark(is_x86_feature_detected!("sse4.1")));
        println!("    AVX2: {}", mark(is_x86_feature_detected!("avx2")));
        println!(" AVX512F: {}", mark(is_x86_feature_detected!("avx512f")));
        println!("AVX512VL: {}", mark(is_x86_feature_detected!("avx512vl")));
        println!("");
    }

    let project_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::env::set_current_dir(project_root_dir)?;

    let mut commands = Vec::new();
    commands.push(TestCommand::new(&["cargo", "test"]));
    commands.push(TestCommand::new(&[
        "cargo",
        "test",
        "--no-default-features",
    ]));

    for purity in &[&[][..], &["prefer_intrinsics"][..], &["pure"][..]] {
        for support in &[
            &[][..],
            &["no_avx512"][..],
            &["no_avx512", "no_avx2"][..],
            &["no_avx512", "no_avx2", "no_sse41"][..],
        ] {
            for &release in &[false, true] {
                let mut command = vec!["cargo", "test"];
                let mut features_vec: Vec<&str> = purity.to_vec();
                features_vec.extend_from_slice(support);
                let features = format!("--features={}", features_vec.join(","));
                command.push(&features);
                if release {
                    command.push("--release");
                }
                commands.push(TestCommand::new(command));
            }
        }
    }

    commands.push(TestCommand::new_with_dir(&["cargo", "test"], "b3sum"));
    commands.push(TestCommand::new_with_dir(
        &["cargo", "test", "--no-default-features"],
        "b3sum",
    ));

    commands.push(TestCommand::new_with_dir(
        &["cargo", "test"],
        "test_vectors",
    ));
    commands.push(TestCommand::new_with_dir(
        &["cargo", "test", "--features=prefer_intrinsics"],
        "test_vectors",
    ));
    commands.push(TestCommand::new_with_dir(
        &["cargo", "test", "--features=pure"],
        "test_vectors",
    ));

    commands.push(TestCommand::new_with_dir(
        &["cargo", "test"],
        "c/blake3_c_rust_bindings",
    ));
    commands.push(TestCommand::new_with_dir(
        &["cargo", "test", "--features=prefer_intrinsics"],
        "c/blake3_c_rust_bindings",
    ));

    commands.push(TestCommand::new_with_dir(
        &["cargo", "test"],
        "reference_impl",
    ));

    for command in &mut commands {
        if show {
            println!("{}", &command.repr);
        } else {
            command.run()?;
        }
    }

    Ok(())
}
