#![allow(unused)]
#![allow(clippy::useless_format)]

use directories_next::ProjectDirs;
use git2::Repository;
use std::io;
use underdose::{
    repo::Dirt,
    task::{DripTask, Exec, Synthesis, TaskArrow},
    utils::{Conf, Prompt},
    Drugstore, Machine,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let underdose_dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
        .expect("No valid config directory fomulated");

    // read underdose_conf into machine
    let underdose_conf_name = "Underdose.toml";
    let underdose_conf = Conf {
        template: include_str!("../../templates/Underdose.toml"),
        path: underdose_dirs.config_dir().join(underdose_conf_name),
    };
    log::info!(
        "\nreading underdose_conf: {}",
        underdose_conf.path.display()
    );
    let machine_buf = underdose_conf.ensure()?.read()?;
    let machine: Machine = machine_buf.as_str().try_into()?;
    log::debug!("\n{:#?}", machine);

    // write local conf to drugstore/.underdose/<name>.toml
    Conf {
        template: &machine_buf,
        path: machine.repo.join(".underdose").join(&machine.name),
    };

    let drugstore_conf_name = "Drugstore.toml";
    let drugstore_conf = Conf {
        template: include_str!("../../templates/Drugstore.toml"),
        path: machine.repo.join(drugstore_conf_name),
    };
    log::info!(
        "\nreading drugstore_conf: {}",
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
        let drip_task = pill.drip.synthesis(name, &machine, TaskArrow::RepoToSite)?;
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

        Prompt::new("proceed? [N/y/!] ").process(|s| {
            match s {
                "y" => {
                    println!("executing...");
                    drip_task.exec()?;
                }
                "!" => {
                    println!("abort!");
                    Err(io::Error::new(io::ErrorKind::Other, "abort!"))?;
                }
                _ => {
                    println!("skipping...");
                }
            };
            Ok(())
        })?;
    }
    Ok(())
}
