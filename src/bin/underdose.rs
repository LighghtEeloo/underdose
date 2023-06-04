#![allow(unused)]
#![allow(clippy::useless_format)]

use directories_next::ProjectDirs;
use git2::Repository;
use std::io;
use underdose::{
    cli::Cli,
    dynamics::{AtomArrow, Execution, PillTask, Probing, Synthesis},
    utils::{
        conf::{Conf, Prompt, DRUGSTORE_TOML, UNDERDOSE_TOML},
        repo::Dirt,
    },
    Drugstore, Machine,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let underdose_dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
        .expect("No valid config directory fomulated");

    let cli = Cli::new();
    // let cli = Cli::new().main()?;

    // read underdose_conf into machine
    let underdose_conf_name = "Underdose.toml";
    let underdose_conf = Conf {
        template: UNDERDOSE_TOML.to_owned(),
        path: cli.config.unwrap_or_else(|| {
            underdose_dirs.config_dir().join(underdose_conf_name)
        }),
    };
    log::info!(
        "\nreading underdose_conf: {}",
        underdose_conf.path.display()
    );
    let machine_buf = underdose_conf.ensure_exist()?.read()?;
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

    // read drugstore_conf into store
    let drugstore_conf_name = "Drugstore.toml";
    let drugstore_conf = Conf {
        template: DRUGSTORE_TOML.to_owned(),
        path: machine.repo.join(drugstore_conf_name),
    };
    log::info!(
        "\nreading drugstore_conf: {}",
        drugstore_conf.path.display()
    );
    let store_buf = drugstore_conf.ensure_exist()?.read()?;
    let store: Drugstore = (store_buf.as_ref(), &machine).try_into()?;
    log::debug!("\n{:#?}", store);

    // open drugstore repo
    let repo = Repository::open(&machine.repo).expect("failed to open repo");

    // check if worktree is clean
    let statuses = repo.statuses(None)?;
    let dirts = Dirt::of_repo_status(&statuses)?;
    if !dirts.is_empty() {
        println!("Worktree not clean!");
        for dirt in dirts {
            println!("{}", dirt);
        }
        return Ok(());
    }

    // synthesize and execute tasks
    for (_, pill) in &store.pills {
        let pill_ob = pill.probing(&machine, AtomArrow::RepoToSite)?;
        println!("{}", pill_ob);
        let pill_task = pill_ob.synthesis(&machine)?;
        println!("{}", pill_task);

        Prompt::new("proceed? [N/y/!] ").process(|s| {
            match s {
                "y" => {
                    println!("executing...");
                    pill_task.execution()?;
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

    println!();
    Ok(())
}
