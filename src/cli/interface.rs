mod uses {
    pub use clap::{Parser, Subcommand};
    pub use std::path::PathBuf;
}
use uses::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Use a custom config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize on a new machine, working from drugstore repo
    Init {
        /// Set the name of the machine
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Configure the machine
    Config,
    /// Shows all path information available
    Where,
    /// Drain the machine to the drugstore
    Drain {
        /// The name of the drugstore repo
        #[arg(short, long, value_name = "NAME")]
        store: Option<String>,
    },
    /// Drain the machine to the drugstore, and pour on condition
    Sync {
        /// The name of the drugstore repo
        #[arg(short, long, value_name = "NAME")]
        store: Option<String>,
    },
    /// Pour the drugstore to the machine
    Pour {
        #[arg(short, long)]
        force: bool,
        /// The name of the drugstore repo
        #[arg(short, long, value_name = "NAME")]
        store: Option<String>,
    },
}
