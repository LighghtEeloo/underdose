use super::interface::{Cli, Commands};
use crate::{
    utils::{
        conf::{Conf, UnderdoseConf},
        global::UNDERDOSE_PATH,
    },
    Machine,
};
use clap::Parser;

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;

        // step 1: read underdose_conf into machine
        let machine_buf = match self.command {
            Commands::Init { name } => {
                // read underdose_conf into machine
                // the priority of config file is:
                // 1. --config
                // 2. current_dir.join(".underdose/").join("{--name}.toml")
                // 3. underdose_dirs.config_dir().join(underdose_conf_name)
                let underdose_conf_path = {
                    self.config.unwrap_or_else(|| {
                        let underdose_repo_conf_path = current_dir
                            .join(".underdose")
                            .join(format!("{}.toml", name));
                        if underdose_repo_conf_path.exists() {
                            underdose_repo_conf_path
                        } else {
                            UNDERDOSE_PATH.conf.clone()
                        }
                    })
                };
                let underdose_conf = UnderdoseConf::new(name).conf(underdose_conf_path);
                log::info!(
                    "\nediting underdose_conf: {}",
                    underdose_conf.path.display()
                );
                underdose_conf.ensure_exist()?;

                // edit underdose_conf
                underdose_conf.edit()?;

                underdose_conf.read()?
            }
            Commands::Config => todo!(),
            Commands::Where => todo!(),
            Commands::Drain { store } => todo!(),
            Commands::Sync { store } => todo!(),
            Commands::Pour { force, store } => todo!(),
        };

        let machine: Machine = machine_buf.as_str().try_into()?;
        log::debug!("\n{:#?}", machine);

        // if overdosed, write local conf to drugstore/.underdose/<name>.toml
        // and create a soft symlink
        if machine.overdose {
            Conf {
                buffer: machine_buf,
                path: machine
                    .local
                    .join(".underdose")
                    .join(&format!("{}.toml", machine.name)),
            }
            .ensure_forced()?;
        }

        Ok(())
    }
}
