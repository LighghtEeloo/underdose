use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Use a custom config file
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize on a new machine, working from drugstore repo
    Init {
        /// Set the name of the machine
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Configure the machine
    Config,
    /// Shows all path information available
    Where,
    /// Drain the machine to the drugstore
    Drain,
    /// Drain the machine to the drugstore, and pour on condition
    Sync,
    /// Pour the drugstore to the machine
    Pour {
        #[arg(short, long)]
        force: bool,
    },
}
