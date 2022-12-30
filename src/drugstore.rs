use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Drugstore {
    /// a map of name -> upward dependencies, up to the root
    pub env: HashMap<String, HashSet<String>>,
    pub pills: HashMap<String, Pill>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pill {
    pub drips: Vec<Drip>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Drip {
    /// env is resolved to trivial form during parsing
    pub env: HashSet<String>,
    pub root: Atom,
    /// variants
    #[serde(flatten)]
    pub var: DripVariant,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DripVariant {
    GitModule {
        remote: String,
    },
    UnderManage {
        /// Atoms are incremented from drips but dirs aren't expanded
        stem: Vec<Atom>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
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

impl FromStr for Drugstore {
    type Err = anyhow::Error;

    fn from_str(buf: &str) -> anyhow::Result<Self> {
        let toml: toml::Value = toml::from_str(&buf)?;

        let mut env = HashMap::new();
        fn register_env<'e>(
            env: &mut HashMap<String, HashSet<String>>,
            worklist: &mut Vec<&'e str>,
            toml: &'e toml::Value,
        ) {
            fn register<'e>(
                env: &mut HashMap<String, HashSet<String>>,
                worklist: &Vec<&'e str>,
                s: &'e str,
            ) {
                env.entry(s.to_owned())
                    .or_default()
                    .extend(worklist.clone().into_iter().map(ToOwned::to_owned))
            };
            if let Some(s) = toml.as_str() {
                register(env, worklist, s);
            } else if let Some(m) = toml.as_table() {
                for (k, v) in m {
                    register(env, worklist, k);
                    worklist.push(k);
                    register_env(env, worklist, v);
                    worklist.pop();
                }
            }
        }
        register_env(&mut env, &mut Vec::new(), &toml["env"]);

        crate::utils::passed_tutorial(&toml)?;

        Ok(Self {
            env,
            pills: HashMap::new(),
        })
    }
}
