use crate::{Arrow, Drip, Machine};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Drugstore {
    pub env: EnvSet,
    pub pills: IndexMap<String, Drip>,
}

/// a map of name -> upward dependencies, up to the root
#[derive(Debug)]
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
            res.insert(tag.to_owned());
            res.extend(deps.to_owned());
        }
        Ok(EnvSet { set: res })
    }
}

/// a set of machine possesed envs
#[derive(Debug)]
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

#[derive(Debug)]
pub struct Pill {
    pub name: String,
    pub drip: Drip,
}

impl Pill {
    pub fn non_empty(&self) -> bool {
        !self.drip.arrows.is_empty()
    }
}

mod parse {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Drugstore {
        pub env: toml::Value,
        pub pill: Vec<Pill>,
        pub tutorial: Option<()>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Pill {
        pub name: String,
        #[serde(alias = "drip")]
        pub drips: Vec<Drip>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Drip {
        /// `tags` are resolved to trivial form during parsing
        #[serde(alias = "env", default)]
        pub tags: HashSet<String>,
        /// where the root of site is, globally
        pub site: Option<PathBuf>,
        /// where the root of drip is, relative to repo root
        pub repo: Option<PathBuf>,
        /// tasks to complete
        #[serde(alias = "arrow", default)]
        pub arrows: Vec<Arrow>,
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
        if let Some(_) = store.tutorial {
            Err(anyhow::anyhow!("tutorial has not been completed yet"))?;
        }
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
                Err(anyhow::anyhow!("duplicated pill name: {}", pill.name))?
            }

            let name = pill.name.clone();
            match DripApplyIncr::new(&env).apply(pill) {
                Ok(pill) => {
                    if pill.non_empty() {
                        pills.insert(pill.name.to_owned(), pill.drip);
                    } else {
                        log::info!("ignored empty pill <{}>", name)
                    }
                }
                Err(e) => {
                    log::warn!("ignored pill <{}>: {}", name, e);
                }
            }
        }

        Ok(Drugstore { env, pills })
    }
}

struct DripApplyIncr<'a> {
    drip: parse::Drip,
    pub envset: &'a EnvSet,
}

impl<'a> DripApplyIncr<'a> {
    fn new(envset: &'a EnvSet) -> Self {
        DripApplyIncr {
            drip: parse::Drip {
                tags: HashSet::new(),
                site: None,
                repo: None,
                arrows: Vec::new(),
            },
            envset,
        }
    }
    fn apply_unchecked(&mut self, drip: parse::Drip) -> anyhow::Result<()> {
        self.drip.site = match (drip.site, self.drip.site.clone()) {
            (Some(_), Some(_)) => Err(anyhow::anyhow!("site set multiple times"))?,
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
        self.drip.repo = match (drip.repo, self.drip.repo.clone()) {
            (Some(_), Some(_)) => Err(anyhow::anyhow!("repo set multiple times"))?,
            (new @ Some(_), _) => new,
            (None, old) => old,
        };
        self.drip.arrows.extend(drip.arrows);
        Ok(())
    }
    pub fn apply(mut self, pill: parse::Pill) -> anyhow::Result<Pill> {
        let mut cnt = 0;
        for drip in pill.drips {
            if self.envset.check_all(&drip.tags) {
                self.apply_unchecked(drip)?;
                cnt += 1;
            }
        }

        if cnt == 0 {
            // no drip applied, no need to check
            return Ok(Pill {
                name: pill.name,
                drip: Drip::default(),
            });
        }

        // check site; set default repo if not set
        let site = (self.drip.site).ok_or_else(|| {
            anyhow::anyhow!(
                "site not set in pill <{}>, please set it in one of the drips",
                pill.name
            )
        })?;
        let rel_repo = self.drip.repo.unwrap_or(PathBuf::from(pill.name.clone()));
        let arrows = self.drip.arrows;
        Ok(Pill {
            name: pill.name,
            drip: Drip {
                site,
                rel_repo,
                arrows,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_store() {
        let content = crate::utils::tests::remove_tutorial(crate::utils::conf::DRUGSTORE_TOML);

        // parse with linux
        let machine = crate::Machine {
            env: ["linux".to_owned()].into(),
            ..Default::default()
        };
        let store = crate::Drugstore::try_from((&content[..], &machine)).unwrap();
        println!("linux: {:#?}", store);

        // parse with mac
        let machine = crate::Machine {
            env: ["mac".to_owned()].into(),
            ..Default::default()
        };
        let store = crate::Drugstore::try_from((&content[..], &machine)).unwrap();
        println!("mac: {:#?}", store);
    }
}
