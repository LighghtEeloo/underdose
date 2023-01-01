use crate::{utils, Machine};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Serialize, Debug)]
pub struct Drugstore {
    pub env: EnvSet,
    pub pills: IndexMap<String, Pill>,
}

/// a map of name -> upward dependencies, up to the root
#[derive(Serialize, Debug)]
pub struct EnvMap {
    pub map: HashMap<String, HashSet<String>>,
}

impl EnvMap {
    pub fn resolve(&self, machine: &Machine) -> anyhow::Result<EnvSet> {
        let mut res = HashSet::new();
        for tag in &machine.env {
            let deps = self.map.get(tag).ok_or_else(|| {
                anyhow::anyhow!(
                    "tag {} is not defined in env dependency map",
                    tag
                )
            })?;
            res.extend(deps.to_owned());
        }
        Ok(EnvSet { set: res })
    }
}

/// a set of machine possesed envs
#[derive(Serialize, Debug)]
pub struct EnvSet {
    pub set: HashSet<String>,
}

impl EnvSet {
    pub fn check(&self, tag: &str) -> bool {
        self.set.contains(tag)
    }
    pub fn check_all(&self, tags: &HashSet<String>) -> bool {
        tags.iter().all(|tag| self.check(tag))
    }
}

#[derive(Serialize, Debug)]
pub struct Pill {
    pub name: String,
    pub drip: Drip,
}

impl Pill {
    pub fn non_empty(&self) -> bool {
        !matches!(
            self.drip,
            Drip {
                root: None,
                inner: None
            }
        )
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Drip {
    pub root: Option<Atom>,
    /// variants
    pub inner: Option<DripInner>,
}

#[derive(Serialize, Debug, Clone)]
pub enum DripInner {
    GitModule {
        /// url to remote git repo
        remote: String,
    },
    Addicted {
        /// Atoms are incremented from drips but dirs aren't expanded
        stem: Vec<Atom>,
        /// ignore
        ignore: Vec<String>,
    },
}

struct DripApplyIncr<'a> {
    pub drip: Drip,
    pub envset: &'a EnvSet,
}

impl<'a> DripApplyIncr<'a> {
    fn new(envset: &'a EnvSet) -> Self {
        DripApplyIncr {
            drip: Drip {
                root: None,
                inner: None,
            },
            envset,
        }
    }
    fn apply_force(&mut self, drip: Drip) -> anyhow::Result<()> {
        use DripInner::*;
        self.drip.root = match (drip.root, self.drip.root.clone()) {
            (Some(_), Some(_)) => {
                Err(anyhow::anyhow!("root set multiple times"))?
            }
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
        self.drip.inner = match (drip.inner, self.drip.inner.clone()) {
            (
                Some(Addicted {
                    stem: new_stem,
                    ignore: new_ignore,
                }),
                Some(Addicted {
                    mut stem,
                    mut ignore,
                }),
            ) => {
                stem.extend(new_stem);
                ignore.extend(new_ignore);
                Some(Addicted { stem, ignore })
            }
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
        Ok(())
    }
    fn apply_incr(
        mut self, drips: Vec<(HashSet<String>, Drip)>,
    ) -> anyhow::Result<Drip> {
        for (tags, drip) in drips {
            if self.envset.check_all(&tags) {
                self.apply_force(drip)?;
            }
        }
        Ok(self.drip)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Atom {
    pub site: PathBuf,
    pub repo: PathBuf,
    pub mode: AtomMode,
}

#[derive(Serialize, Debug, Clone)]
pub struct QuasiAtom {
    pub site: Option<PathBuf>,
    pub repo: Option<PathBuf>,
    pub mode: Option<AtomMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomMode {
    #[serde(rename = "copy")]
    FileCopy,
    #[serde(rename = "link")]
    Link,
}

impl Display for AtomMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomMode::FileCopy => write!(f, "cp"),
            AtomMode::Link => write!(f, "ln"),
        }
    }
}

mod parse {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Drugstore {
        pub env: toml::Value,
        pub pill: Vec<Pill>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Pill {
        pub name: String,
        pub drip: Vec<Drip>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Drip {
        /// `tags` is resolved to trivial form during parsing
        #[serde(alias = "env", default)]
        pub tags: HashSet<String>,
        pub root: Option<Atom>,
        /// variants
        #[serde(flatten)]
        pub inner: DripInner,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    #[serde(deny_unknown_fields)]
    pub enum DripInner {
        GitModule {
            /// url to remote git repo
            remote: String,
        },
        Addicted {
            /// Atoms are incremented from drips but dirs aren't expanded
            stem: Option<Vec<Atom>>,
            /// ignore
            ignore: Option<Vec<String>>,
        },
        Empty,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    #[serde(deny_unknown_fields)]
    pub enum Atom {
        Plain(String),
        Rich {
            site: Option<PathBuf>,
            repo: Option<PathBuf>,
            mode: Option<AtomMode>,
        },
    }
}

impl TryFrom<(&str, &Machine)> for Drugstore {
    type Error = anyhow::Error;

    fn try_from((buf, machine): (&str, &Machine)) -> anyhow::Result<Self> {
        let conf: parse::Drugstore = toml::from_str(buf)?;
        (conf, machine).try_into()
    }
}

impl TryFrom<(parse::Drugstore, &Machine)> for Drugstore {
    type Error = anyhow::Error;

    fn try_from(
        (conf, machine): (parse::Drugstore, &Machine),
    ) -> anyhow::Result<Self> {
        let mut map = HashMap::new();
        fn register_env<'e>(
            env: &mut HashMap<String, HashSet<String>>,
            worklist: &mut Vec<&'e str>, toml: &'e toml::Value,
        ) {
            fn register<'e>(
                env: &mut HashMap<String, HashSet<String>>,
                worklist: &'e [&'e str], s: &'e str,
            ) {
                env.entry(s.to_owned())
                    .or_default()
                    .extend(worklist.iter().map(|s| s.to_string()))
            }
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
        register_env(&mut map, &mut Vec::new(), &conf.env);
        let env = EnvMap { map }.resolve(machine)?;

        let mut pills = IndexMap::new();
        for pill in conf.pill {
            if pills.contains_key(&pill.name) {
                Err(anyhow::anyhow!("duplicated pill name"))?
            }

            let mut drips = Vec::new();
            for conf in pill.drip {
                drips.push((
                    conf.tags.to_owned(),
                    (conf, &pill.name, machine).try_into()?,
                ))
            }

            let pill = Pill {
                name: pill.name.to_owned(),
                drip: DripApplyIncr::new(&env).apply_incr(drips)?,
            };

            if pill.non_empty() {
                pills.insert(pill.name.to_owned(), pill);
            } else {
                log::info!("ignored empty pill <{}>", pill.name)
            }
        }

        Ok(Drugstore { env, pills })
    }
}

impl TryFrom<(parse::Drip, &String, &Machine)> for Drip {
    type Error = anyhow::Error;

    fn try_from(
        (conf, name, machine): (parse::Drip, &String, &Machine),
    ) -> anyhow::Result<Self> {
        let tags = conf.tags;

        let root = if let Some(root) = conf.root {
            let quasi: QuasiAtom = root.try_into()?;
            Some(Atom {
                site: utils::expand_path(
                    quasi
                        .site
                        .ok_or_else(|| anyhow::anyhow!("no site found"))?,
                )?,
                repo: utils::expand_path(
                    machine
                        .repo
                        .join(quasi.repo.unwrap_or_else(|| name.into())),
                )?,
                mode: quasi.mode.unwrap_or(machine.sync),
            })
        } else {
            None
        };

        use parse::DripInner::*;
        let inner = match conf.inner {
            GitModule { remote } => Some(DripInner::GitModule { remote }),
            Addicted { stem, ignore } => {
                let mut new_stem = Vec::new();
                for conf in stem.unwrap_or_default() {
                    let quasi: QuasiAtom = conf.try_into()?;
                    let site = quasi
                        .site
                        .ok_or_else(|| anyhow::anyhow!("no site found"))?;
                    new_stem.push(Atom {
                        site: site.clone(),
                        repo: quasi.repo.unwrap_or(site),
                        mode: quasi.mode.unwrap_or(machine.sync),
                    })
                }
                Some(DripInner::Addicted {
                    stem: new_stem,
                    ignore: ignore.unwrap_or_default(),
                })
            }
            Empty => None,
        };

        Ok(Self { root, inner })
    }
}

impl TryFrom<parse::Atom> for QuasiAtom {
    type Error = anyhow::Error;

    fn try_from(conf: parse::Atom) -> anyhow::Result<Self> {
        match conf {
            parse::Atom::Plain(s) => Ok(QuasiAtom {
                site: Some(s.into()),
                repo: None,
                mode: None,
            }),
            parse::Atom::Rich { site, repo, mode } => {
                Ok(QuasiAtom { site, repo, mode })
            }
        }
    }
}
