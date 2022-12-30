use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub struct DrugStore {
    /// a map of name -> upward dependencies, up to the root
    pub env: HashMap<String, HashSet<String>>,
    pub pills: HashMap<String, Pill>,
    pub tutorial: Option<()>,
}

#[derive(Serialize, Deserialize)]
pub struct Pill {
    pub drips: Vec<Drip>,
}

#[derive(Serialize, Deserialize)]
pub struct Drip {
    /// env is resolved to trivial form during parsing
    pub env: HashSet<String>,
    pub root: Atom,
    /// variants
    #[serde(flatten)]
    pub var: DripVariant,
}

#[derive(Serialize, Deserialize)]
pub enum DripVariant {
    GitModule {
        remote: String,
    },
    UnderManage {
        /// Atoms are incremented from drips but dirs aren't expanded
        stem: Vec<Atom>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Atom {
    pub site: PathBuf,
    pub repo: PathBuf,
    pub mode: AtomMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AtomMode {
    #[serde(rename = "copy")]
    FileCopy,
    #[serde(rename = "link")]
    Link,
}

impl AtomMode {
    pub fn display_arrow(&self) -> &'static str {
        match self {
            Self::FileCopy => "==>",
            Self::Link => "~~>",
        }
    }
}

impl Display for AtomMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomMode::FileCopy => write!(f, "copy"),
            AtomMode::Link => write!(f, "link"),
        }
    }
}
