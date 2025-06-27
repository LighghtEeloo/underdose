use super::interface::{Cli, Commands};
use crate::{
    Dreamer, Drugstore, Executor, Machine,
    utils::{
        conf::{Conf, TomlStr, UnderdoseConf},
        global::UNDERDOSE_PATH,
    },
};
use clap::Parser;

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn main(self) -> anyhow::Result<()> {
        match self.command {
            | Commands::Init { name } => {
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
            | Commands::Conf => {
                let conf = Conf {
                    buffer: String::new(),
                    path: UNDERDOSE_PATH.conf.clone(),
                };
                conf.edit()?;
            }
            | Commands::Where => {
                let conf_path = UNDERDOSE_PATH.conf.display();
                let conf = Conf {
                    buffer: String::new(),
                    path: UNDERDOSE_PATH.conf.clone(),
                };
                let content = conf.read()?;
                let machine = Machine::try_from(&content[..])?;
                let drugstore_path = machine.local.display();
                let dreams_path = UNDERDOSE_PATH.dreams.display();
                print!("[configurations] ");
                println!("{}", conf_path);
                print!("[drugstore] ");
                println!("{}", drugstore_path);
                print!("[dreams] ");
                println!("{}", dreams_path);
            }
            | Commands::Sync { names } => {
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
                let toml = TomlStr::new(&content[..]);
                let store = Drugstore::try_from((toml, &machine))?;

                log::trace!("{:#?}", machine);
                log::trace!("{:#?}", store);

                for name in names.iter() {
                    if !store.pills.contains_key(name) && !store.cmds.contains_key(name) {
                        anyhow::bail!("no such pill or command: {}", name);
                    }
                }

                for (name, cmd) in store.cmds.iter() {
                    if names.len() > 0 && !names.contains(&name) {
                        continue;
                    }
                    log::info!(
                        "running command <{}> :: {} {}",
                        name,
                        cmd.prog,
                        cmd.args.join(" ")
                    );
                    let status = std::process::Command::new(&cmd.prog)
                        .args(&cmd.args)
                        .status()?;
                    if !status.success() {
                        anyhow::bail!("command failed: {}", name);
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
            | Commands::Clean { name, version } => {
                let mut dreamer = Dreamer::new();
                let drip = dreamer
                    .map
                    .get_mut(&name)
                    .ok_or_else(|| anyhow::anyhow!("the name doesn't exist"))?;
                let removing = drip.matches_uuid(version);
                drip.remove_uuids(removing)?;
            }
        };

        Ok(())
    }
}
