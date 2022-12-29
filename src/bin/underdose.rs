#![allow(unused)]
#![allow(clippy::useless_format)]

use git2::{Delta, DiffFile, Repository, Statuses};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io::{self, Write},
    path::{Path, PathBuf},
};
use underdose::{Atom, AtomTaskMode::*, Drip, DrugStore, Machine, Pill};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let home_path = PathBuf::from(std::env::var("HOME").unwrap());
    let machine = Machine {
        env: HashSet::from([format!("arch")]),
        repo: PathBuf::from(&home_path).join(format!(".IronCloak/")),
        tutorial: None,
    };

    let repo = Repository::open(&machine.repo).expect("failed to open repo");

    let statuses = repo.statuses(None)?;
    let dirts = repo_status_dirts(&statuses)?;
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

    let store = DrugStore {
        env: HashMap::new(),
        pills: HashMap::from([(
            format!("nvim"),
            Pill {
                drip: Vec::from([Drip {
                    env: HashSet::new(),
                    root: home_path.join(".config/").into(),
                    pill: machine.repo.join("Neovim/"),
                    stem: Vec::from([Atom {
                        site: format!("nvim/").into(),
                        repo: format!("./").into(),
                        mode: FileCopy,
                    }]),
                }]),
            },
        )]),
        tutorial: None,
    };

    for (name, pill) in &store.pills {
        for drip in &pill.drip {
            if !drip.check_env(&machine.env) {
                continue;
            }
            let atom_tasks = drip.resolve_atoms_to_repo();
            log::info!(
                "\n[[{}]]::resolve_atoms_to_repo: {:#?}",
                name,
                atom_tasks
                    .iter()
                    .map(|task| { format!("{}", task) })
                    .collect::<Vec<String>>()
            );
            let mut response = String::new();
            print!("proceed?");
            io::stdout().flush();
            {
                let stdin = io::stdin();
                stdin.read_line(&mut response)?;
            }
            if (response.to_lowercase().starts_with("y")) {
                println!("executing...");
                for task in atom_tasks {
                    task.exec()?;
                }
            }
        }
    }

    // for (name, pill) in &store.pills {
    //     for drip in &pill.drip {
    //         let atom_tasks = drip.resolve_atoms_to_site();
    //         log::info!(
    //             "\n[[{}]]::resolve_atoms_to_site: {:#?}",
    //             name,
    //             atom_tasks
    //                 .iter()
    //                 .map(|task| { format!("{}", task) })
    //                 .collect::<Vec<String>>()
    //         );
    //         for task in atom_tasks {
    //             task.exec()?;
    //         }
    //     }
    // }

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
            "{} ==[{:?}]==> {}",
            self.old.display(),
            self.delta,
            self.new.display(),
        )
    }
}

fn repo_status_dirts<'a>(statuses: &'a Statuses) -> anyhow::Result<Vec<Dirt<'a>>> {
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
