fn main() {
    // Set in build.rs.
    let compiler_path = env!("COMPILER_PATH");

    let mut compiler_command = std::process::Command::new(compiler_path);
    // Use the --version flag on everything other than MSVC.
    if !cfg!(target_env = "msvc") {
        compiler_command.arg("--version");
    }
    // Run the compiler to print its version. Ignore the exit status.
    let _ = compiler_command.status().unwrap();
}
