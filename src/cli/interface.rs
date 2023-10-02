mod uses {
    pub use clap::{Parser, Subcommand};
    pub use std::path::PathBuf;
}
use uses::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize on a new machine, working from drugstore repo
    Init {
        /// name of the machine
        #[arg(required = true, index = 1)]
        name: String,
    },
    /// Configure the machine
    #[command(alias = "edit")]
    Config,
    /// Shows all path information available
    Where,
    /// Make a dream on the machine, and pour if possible
    Sync,
}
