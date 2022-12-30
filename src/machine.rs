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

// impl TryFrom<&str> for Machine {
//     type Error = anyhow::Error;

//     fn try_from(buf: &str) -> anyhow::Result<Self> {
//         let toml: toml::Value = toml::from_str(&buf)?;
//         let env = toml["env"]
//             .as_array()
//             .ok_or_else(|| anyhow::anyhow!("env is not an array"))?
//             .into_iter()
//             .map(|e| match e.as_str() {
//                 Some(s) => Ok(s.to_string()),
//                 None => Err(anyhow::anyhow!("env item is not a string")),
//             })
//             .collect::<anyhow::Result<HashSet<String>>>()?;
//         let repo = utils::expand_path(
//             toml["repo"]
//                 .as_str()
//                 .ok_or_else(|| anyhow::anyhow!("repo is not a string"))?,
//         );
//         if !repo.exists() {
//             Err(anyhow::anyhow!("repo path does not exist"))?;
//         }

//         let defaults = &toml["defaults"];
//         let sync = toml::from_str(&format!(
//             "'{}'",
//             defaults["sync"]
//                 .as_str()
//                 .ok_or_else(|| anyhow::anyhow!("defaults.sync is missing"))?
//         ))?;
//         let undo = {
//             let undo = defaults["undo"]
//                 .as_integer()
//                 .ok_or_else(|| anyhow::anyhow!("defaults.undo is missing"))?;
//             if undo < 0 {
//                 0
//             } else {
//                 undo as usize
//             }
//         };
//         let ignore = {
//             let ignore = defaults["ignore"]
//                 .as_array()
//                 .ok_or_else(|| anyhow::anyhow!("defaults.ignore is missing"))?;
//             ignore
//                 .into_iter()
//                 .map(|i| {
//                     let i = i
//                         .as_str()
//                         .ok_or_else(|| anyhow::anyhow!("defaults.ignore item is not a string"))?;
//                     Ok(utils::expand_path(i))
//                 })
//                 .collect::<anyhow::Result<Vec<PathBuf>>>()?
//         };

//         let features = &toml["features"];
//         let submodule = features["submodule"]
//             .as_bool()
//             .ok_or_else(|| anyhow::anyhow!("features.submodule is missing"))?;
//         let glob = features["glob"]
//             .as_bool()
//             .ok_or_else(|| anyhow::anyhow!("features.glob is missing"))?;
//         let symlink = features["symlink"]
//             .as_bool()
//             .ok_or_else(|| anyhow::anyhow!("features.symlink is missing"))?;

//         let cleanup = &toml["cleanup"]["empty_dir"];
//         let cleanup_site = cleanup["site"]
//             .as_bool()
//             .ok_or_else(|| anyhow::anyhow!("cleanup.empty_dir.site is missing"))?;
//         let cleanup_repo = cleanup["repo"]
//             .as_bool()
//             .ok_or_else(|| anyhow::anyhow!("cleanup.empty_dir.repo is missing"))?;

//         crate::utils::passed_tutorial(&toml)?;

//         Ok(Self {
//             env,
//             repo,
//             sync,
//             undo,
//             ignore,
//             submodule,
//             glob,
//             symlink,
//             cleanup_site,
//             cleanup_repo,
//         })
//     }
// }

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
            repo: utils::expand_path(&repo),
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
