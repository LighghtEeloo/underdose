pub use clap::{Parser, Subcommand};

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
    Conf,
    /// Shows all path information available
    Where,
    /// Make a dream on the machine, and pour if possible
    Sync {
        #[arg()]
        names: Vec<String>,
    },
    /// Clean up backups
    Clean {
        /// name of the backup
        #[arg(short, long)]
        name: String,
        /// version of the backup, can be a uuid or "all"
        #[arg(short, long)]
        version: String,
    },
}
