use super::interface::{Cli, Commands};
use clap::Parser;

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        // step 1: read underdose_conf into machine
        match self.command {
            Commands::Init => {}
            Commands::Config => todo!(),
            Commands::Where => todo!(),
            Commands::Sync => todo!(),
        };

        Ok(())
    }
}
