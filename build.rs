use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate_to};
use std::{io, path::PathBuf};

include!("src/cli/interface.rs");

fn main() -> Result<(), io::Error> {
    let mut app = Cli::command();

    let outdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("completions");
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut app, "underdose", &outdir)?;
    }

    Ok(())
}
