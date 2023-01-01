use crate::{
    dynamics::{Execution, PillTask, Synthesis, TaskArrow},
    repo::Dirt,
    utils::{Conf, Prompt, UnderdoseConf},
    Drugstore, Machine,
};
use clap::{Parser, Subcommand};
use directories_next::ProjectDirs;
use std::path::PathBuf;

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
    Drain,
    /// Drain the machine to the drugstore, and pour on condition
    Sync,
    /// Pour the drugstore to the machine
    Pour {
        #[arg(short, long)]
        force: bool,
    },
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        let underdose_dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
            .expect("No valid config directory fomulated");

        match self.command {
            Commands::Init { name } => {
                // read underdose_conf into machine
                let underdose_conf_name = "Underdose.toml";
                let underdose_conf = UnderdoseConf::new(name).conf(
                    self.config.unwrap_or_else(|| {
                        underdose_dirs.config_dir().join(underdose_conf_name)
                    }),
                );
                log::info!(
                    "\nreading underdose_conf: {}",
                    underdose_conf.path.display()
                );
                underdose_conf.ensure_force()?;
                let machine_buf = underdose_conf.read()?;
                let machine: Machine = machine_buf.as_str().try_into()?;
                log::debug!("\n{:#?}", machine);

                // write local conf to drugstore/.underdose/<name>.toml
                Conf {
                    template: machine_buf,
                    path: machine
                        .repo
                        .join(".underdose")
                        .join(&format!("{}.toml", machine.name)),
                }
                .ensure_force()?;
            }
            Commands::Config => todo!(),
            Commands::Where => todo!(),
            Commands::Drain => todo!(),
            Commands::Sync => todo!(),
            Commands::Pour { force } => todo!(),
        }

        Ok(())
    }
}
