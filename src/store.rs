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
                anyhow::anyhow!("tag {} is not defined in env dependency map", tag)
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
        self.drip.root.is_some() || !self.drip.stem.is_empty()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Drip {
    pub root: Option<Atom>,
    /// Atoms are incremented from drips but dirs aren't expanded
    pub stem: Vec<Atom>,
    /// ignore
    pub ignore: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Atom {
    pub site: PathBuf,
    pub src: AtomSrc,
}

#[derive(Serialize, Debug, Clone)]
pub struct QuasiAtom {
    pub site: Option<PathBuf>,
    pub repo: Option<PathBuf>,
    pub mode: Option<AtomSrc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AtomSrc {
    #[serde(rename = "git")]
    Git(String),
    #[serde(rename = "link")]
    Link(PathBuf),
    #[serde(rename = "collector")]
    Collector,
}

impl Display for AtomSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomSrc::Git(remote) => write!(f, "git({})", remote),
            AtomSrc::Link(repo) => write!(f, "ln({})", repo.display()),
            AtomSrc::Collector => write!(f, "collector"),
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
        /// `tags` are resolved to trivial form during parsing
        #[serde(alias = "env", default)]
        pub tags: HashSet<String>,
        pub root: Option<Atom>,
        /// Atoms are incremented from drips
        pub stem: Option<Vec<Atom>>,
        /// ignore
        pub ignore: Option<Vec<String>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    #[serde(deny_unknown_fields)]
    pub enum Atom {
        Plain(String),
        Rich {
            site: Option<PathBuf>,
            repo: Option<PathBuf>,
            mode: Option<AtomSrc>,
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

    fn try_from((store, machine): (parse::Drugstore, &Machine)) -> anyhow::Result<Self> {
        let mut map = HashMap::new();
        fn register_env<'e>(
            env: &mut HashMap<String, HashSet<String>>, worklist: &mut Vec<&'e str>,
            toml: &'e toml::Value,
        ) {
            fn register<'e>(
                env: &mut HashMap<String, HashSet<String>>, worklist: &'e [&'e str], s: &'e str,
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
        register_env(&mut map, &mut Vec::new(), &store.env);
        let env = EnvMap { map }.resolve(machine)?;

        let mut pills = IndexMap::new();
        for pill in store.pill {
            if pills.contains_key(&pill.name) {
                Err(anyhow::anyhow!("duplicated pill name"))?
            }

            let mut drips = Vec::new();
            for drip in pill.drip {
                drips.push((
                    drip.tags.to_owned(),
                    (drip, &pill.name, machine).try_into()?,
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

struct DripApplyIncr<'a> {
    pub drip: Drip,
    pub envset: &'a EnvSet,
}

impl<'a> DripApplyIncr<'a> {
    fn new(envset: &'a EnvSet) -> Self {
        DripApplyIncr {
            drip: Drip {
                root: None,
                stem: Vec::new(),
                ignore: Vec::new(),
            },
            envset,
        }
    }
    fn apply_force(&mut self, drip: Drip) -> anyhow::Result<()> {
        self.drip.root = match (drip.root, self.drip.root.clone()) {
            (Some(_), Some(_)) => Err(anyhow::anyhow!("root set multiple times"))?,
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
        self.drip.stem.extend(drip.stem);
        self.drip.ignore.extend(drip.ignore);
        Ok(())
    }
    fn apply_incr(mut self, drips: Vec<(HashSet<String>, Drip)>) -> anyhow::Result<Drip> {
        for (tags, drip) in drips {
            if self.envset.check_all(&tags) {
                self.apply_force(drip)?;
            }
        }
        Ok(self.drip)
    }
}

impl TryFrom<(parse::Drip, &String, &Machine)> for Drip {
    type Error = anyhow::Error;

    fn try_from((drip, name, machine): (parse::Drip, &String, &Machine)) -> anyhow::Result<Self> {
        let root = if let Some(root) = drip.root {
            let quasi: QuasiAtom = root.try_into()?;
            Some(Atom {
                site: utils::path::expand_home(
                    quasi.site.ok_or_else(|| anyhow::anyhow!("no site found"))?,
                ),
                src: quasi.mode.unwrap_or_else(|| {
                    AtomSrc::Link(utils::path::expand_home(
                        machine
                            .local
                            .join(quasi.repo.unwrap_or_else(|| name.into())),
                    ))
                }),
            })
        } else {
            None
        };

        let mut stem = Vec::new();
        for conf in drip.stem.unwrap_or_default() {
            let quasi: QuasiAtom = conf.try_into()?;
            let site = quasi.site.ok_or_else(|| anyhow::anyhow!("no site found"))?;
            stem.push(Atom {
                site: site.clone(),
                src: quasi
                    .mode
                    .unwrap_or_else(|| AtomSrc::Link(quasi.repo.unwrap_or(site))),
            })
        }
        let ignore = drip.ignore.unwrap_or_default();

        Ok(Self { root, stem, ignore })
    }
}

impl TryFrom<parse::Atom> for QuasiAtom {
    type Error = anyhow::Error;

    fn try_from(atom: parse::Atom) -> anyhow::Result<Self> {
        match atom {
            parse::Atom::Plain(s) => Ok(QuasiAtom {
                site: Some(s.into()),
                repo: None,
                mode: None,
            }),
            parse::Atom::Rich { site, repo, mode } => Ok(QuasiAtom { site, repo, mode }),
        }
    }
}
