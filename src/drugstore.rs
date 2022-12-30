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

#[derive(Debug)]
pub struct Drugstore {
    /// a map of name -> upward dependencies, up to the root
    pub env: HashMap<String, HashSet<String>>,
    pub pills: HashMap<String, Pill>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pill {
    pub drips: Vec<Drip>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Drip {
    /// env is resolved to trivial form during parsing
    pub env: HashSet<String>,
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
    UnderManage {
        /// Atoms are incremented from drips but dirs aren't expanded
        stem: Vec<Atom>,
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

impl FromStr for Drugstore {
    type Err = anyhow::Error;

    fn from_str(buf: &str) -> anyhow::Result<Self> {
        let toml: toml::Value = toml::from_str(&buf)?;

        let mut env = HashMap::new();
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
        register_env(&mut env, &mut Vec::new(), &toml["env"]);

        let pills = if let Some(pills) = toml.get("pill") {
            if let Some(pills) = pills.as_array() {
                pills
                    .into_iter()
                    .map(|pill| {
                        let name = pill["name"]
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("pill name is not string"))?
                            .to_owned();
                        let mut drips = pill["drip"]
                            .as_array()
                            .ok_or_else(|| anyhow::anyhow!("drips are not in an array"))?
                            .into_iter()
                            .map(|drip| drip.try_into())
                            .collect::<anyhow::Result<Vec<Drip>>>()?;

                        drips = Drip::new().apply_incr(drips);

                        Ok((name, Pill { drips }))
                    })
                    .collect::<anyhow::Result<HashMap<String, Pill>>>()?
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

impl TryFrom<&toml::Value> for Drip {
    type Error = anyhow::Error;

    fn try_from(toml: &toml::Value) -> anyhow::Result<Self> {
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

        let root = if let Some(root) = toml.get("env") {
            Some(root.try_into()?)
        } else {
            None
        };

        todo!()
    }
}

impl Drip {
    pub fn new() -> Drip {
        Drip {
            env: HashSet::new(),
            root: None,
            var: None,
        }
    }
    pub fn apply(&mut self, drip: Drip) {
        use DripVariant::*;
        self.env.extend(drip.env);
        self.root = drip.root.or(self.root.clone());
        self.var = match (drip.var, self.var.clone()) {
            (Some(UnderManage { stem: new }), Some(UnderManage { mut stem })) => {
                stem.extend(new);
                Some(UnderManage { stem })
            }
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
    }
    pub fn apply_incr(mut self, mut drips: Vec<Drip>) -> Vec<Drip> {
        for drip in &mut drips {
            self.apply(drip.clone());
            *drip = self.clone();
        }
        drips
    }
}

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
