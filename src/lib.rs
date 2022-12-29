#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use toml_edit::Document;

#[derive(Serialize, Deserialize)]
pub struct DrugStore {
    /// a map of name -> upward dependencies, up to the root
    pub env: HashMap<String, HashSet<String>>,
    pub pill: Vec<Pill>,
    pub tutorial: Option<()>,
}

#[derive(Serialize, Deserialize)]
pub struct Pill {
    pub name: String,
    pub drip: Vec<Drip>,
}

#[derive(Serialize, Deserialize)]
pub struct Drip {
    /// env is resolved to trivial form during parsing
    pub env: HashSet<String>,
    /// site root in the machine
    pub root: PathBuf,
    /// drip root in the drugstore
    pub pill: PathBuf,
    /// Atoms are incremented from drips but dirs aren't expanded
    pub stem: Vec<Atom>,
}

#[derive(Serialize, Deserialize)]
pub struct Atom {
    pub site: PathBuf,
    pub repo: PathBuf,
    pub mode: AtomTaskMode,
}

pub struct AtomTask {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub mode: AtomTaskMode,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum AtomTaskMode {
    #[serde(rename = "copy")]
    FileCopy,
    #[serde(rename = "link")]
    Link,
}

impl AtomTaskMode {
    pub fn display_arrow(&self) -> &'static str {
        match self {
            Self::FileCopy => "==>",
            Self::Link => "~~>",
        }
    }
}

impl Display for AtomTaskMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomTaskMode::FileCopy => write!(f, "copy"),
            AtomTaskMode::Link => write!(f, "link"),
        }
    }
}

// impl DrugStore {
//     pub fn of_file(mut file: File) -> Self {
//         let mut content = String::new();
//         file.read_to_string(&mut content);
//         let doc = content.parse::<Document>().expect("invalid doc");
//         let item = doc["env"].clone();
//         todo!()
//     }
// }

impl Drip {
    pub fn check_env(&self, machine_env: HashSet<String>) -> bool {
        true
    }
    /// Resolve stem atoms to absolute file paths; requires a direction
    pub fn resolve_atoms<VisSrc, VisDst>(&self, src: &VisSrc, dst: &VisDst) -> Vec<AtomTask>
    where
        VisSrc: Fn(&Atom) -> &Path + Clone,
        VisDst: Fn(&Atom) -> &Path + Clone,
    {
        fn extend_atom(tasks: &mut Vec<AtomTask>, src: &Path, dst: &Path, mode: AtomTaskMode) {
            if src.is_file() || matches!(mode, AtomTaskMode::Link) {
                tasks.push(AtomTask {
                    src: src.to_owned(),
                    dst: dst.to_owned(),
                    mode,
                })
            } else if src.is_dir() {
                for entry in src.read_dir().expect("read_dir failed") {
                    let entry = entry.expect("entry failed");
                    let path = entry.path();
                    let file_name = path.file_name().expect("file_name failed");
                    let dst_path = dst.join(file_name);
                    extend_atom(tasks, &path, &dst_path, mode)
                }
            }
        }
        let mut tasks = Vec::new();
        for atom in &self.stem {
            let src_path = self.root.join(src(atom));
            let dst_path = self.pill.join(dst(atom));
            extend_atom(&mut tasks, &src_path, &dst_path, atom.mode)
        }
        tasks
    }
    pub fn resolve_atoms_to_repo(&self) -> Vec<AtomTask> {
        self.resolve_atoms(&|atom| atom.site.as_path(), &|atom| atom.repo.as_path())
    }
    pub fn resolve_atoms_to_site(&self) -> Vec<AtomTask> {
        self.resolve_atoms(&|atom| atom.repo.as_path(), &|atom| atom.site.as_path())
    }
}
