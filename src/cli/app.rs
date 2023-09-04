use super::interface::{Cli, Commands};
use crate::{
    dynamics::{AtomArrow, Execution, PillTask, Synthesis},
    utils::{
        conf::{Conf, Prompt, UnderdoseConf},
        repo::Dirt,
    },
    Drugstore, Machine,
};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::path::PathBuf;

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
                        .local
                        .join(".underdose")
                        .join(&format!("{}.toml", machine.name)),
                }
                .ensure_template_forced()?;
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
