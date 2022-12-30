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

use crate::AtomMode;

#[derive(Serialize, Deserialize, Debug)]
pub struct Machine {
    pub env: HashSet<String>,
    pub repo: PathBuf,
    pub sync: AtomMode,
    pub undo: usize,
    pub glob: bool,
    pub git: bool,
    pub symlink: bool,
    pub cleanup_site: bool,
    pub cleanup_repo: bool,
}

impl FromStr for Machine {
    type Err = anyhow::Error;

    fn from_str(buf: &str) -> anyhow::Result<Self> {
        let toml: toml::Value = toml::from_str(&buf)?;
        let env = toml["env"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("env is not an array"))?
            .into_iter()
            .map(|e| match e.as_str() {
                Some(s) => Ok(s.to_string()),
                None => Err(anyhow::anyhow!("env item is not a string")),
            })
            .collect::<anyhow::Result<HashSet<String>>>()?;
        let repo = PathBuf::from(
            toml["repo"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("repo is not a string"))?,
        );
        if !repo.exists() {
            Err(anyhow::anyhow!("repo path does not exist"))?;
        }

        let defaults = &toml["defaults"];
        let sync = toml::from_str(&format!(
            "'{}'",
            defaults["sync"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("defaults.sync is missing"))?
        ))?;
        let undo = {
            let undo = defaults["undo"]
                .as_integer()
                .ok_or_else(|| anyhow::anyhow!("defaults.undo is missing"))?;
            if undo < 0 {
                0
            } else {
                undo as usize
            }
        };

        let enabled = &toml["enabled"];
        let glob = enabled["glob"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("enabled.glob is missing"))?;
        let git = enabled["git"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("enabled.git is missing"))?;
        let symlink = enabled["symlink"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("enabled.symlink is missing"))?;

        let cleanup = &toml["cleanup"]["empty_dir"];
        let cleanup_site = cleanup["site"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("cleanup.empty_dir.site is missing"))?;
        let cleanup_repo = cleanup["repo"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("cleanup.empty_dir.repo is missing"))?;

        crate::utils::passed_tutorial(&toml)?;

        Ok(Self {
            env,
            repo,
            sync,
            undo,
            glob,
            git,
            symlink,
            cleanup_site,
            cleanup_repo,
        })
    }
}
