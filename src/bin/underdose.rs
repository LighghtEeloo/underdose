#![allow(unused)]
#![allow(clippy::useless_format)]

use directories_next::ProjectDirs;
use git2::{Delta, DiffFile, Repository, Statuses};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io::{self, Write},
    path::{Path, PathBuf},
};
use underdose::{
    task::{AtomTask, DripTask, Synthesis, Task, TaskArrow},
    utils::{self, Conf},
    Atom,
    AtomMode::{self, *},
    Drip, DripVariant, Drugstore, Machine, Pill,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let home_path = utils::expand_path(std::env::var("HOME").unwrap());

    let underdose_conf_name = "Underdose.toml";
    let underdose_dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
        .expect("No valid config directory fomulated");

    let underdose_conf = Conf {
        name: underdose_conf_name.to_string(),
        template: include_str!("../../templates/Underdose.toml"),
        path: underdose_dirs.config_dir().join(underdose_conf_name),
    };
    let machine_buf = underdose_conf.ensure()?.read()?;
    let machine: Machine = machine_buf.as_str().try_into()?;
    log::info!("{:#?}", machine);

    let drugstore_conf_name = "Drugstore.toml";
    let drugstore_conf = Conf {
        name: drugstore_conf_name.to_string(),
        template: include_str!("../../templates/Drugstore.toml"),
        path: machine.repo.join(drugstore_conf_name),
    };
    let store_buf = drugstore_conf.ensure()?.read()?;
    let store: Drugstore = (&toml::from_str(&store_buf)?, &machine).try_into()?;
    log::info!("{:#?}", store);

    let repo = Repository::open(&machine.repo).expect("failed to open repo");

    let statuses = repo.statuses(None)?;
    let dirts = Dirt::of_repo_status(&statuses)?;
    if !dirts.is_empty() {
        println!("Worktree not clean!");
        println!(
            "{:#?}",
            dirts
                .into_iter()
                .map(|dirt| { format!("{}", dirt) })
                .collect::<Vec<String>>()
        );
        return Ok(());
    }

    return Ok(());

    for (name, pill) in &store.pills {
        for drip in &pill.drips {
            if !drip.check_env(&machine.env) {
                continue;
            }
            let drip_task = drip.syn(TaskArrow::SiteToRepo)?;
            match drip_task {
                DripTask::GitModule { remote, .. } => {}
                DripTask::Addicted { ref atoms } => {
                    log::info!(
                        "\n[[{}]]::site_to_repo: {:#?}",
                        name,
                        atoms
                            .iter()
                            .map(|task| { format!("{}", task) })
                            .collect::<Vec<String>>()
                    );
                }
            }

            // let mut response = String::new();
            // print!("proceed?");
            // io::stdout().flush();
            // {
            //     let stdin = io::stdin();
            //     stdin.read_line(&mut response)?;
            // }

            // if (response.to_lowercase().starts_with('y')) {
            //     println!("executing...");
            //     drip_task.exec()?;
            // }
        }
    }
    Ok(())
}

pub struct Dirt<'a> {
    old: &'a Path,
    new: &'a Path,
    delta: Delta,
}

impl<'a> Display for Dirt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ===[{:?}]==> {}",
            self.old.display(),
            self.delta,
            self.new.display(),
        )
    }
}

impl<'a> Dirt<'a> {
    fn of_repo_status(statuses: &'a Statuses) -> anyhow::Result<Vec<Dirt<'a>>> {
        let mut dirts = Vec::new();
        let file_to_path =
            |file: DiffFile<'a>| file.path().ok_or_else(|| anyhow::anyhow!("file path err"));
        for status in statuses.iter() {
            match status.index_to_workdir() {
                None => (),
                Some(status) => {
                    let delta = status.status();
                    match delta {
                        Delta::Unmodified | Delta::Ignored => (),
                        Delta::Added
                        | Delta::Deleted
                        | Delta::Modified
                        | Delta::Renamed
                        | Delta::Copied
                        | Delta::Untracked
                        | Delta::Typechange
                        | Delta::Unreadable
                        | Delta::Conflicted => dirts.push(Dirt {
                            old: file_to_path(status.old_file())?,
                            new: file_to_path(status.new_file())?,
                            delta,
                        }),
                    }
                }
            }
        }
        Ok(dirts)
    }
}
