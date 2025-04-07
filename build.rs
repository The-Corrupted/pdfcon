use clap_complete::{generate_to, shells::Zsh};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = build_command();

    let path = generate_to(Zsh, &mut cmd, "pdfcon", outdir)?;

    println!("cargo:warning=completion file is generated: {path:?}");

    Ok(())
}
