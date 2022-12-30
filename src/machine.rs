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

use crate::{utils, AtomMode};

#[derive(Serialize, Deserialize, Debug)]
pub struct Machine {
    pub env: HashSet<String>,
    pub repo: PathBuf,
    pub sync: AtomMode,
    pub undo: usize,
    pub ignore: Vec<PathBuf>,
    pub submodule: bool,
    pub glob: bool,
    pub symlink: bool,
    pub cleanup_site: bool,
    pub cleanup_repo: bool,
}

impl TryFrom<&str> for Machine {
    type Error = anyhow::Error;

    fn try_from(buf: &str) -> anyhow::Result<Self> {
        let conf: MachineConf = toml::from_str(buf)?;
        conf.try_into()
    }
}

impl TryFrom<MachineConf> for Machine {
    type Error = anyhow::Error;

    fn try_from(
        MachineConf {
            env,
            repo,
            defaults: DefaultsConf { sync, undo, ignore },
            features:
                FeaturesConf {
                    submodule,
                    glob,
                    symlink,
                },
            cleanup:
                CleanupConf {
                    empty_dir:
                        CleanupEmptyDirConf {
                            site: cleanup_site,
                            repo: cleanup_repo,
                        },
                },
            tutorial,
        }: MachineConf,
    ) -> Result<Self, Self::Error> {
        if let Some(_) = tutorial {
            Err(anyhow::anyhow!("tutorial has not been completed yet"))?;
        }

        Ok(Self {
            env,
            repo: utils::expand_path(&repo)?,
            sync,
            undo,
            ignore,
            submodule,
            glob,
            symlink,
            cleanup_site,
            cleanup_repo,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MachineConf {
    pub env: HashSet<String>,
    pub repo: PathBuf,
    pub defaults: DefaultsConf,
    pub features: FeaturesConf,
    pub cleanup: CleanupConf,
    pub tutorial: Option<()>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultsConf {
    pub sync: AtomMode,
    pub undo: usize,
    pub ignore: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeaturesConf {
    pub submodule: bool,
    pub glob: bool,
    pub symlink: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CleanupConf {
    pub empty_dir: CleanupEmptyDirConf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CleanupEmptyDirConf {
    pub site: bool,
    pub repo: bool,
}
