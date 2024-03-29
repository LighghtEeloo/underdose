use crate::{utils::global::UNDERDOSE_PATH, Arrow, Drip};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Dreamer {
    pub map: HashMap<String, DreamDrip>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DreamDrip {
    pub name: String,
    pub site: PathBuf,
    pub versions: Vec<Uuid>,
}

impl Dreamer {
    pub fn path() -> &'static Path {
        &UNDERDOSE_PATH.dreams.as_path()
    }
    fn index_path() -> PathBuf {
        Self::path().join("index.json")
    }
    fn uid_path(name: impl AsRef<str>, uid: Uuid) -> PathBuf {
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

    pub fn dump(&mut self, name: String, drip: &Drip) -> anyhow::Result<()> {
        let uid = Uuid::now_v1(&[0, 0, 0, 0, 0, 0]);
        let path = Self::uid_path(&name, uid);
        std::fs::create_dir_all(&path)?;
        let mut non_empty = false;
        for Arrow { rel_site: stem, .. } in drip.arrows.iter() {
            let site = drip.site.join(stem);
            log::info!("mv {} {}", site.display(), path.join(stem).display());
            crate::utils::path::create_dir_parent(&site)?;
            let site = crate::utils::path::canonicalize(site)?;
            crate::utils::path::create_dir_parent(&path.join(stem))?;
            let dump = crate::utils::path::canonicalize(path.join(stem))?;
            if site.is_symlink() {
                std::fs::remove_file(&site).map_err(|e| {
                    anyhow::anyhow!("failed to remove symlink {}: {}", site.display(), e)
                })?;
            } else if site.exists() {
                std::fs::rename(&site, &dump).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to move {} to {}: {}",
                        site.display(),
                        dump.display(),
                        e
                    )
                })?;
                non_empty = true;
            }
        }
        if non_empty {
            self.map
                .entry(name.clone())
                .or_insert(DreamDrip {
                    name,
                    site: drip.site.clone(),
                    versions: Vec::new(),
                })
                .versions
                .push(uid);
        }
        Ok(())
    }

    pub fn write_index(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string(&self)
            .map_err(|e| anyhow::anyhow!("failed to serialize index.json: {}", e))?;
        std::fs::create_dir_all(Self::path())
            .map_err(|e| anyhow::anyhow!("failed to create dump dir: {}", e))?;
        std::fs::write(Self::index_path(), content)
            .map_err(|e| anyhow::anyhow!("failed to write index.json: {}", e))?;
        log::trace!("dumped index.json at {}", Self::index_path().display());
        Ok(())
    }
}

impl Drop for Dreamer {
    fn drop(&mut self) {
        self.write_index().expect("fail on dropping dreamer");
    }
}
