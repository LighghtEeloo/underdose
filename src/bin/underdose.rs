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
    drugstore::{
        Atom,
        AtomMode::{self, *},
        Drip, DripVariant, Pill,
    },
    repo::Dirt,
    task::{AtomTask, DripTask, Exec, Synthesis, TaskArrow},
    utils::{self, Conf},
    Drugstore, Machine,
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
    log::info!(
        "\nreading underdose_conf: \n\tname: {}\n\tpath: {}",
        underdose_conf.name,
        underdose_conf.path.display()
    );
    let machine_buf = underdose_conf.ensure()?.read()?;
    let machine: Machine = machine_buf.as_str().try_into()?;
    log::debug!("\n{:#?}", machine);

    let drugstore_conf_name = "Drugstore.toml";
    let drugstore_conf = Conf {
        name: drugstore_conf_name.to_string(),
        template: include_str!("../../templates/Drugstore.toml"),
        path: machine.repo.join(drugstore_conf_name),
    };
    log::info!(
        "\nreading drugstore_conf: \n\tname: {}\n\tpath: {}",
        drugstore_conf.name,
        drugstore_conf.path.display()
    );
    let store_buf = drugstore_conf.ensure()?.read()?;
    let store: Drugstore = (&toml::from_str(&store_buf)?, &machine).try_into()?;
    log::debug!("\n{:#?}", store);

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

    for (name, pill) in &store.pills {
        let drip_task = pill.drip.synthesis(&machine, TaskArrow::SiteToRepo)?;
        match &drip_task {
            DripTask::GitModule { remote, .. } => {}
            DripTask::Addicted {
                ref root,
                ref atoms,
            } => {
                log::info!(
                    "\n[[{}]] {} {:#?}",
                    name,
                    root,
                    atoms
                        .iter()
                        .map(|task| { format!("{}", task) })
                        .collect::<Vec<String>>()
                );
            }
        }

        let mut response = String::new();
        print!("proceed? [N/y/s] ");
        io::stdout().flush();
        {
            let stdin = io::stdin();
            stdin.read_line(&mut response)?;
        }

        match response.to_lowercase().trim() {
            "y" => {
                println!("executing...");
                drip_task.exec()?;
            }
            "n" => {
                println!("abort!");
            }
            _ => {
                println!("skipping...");
            }
        }
    }
    Ok(())
}
