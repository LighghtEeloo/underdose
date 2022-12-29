#![allow(unused)]

use git2::Repository;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use underdose::{Atom, AtomTaskMode::*, Drip, DrugStore, Pill};

fn main() {
    env_logger::init();

    let home_path = std::env::var("HOME").unwrap();
    let drugstore_path = PathBuf::from(&home_path).join(format!(".IronCloak/"));
    let store = DrugStore {
        env: HashMap::new(),
        pill: vec![Pill {
            name: format!("nvim"),
            drip: vec![Drip {
                env: HashSet::new(),
                root: (&home_path).into(),
                pill: PathBuf::from(&home_path).join(format!(".IronCloak/Zshrc/")),
                stem: vec![
                    Atom {
                        site: format!(".zshrcx/").into(),
                        repo: format!(".zshrcx/").into(),
                        mode: FileCopy,
                    },
                    Atom {
                        site: format!(".zshrc").into(),
                        repo: format!(".zshrc").into(),
                        mode: FileCopy,
                    },
                ],
            }],
        }],
        tutorial: None,
    };

    for pill in store.pill {
        for drip in &pill.drip {
            log::info!(
                "\n[[{}]]::resolve_atoms_to_repo: {:#?}",
                pill.name,
                drip.resolve_atoms_to_repo()
                    .into_iter()
                    .map(|task| {
                        format!(
                            "<{}> {} {} {}",
                            task.mode,
                            task.src.display(),
                            task.mode.display_arrow(),
                            task.dst.display()
                        )
                    })
                    .collect::<Vec<String>>()
            )
        }
        // for drip in &pill.drip {
        //     log::info!(
        //         "\n[[{}]]::resolve_atoms_to_site: {:#?}",
        //         pill.name,
        //         drip.resolve_atoms_to_site()
        //             .into_iter()
        //             .map(|task| {
        //                 format!(
        //                     "<{}> {} {} {}",
        //                     task.mode,
        //                     task.src.display(),
        //                     task.mode.display_arrow(),
        //                     task.dst.display()
        //                 )
        //             })
        //             .collect::<Vec<String>>()
        //     )
        // }
    }
}
