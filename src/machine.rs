use crate::utils::conf::IgnoreSetBuilder;
use crate::{store::AtomMode, utils};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub env: HashSet<String>,
    pub repo: PathBuf,
    pub sync: AtomMode,
    pub cache: Option<PathBuf>,
    pub undo: Option<usize>,
    pub ignore: IgnoreSetBuilder,
    pub overdose: bool,
    pub submodule: bool,
    pub symlink: bool,
}

mod parse {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Machine {
        pub name: String,
        pub env: HashSet<String>,
        pub repo: PathBuf,
        pub defaults: Defaults,
        pub features: Features,
        pub tutorial: Option<()>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Defaults {
        pub sync: AtomMode,
        pub cache: Option<PathBuf>,
        pub undo: Option<usize>,
        pub ignore: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Features {
        pub overdose: bool,
        pub submodule: bool,
        pub symlink: bool,
    }
}

impl TryFrom<&str> for Machine {
    type Error = anyhow::Error;

    fn try_from(buf: &str) -> anyhow::Result<Self> {
        let conf: parse::Machine = toml::from_str(buf)?;
        conf.try_into()
    }
}

impl TryFrom<parse::Machine> for Machine {
    type Error = anyhow::Error;

    fn try_from(
        parse::Machine {
            name,
            env,
            repo,
            defaults:
                parse::Defaults {
                    sync,
                    cache,
                    undo,
                    ignore,
                },
            features:
                parse::Features {
                    overdose,
                    submodule,
                    symlink,
                },
            tutorial,
        }: parse::Machine,
    ) -> Result<Self, Self::Error> {
        if let Some(_) = tutorial {
            Err(anyhow::anyhow!("tutorial has not been completed yet"))?;
        }

        Ok(Self {
            name,
            env,
            repo: utils::conf::expand_path(&repo)?,
            sync,
            cache,
            undo,
            ignore: IgnoreSetBuilder::new().chain(ignore.iter()),
            overdose,
            submodule,
            symlink,
        })
    }
}
