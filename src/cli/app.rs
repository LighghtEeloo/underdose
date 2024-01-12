use super::interface::{Cli, Commands};
use crate::{
    utils::{
        conf::{Conf, UnderdoseConf},
        global::UNDERDOSE_PATH,
    },
    Dreamer, Drugstore, Executor, Machine,
};
use clap::Parser;

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Init { name } => {
                // setup underdose configuration
                let repo = std::env::current_dir()?;
                let underdose_conf = UnderdoseConf::new(name, repo);
                let conf = underdose_conf.conf(UNDERDOSE_PATH.conf.clone());
                log::info!("writing underdose configuration to {}", conf.path.display());
                if conf.path.exists() {
                    print!("underdose configuration already exists; overwrite? [y/N] ");
                    std::io::Write::flush(&mut std::io::stdout())?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() == "y" {
                        conf.ensure_forced()?;
                    } else {
                        anyhow::bail!("not overwriting underdose configuration, aborting...")
                    }
                } else {
                    conf.ensure_exist()?;
                }
                conf.edit()?;
            }
            Commands::Config => {
                let conf = Conf {
                    buffer: String::new(),
                    path: UNDERDOSE_PATH.conf.clone(),
                };
                conf.edit()?;
            }
            Commands::Where => unimplemented!(),
            Commands::Sync { names } => {
                let content = Conf {
                    buffer: String::new(),
                    path: UNDERDOSE_PATH.conf.clone(),
                }
                .read()?;
                let machine = Machine::try_from(&content[..])?;
                let content = Conf {
                    buffer: String::new(),
                    path: machine.local.join("Drugstore.toml"),
                }
                .read()?;
                let store = Drugstore::try_from((&content[..], &machine))?;

                log::trace!("{:#?}", machine);
                log::trace!("{:#?}", store);

                for name in names.iter() {
                    if !store.pills.contains_key(name) {
                        anyhow::bail!("no such pill: {}", name);
                    }
                }

                let mut dreamer = Dreamer::new();
                for (name, drip) in store.pills.iter() {
                    if names.len() > 0 && !names.contains(name) {
                        continue;
                    }
                    // dump current site to dreamer
                    dreamer.dump(name.clone(), drip)?;
                    // execute drip
                    Executor {
                        repo: &machine.local,
                        drip,
                    }
                    .run()?;
                }
            }
        };

        Ok(())
    }
}
