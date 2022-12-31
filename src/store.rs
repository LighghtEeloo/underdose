use crate::{utils, Machine};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Drugstore {
    pub env: EnvMap,
    pub pills: IndexMap<String, Pill>,
}

/// a map of name -> upward dependencies, up to the root
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Pill {
    pub name: String,
    pub drip: Drip,
}

impl Pill {
    pub fn non_empty(&self) -> bool {
        !self.drip.root.is_none() && !self.drip.inner.is_none()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Drip {
    pub root: Option<Atom>,
    /// variants
    #[serde(flatten)]
    pub inner: Option<DripInner>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DripConf {
    /// `tags` is resolved to trivial form during parsing
    pub tags: HashSet<String>,
    pub root: Option<Atom>,
    /// variants
    #[serde(flatten)]
    pub inner: Option<DripInner>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Atom {
    pub site: PathBuf,
    pub repo: PathBuf,
    pub mode: AtomMode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomConf {
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

    impl TryFrom<(&toml::Value, &Machine)> for Drugstore {
        type Error = anyhow::Error;

        fn try_from(
            (toml, machine): (&toml::Value, &Machine),
        ) -> anyhow::Result<Self> {
            let mut envmap = HashMap::new();
            fn register_env<'e>(
                env: &mut HashMap<String, HashSet<String>>,
                worklist: &mut Vec<&'e str>, toml: &'e toml::Value,
            ) {
                fn register<'e>(
                    env: &mut HashMap<String, HashSet<String>>,
                    worklist: &Vec<&'e str>, s: &'e str,
                ) {
                    env.entry(s.to_owned()).or_default().extend(
                        worklist.clone().into_iter().map(ToOwned::to_owned),
                    )
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
            register_env(&mut envmap, &mut Vec::new(), &toml["env"]);
            let env = EnvMap { map: envmap };

            let envset = env.resolve(machine)?;

            let pills = if let Some(pills) = toml.get("pill") {
                if let Some(pills_raw) = pills.as_array() {
                    let mut pills = IndexMap::new();
                    for pill in pills_raw {
                        let name = pill["name"]
                            .as_str()
                            .ok_or_else(|| {
                                anyhow::anyhow!("pill name is not string")
                            })?
                            .to_owned();

                        if pills.contains_key(&name) {
                            Err(anyhow::anyhow!("duplicated pill name"))?
                        }

                        let drips_raw =
                            pill["drip"].as_array().ok_or_else(|| {
                                anyhow::anyhow!("drips are not in an array")
                            })?;

                        let mut drips = Vec::new();
                        for drip_raw in drips_raw {
                            let conf: DripConf =
                                (drip_raw, &name, machine).try_into()?;
                            drips.push((
                                conf.tags,
                                Drip {
                                    root: conf.root,
                                    inner: conf.inner,
                                },
                            ))
                        }

                        let pill = Pill {
                            name: name.to_owned(),
                            drip: DripApplyIncr::new(&envset)
                                .apply_incr(drips)?,
                        };

                        if pill.non_empty() {
                            pills.insert(name, pill);
                        } else {
                            log::info!("ignored empty pill <{}>", name)
                        }
                    }
                    pills
                } else {
                    Err(anyhow::anyhow!("pills are not in an array"))?
                }
            } else {
                IndexMap::new()
            };

            crate::utils::passed_tutorial(&toml)?;

            Ok(Self { env, pills })
        }
    }

    impl TryFrom<(&toml::Value, &String, &Machine)> for DripConf {
        type Error = anyhow::Error;

        fn try_from(
            (toml, name, machine): (&toml::Value, &String, &Machine),
        ) -> anyhow::Result<Self> {
            let tags = toml.get("env").map_or_else(
                || Ok(HashSet::new()),
                |env| {
                    env.as_array()
                        .ok_or_else(|| anyhow::anyhow!("env is not an array"))?
                        .into_iter()
                        .map(|e| match e.as_str() {
                            Some(s) => Ok(s.to_string()),
                            None => {
                                Err(anyhow::anyhow!("env item is not a string"))
                            }
                        })
                        .collect::<anyhow::Result<HashSet<String>>>()
                },
            )?;

            let root =
                if let Some(root) = toml.get("root") {
                    let quasi = AtomConf::try_from(root)?;
                    Some(Atom {
                        site: utils::expand_path(quasi.site.ok_or_else(
                            || anyhow::anyhow!("no site found"),
                        )?)?,
                        repo: utils::ensured_dir(
                            machine.repo.join(
                                quasi.repo.unwrap_or_else(|| name.into()),
                            ),
                        )?,
                        mode: quasi.mode.unwrap_or_else(|| machine.sync),
                    })
                } else {
                    None
                };

            let remote = if let Some(remote) = toml.get("remote") {
                remote.as_str().map(ToOwned::to_owned)
            } else {
                None
            };
            let stems = if let Some(stems) = toml.get("stem") {
                if let Some(stems_raw) = stems.as_array() {
                    let mut stems = Vec::new();
                    for stem in stems_raw {
                        let quasi = AtomConf::try_from(stem)?;
                        let site = quasi
                            .site
                            .ok_or_else(|| anyhow::anyhow!("no site found"))?;
                        stems.push(Atom {
                            site: utils::expand_path(site.clone())?,
                            repo: utils::expand_path(
                                quasi.repo.unwrap_or(site),
                            )?,
                            mode: quasi.mode.unwrap_or(machine.sync),
                        });
                    }
                    Some(stems)
                } else {
                    Err(anyhow::anyhow!("stem must be an array"))?
                }
            } else {
                None
            };
            let ignore = if let Some(ignore) = toml.get("ignore") {
                if let Some(ignore_raw) = ignore.as_array() {
                    let mut ignore = Vec::new();
                    for i in ignore_raw {
                        let i = i.as_str().ok_or_else(|| {
                            anyhow::anyhow!("ignore item is not a string")
                        })?;
                        ignore.push(i.to_owned());
                    }
                    Some(ignore)
                } else {
                    Err(anyhow::anyhow!("ignore must be an array"))?
                }
            } else {
                None
            };

            use DripInner::*;
            let var = match (remote, stems) {
                (Some(_), Some(_)) => Err(anyhow::anyhow!(
                    "can't have both git and stem at the same time"
                ))?,
                (Some(remote), None) => Some(GitModule { remote }),
                (None, Some(stem)) => Some(Addicted {
                    stem,
                    ignore: ignore.unwrap_or_default(),
                }),
                (None, None) => None,
            };

            Ok(Self {
                tags,
                root,
                inner: var,
            })
        }
    }

    impl TryFrom<&toml::Value> for AtomConf {
        type Error = anyhow::Error;

        fn try_from(value: &toml::Value) -> anyhow::Result<Self> {
            if let Some(value) = value.as_str() {
                Ok(AtomConf {
                    site: Some(PathBuf::from(value)),
                    repo: None,
                    mode: None,
                })
            } else if let Some(value) = value.as_table() {
                fn as_path(
                    entry: &str, value: &toml::value::Map<String, toml::Value>,
                ) -> Option<PathBuf> {
                    value
                        .get(entry)
                        .map(|site| match site.as_str() {
                            Some(site) => Some(PathBuf::from(site)),
                            None => None,
                        })
                        .flatten()
                }
                let mode = match value.get("mode") {
                    Some(mode) => match mode.as_str() {
                        Some("copy") => Some(AtomMode::FileCopy),
                        Some("link") => Some(AtomMode::Link),
                        _ => None,
                    },
                    None => None,
                };
                Ok(AtomConf {
                    site: as_path("site", value),
                    repo: as_path("repo", value),
                    mode,
                })
            } else {
                Err(anyhow::anyhow!("root is neither a path nor an atom"))?
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn atom_conf_parse() {
        let toml = r#"
            site = "site"
            repo = "repo"
            mode = "copy"
        "#;
        let conf: AtomConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.site, Some(PathBuf::from("site")));
        assert_eq!(conf.repo, Some(PathBuf::from("repo")));
        assert_eq!(conf.mode, Some(AtomMode::FileCopy));

        let toml = r#"
            site = "site"
            mode = "link"
        "#;
        let conf: AtomConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.site, Some(PathBuf::from("site")));
        assert_eq!(conf.repo, None);
        assert_eq!(conf.mode, Some(AtomMode::Link));

        let toml = r#"
            mode = "link"
        "#;
        let conf: AtomConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.site, None);
        assert_eq!(conf.repo, None);
        assert_eq!(conf.mode, Some(AtomMode::Link));
    }
}