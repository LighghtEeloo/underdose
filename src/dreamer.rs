use crate::utils::UNDERDOSE_STATICS;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Dreamer {
    pub map: HashMap<String, DumpPill>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DumpPill {
    pub name: String,
    pub root: PathBuf,
    pub stems: Vec<PathBuf>,
    pub versions: BTreeSet<u128>,
}

impl Dreamer {
    pub fn path() -> &'static Path {
        &UNDERDOSE_STATICS.dump.as_path()
    }
    pub fn index_path() -> PathBuf {
        Self::path().join("index.json")
    }
    pub fn uid_path(name: impl AsRef<str>, uid: u128) -> PathBuf {
        Self::path().join(name.as_ref()).join(format!("{}", uid))
    }

    pub fn new() -> Self {
        let Ok(content) = std::fs::read_to_string(Self::index_path()) else {
            return Self::default();
        };
        let Ok(res) = serde_json::from_str(&content) else {
            return Self::default();
        };
        res
    }

    pub fn dump(&mut self, name: String, root: PathBuf, stems: Vec<PathBuf>) -> anyhow::Result<()> {
        let uid = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u128;
        let path = Self::uid_path(&name, uid);
        std::fs::create_dir_all(&path)?;
        for stem in stems.iter() {
            std::fs::rename(root.join(stem), path.join(stem))?;
        }
        self.map
            .entry(name.clone())
            .or_insert(DumpPill {
                name,
                root,
                stems,
                versions: BTreeSet::new(),
            })
            .versions
            .insert(uid);
        Ok(())
    }

    pub fn recover_last(&mut self, name: String) -> anyhow::Result<()> {
        let Some(pill) = self.map.get(&name).cloned() else {
            return Ok(())
        };
        let Some(uid) = pill.versions.iter().last() else {
            return Ok(())
        };

        // check if need to dump
        let mut need_dump = false;
        for stem in pill.stems.iter() {
            if pill.root.join(stem).exists() {
                need_dump = true;
                break;
            }
        }
        if need_dump {
            self.dump(name.clone(), pill.root.clone(), pill.stems.clone())?;
        }

        // recover
        let path = Self::uid_path(&name, *uid);
        std::fs::create_dir_all(&pill.root)?;
        for stem in pill.stems.iter() {
            std::fs::rename(path.join(stem), pill.root.join(stem))?;
        }
        Ok(())
    }

    fn remove_from_version(
        &mut self, name: String, mut f: impl FnMut(&mut BTreeSet<u128>) -> Option<u128>,
    ) -> anyhow::Result<()> {
        let Some(pill) = self.map.get_mut(&name) else {
            return Ok(())
        };
        let Some(uid) = f(&mut pill.versions) else {
            return Ok(())
        };
        std::fs::remove_dir(Self::uid_path(&name, uid))?;
        pill.versions.remove(&uid);
        Ok(())
    }

    pub fn remove_oldest(&mut self, name: String) -> anyhow::Result<()> {
        self.remove_from_version(name, |version| version.pop_first())
    }
    pub fn remove_latest(&mut self, name: String) -> anyhow::Result<()> {
        self.remove_from_version(name, |version| version.pop_last())
    }
}

impl Drop for Dreamer {
    fn drop(&mut self) {
        let content = serde_json::to_string(&self).expect("failed to serialize index.json");
        std::fs::create_dir_all(Self::path()).expect("failed to create dump dir");
        std::fs::write(Self::index_path(), content).expect("failed to write index.json");
        log::info!("dumped index.json at {:?}", SystemTime::now());
    }
}
