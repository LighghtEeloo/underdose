use crate::{
    dynamics::{AtomArrow, Execution, PillTask, Synthesis},
    utils::{repo::Dirt, Conf, Prompt, UnderdoseConf},
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

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        let default_dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
            .expect("No valid config directory fomulated");
        let current_dir = std::env::current_dir()?;

        match self.command {
            Commands::Init { name } => {
                // read underdose_conf into machine
                // the priority of config file is:
                // 1. --config
                // 2. current_dir.join(".underdose/").join("{--name}.toml")
                // 3. underdose_dirs.config_dir().join(underdose_conf_name)
                let underdose_conf_name = "Underdose.toml";
                let underdose_conf_path = {
                    self.config.unwrap_or_else(|| {
                        let underdose_repo_conf_path = current_dir
                            .join(".underdose")
                            .join(format!("{}.toml", name));
                        if underdose_repo_conf_path.exists() {
                            underdose_repo_conf_path
                        } else {
                            default_dirs.config_dir().join(underdose_conf_name)
                        }
                    })
                };
                let underdose_conf =
                    UnderdoseConf::new(name).conf(underdose_conf_path);
                log::info!(
                    "\nediting underdose_conf: {}",
                    underdose_conf.path.display()
                );
                underdose_conf.ensure_exist()?;

                // edit underdose_conf
                underdose_conf.edit()?;

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
            Commands::Drain { store } => todo!(),
            Commands::Sync { store } => todo!(),
            Commands::Pour { force, store } => todo!(),
        }

        Ok(())
    }
}
