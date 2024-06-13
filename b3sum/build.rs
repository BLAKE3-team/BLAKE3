use clap::CommandFactory;

include!("src/cli.rs");

fn generate_completions(out_dir: &std::path::Path) -> std::io::Result<()> {
    fn generate_to(
        gen: impl clap_complete::Generator,
        out_dir: &std::path::Path,
    ) -> std::io::Result<std::path::PathBuf> {
        let mut command = Inner::command();
        clap_complete::generate_to(gen, &mut command, "b3sum", out_dir)
    }

    generate_to(clap_complete::Shell::Bash, out_dir)?;
    generate_to(clap_complete::Shell::Elvish, out_dir)?;
    generate_to(clap_complete::Shell::Fish, out_dir)?;
    generate_to(clap_complete::Shell::PowerShell, out_dir)?;
    generate_to(clap_complete::Shell::Zsh, out_dir)?;
    Ok(())
}

fn generate_man_page(out_dir: &std::path::Path) -> std::io::Result<()> {
    let command = Inner::command();

    let man = clap_mangen::Man::new(command).date("2024-04-24");
    let mut buf = Vec::new();
    man.render_title(&mut buf)?;

    // The NAME section.
    let mut roff = clap_mangen::roff::Roff::new();
    roff.control("SH", ["NAME"]);
    roff.text([clap_mangen::roff::roman(
        "b3sum - compute and check BLAKE3 message digest",
    )]);
    roff.to_writer(&mut buf)?;

    // The SYNOPSIS section.
    let mut roff = clap_mangen::roff::Roff::new();
    roff.control("SH", ["SYNOPSIS"]);
    roff.text([
        clap_mangen::roff::bold("b3sum"),
        clap_mangen::roff::roman(" ["),
        clap_mangen::roff::italic("OPTIONS"),
        clap_mangen::roff::roman("] ["),
        clap_mangen::roff::italic("FILE"),
        clap_mangen::roff::roman("]..."),
    ]);
    roff.to_writer(&mut buf)?;

    man.render_description_section(&mut buf)?;
    man.render_options_section(&mut buf)?;

    // The SEE ALSO section.
    let mut roff = clap_mangen::roff::Roff::new();
    roff.control("SH", ["SEE ALSO"]);
    roff.text([
        clap_mangen::roff::bold("b2sum"),
        clap_mangen::roff::roman("(1), "),
        clap_mangen::roff::bold("md5sum"),
        clap_mangen::roff::roman("(1)"),
    ]);
    roff.to_writer(&mut buf)?;

    std::fs::write(out_dir.join("b3sum.1"), buf)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/cli.rs");

    let out_dir = std::env::var("OUT_DIR").expect("environment variable `OUT_DIR` not defined");
    let out_dir = std::path::PathBuf::from(out_dir);

    generate_completions(&out_dir)?;
    generate_man_page(&out_dir)?;
    Ok(())
}
