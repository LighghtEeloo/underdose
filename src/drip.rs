use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Drip {
    /// where the root of site is, globally
    pub site: PathBuf,
    /// where the root of drip is, relative to repo root
    pub rel_repo: PathBuf,
    /// tasks to complete
    pub arrows: Vec<Arrow>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Arrow {
    /// where the site is, relative to drip root
    #[serde(rename = "site")]
    pub rel_site: PathBuf,
    pub src: ArrowSrc,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum ArrowSrc {
    #[serde(rename = "git")]
    Git(String),
    #[serde(rename = "link")]
    Link(PathBuf),
    #[serde(rename = "collector")]
    Collector,
}

impl Display for ArrowSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrowSrc::Git(remote) => write!(f, "git({})", remote),
            ArrowSrc::Link(repo) => write!(f, "ln({})", repo.display()),
            ArrowSrc::Collector => write!(f, "collector"),
        }
    }
}
