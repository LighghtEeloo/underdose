use crate::{utils, Machine};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Drugstore {
    pub env: EnvMap,
    pub pills: HashMap<String, Pill>,
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
                anyhow::anyhow!("tag {} is not defined in env dependency map", tag)
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
    pub drips: Vec<Drip>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Drip {
    /// `tags` is resolved to trivial form during parsing
    pub tags: HashSet<String>,
    pub root: Option<Atom>,
    /// variants
    #[serde(flatten)]
    pub var: Option<DripVariant>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DripVariant {
    GitModule {
        remote: String,
    },
    Addicted {
        /// Atoms are incremented from drips but dirs aren't expanded
        stems: Vec<Atom>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Atom {
    pub site: PathBuf,
    pub repo: PathBuf,
    pub mode: AtomMode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuasiAtom {
    pub site: Option<PathBuf>,
    pub repo: Option<PathBuf>,
    pub mode: Option<AtomMode>,
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

impl TryFrom<(&toml::Value, &Machine)> for Drugstore {
    type Error = anyhow::Error;

    fn try_from((toml, machine): (&toml::Value, &Machine)) -> anyhow::Result<Self> {
        let mut envmap = HashMap::new();
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
        register_env(&mut envmap, &mut Vec::new(), &toml["env"]);
        let env = EnvMap { map: envmap };

        let envset = env.resolve(machine)?;

        let pills = if let Some(pills) = toml.get("pill") {
            if let Some(pills_raw) = pills.as_array() {
                let mut pills = HashMap::new();
                for pill in pills_raw {
                    let name = pill["name"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("pill name is not string"))?
                        .to_owned();

                    if pills.contains_key(&name) {
                        Err(anyhow::anyhow!("duplicated pill name"))?
                    }

                    let drips_raw = pill["drip"]
                        .as_array()
                        .ok_or_else(|| anyhow::anyhow!("drips are not in an array"))?;

                    let mut drips = Vec::new();
                    for drip in drips_raw {
                        drips.push((drip, &name, machine).try_into()?)
                    }
                    drips = DripApplyIncr::new(&envset).apply_incr(drips);

                    pills.insert(name, Pill { drips });
                }
                pills
            } else {
                Err(anyhow::anyhow!("pills are not in an array"))?
            }
        } else {
            HashMap::new()
        };

        crate::utils::passed_tutorial(&toml)?;

        Ok(Self { env, pills })
    }
}

pub struct DripApplyIncr<'a> {
    pub drip: Drip,
    pub envset: &'a EnvSet,
}

impl<'a> DripApplyIncr<'a> {
    pub fn new(envset: &'a EnvSet) -> Self {
        DripApplyIncr {
            drip: Drip {
                tags: HashSet::new(),
                root: None,
                var: None,
            },
            envset,
        }
    }
    pub fn apply(&mut self, drip: Drip) {
        use DripVariant::*;
        self.drip.tags.extend(drip.tags);
        self.drip.root = drip.root.or(self.drip.root.clone());
        self.drip.var = match (drip.var, self.drip.var.clone()) {
            (Some(Addicted { stems: new }), Some(Addicted { stems: mut stem })) => {
                stem.extend(new);
                Some(Addicted { stems: stem })
            }
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
    }
    pub fn apply_incr(mut self, drips: Vec<Drip>) -> Vec<Drip> {
        let mut res = Vec::new();
        for drip in drips {
            if self.envset.check_all(&drip.tags) {
                self.apply(drip);
                res.push(self.drip.clone());
            }
        }
        res
    }
}

impl TryFrom<(&toml::Value, &String, &Machine)> for Drip {
    type Error = anyhow::Error;

    fn try_from((toml, name, machine): (&toml::Value, &String, &Machine)) -> anyhow::Result<Self> {
        let env = toml.get("env").map_or_else(
            || Ok(HashSet::new()),
            |env| {
                env.as_array()
                    .ok_or_else(|| anyhow::anyhow!("env is not an array"))?
                    .into_iter()
                    .map(|e| match e.as_str() {
                        Some(s) => Ok(s.to_string()),
                        None => Err(anyhow::anyhow!("env item is not a string")),
                    })
                    .collect::<anyhow::Result<HashSet<String>>>()
            },
        )?;

        let root = if let Some(root) = toml.get("root") {
            let quasi = QuasiAtom::try_from(root)?;
            Some(Atom {
                site: utils::expand_path(
                    quasi.site.ok_or_else(|| anyhow::anyhow!("no site found"))?,
                )?,
                repo: utils::expand_path(quasi.repo.unwrap_or_else(|| machine.repo.join(name)))?,
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
                    let quasi = QuasiAtom::try_from(stem)?;
                    stems.push(Atom {
                        site: utils::expand_path(
                            quasi
                                .site
                                .clone()
                                .ok_or_else(|| anyhow::anyhow!("no site found"))?,
                        )?,
                        repo: utils::expand_path(
                            quasi.repo.unwrap_or_else(|| quasi.site.unwrap()),
                        )?,
                        mode: quasi.mode.unwrap_or_else(|| machine.sync),
                    });
                }
                Some(stems)
            } else {
                Err(anyhow::anyhow!("stem must be an array"))?
            }
        } else {
            None
        };

        use DripVariant::*;
        let var = match (remote, stems) {
            (Some(_), Some(_)) => Err(anyhow::anyhow!(
                "can't have both git and stem at the same time"
            ))?,
            (Some(remote), None) => Some(GitModule { remote }),
            (None, Some(stems)) => Some(Addicted { stems }),
            (None, None) => None,
        };

        Ok(Drip {
            tags: env,
            root,
            var,
        })
    }
}

// impl TryFrom<(&toml::Value, &String, &Machine)> for Atom {
//     type Error = anyhow::Error;

//     fn try_from(
//         (value, name, machine): (&toml::Value, &String, &Machine),
//     ) -> Result<Self, Self::Error> {
//         let quasi = QuasiAtom::try_from(value)?;
//         Ok(Atom {
//             site: quasi.site.ok_or_else(|| anyhow::anyhow!("no site found"))?,
//             repo: quasi.repo.unwrap_or_else(|| machine.repo.join(name)),
//             mode: quasi.mode.unwrap_or_else(|| machine.sync),
//         })
//     }
// }

impl TryFrom<&toml::Value> for QuasiAtom {
    type Error = anyhow::Error;

    fn try_from(value: &toml::Value) -> anyhow::Result<Self> {
        if let Some(value) = value.as_str() {
            Ok(QuasiAtom {
                site: Some(PathBuf::from(value)),
                repo: None,
                mode: None,
            })
        } else if let Some(value) = value.as_table() {
            fn as_path(
                entry: &str,
                value: &toml::value::Map<String, toml::Value>,
            ) -> Option<PathBuf> {
                value
                    .get(entry)
                    .map(|site| match site.as_str() {
                        Some(site) => Some(PathBuf::from(site)),
                        None => None,
                    })
                    .flatten()
            }
            Ok(QuasiAtom {
                site: as_path("site", value),
                repo: as_path("repo", value),
                mode: None,
            })
        } else {
            Err(anyhow::anyhow!("root is neither a path nor an atom"))?
        }
    }
}
