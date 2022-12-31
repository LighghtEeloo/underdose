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

use crate::utils::IgnoreSetBuilder;
use crate::{store::AtomMode, utils};

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub env: HashSet<String>,
    pub repo: PathBuf,
    pub sync: AtomMode,
    pub undo: usize,
    pub ignore: IgnoreSetBuilder,
    pub submodule: bool,
    pub symlink: bool,
    pub cleanup_site: bool,
    pub cleanup_repo: bool,
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
            defaults: parse::Defaults { sync, undo, ignore },
            features: parse::Features { submodule, symlink },
            cleanup:
                parse::Cleanup {
                    empty_dir:
                        parse::CleanupEmptyDir {
                            site: cleanup_site,
                            repo: cleanup_repo,
                        },
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
            repo: utils::expand_path(&repo)?,
            sync,
            undo,
            ignore: IgnoreSetBuilder::new().chain(ignore.iter()),
            submodule,
            symlink,
            cleanup_site,
            cleanup_repo,
        })
    }
}

mod parse {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Machine {
        pub name: String,
        pub env: HashSet<String>,
        pub repo: PathBuf,
        pub defaults: Defaults,
        pub features: Features,
        pub cleanup: Cleanup,
        pub tutorial: Option<()>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Defaults {
        pub sync: AtomMode,
        pub undo: usize,
        pub ignore: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Features {
        pub submodule: bool,
        pub symlink: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Cleanup {
        pub empty_dir: CleanupEmptyDir,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CleanupEmptyDir {
        pub site: bool,
        pub repo: bool,
    }
}
