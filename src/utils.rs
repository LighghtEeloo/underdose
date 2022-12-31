use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Conf {
    pub name: String,
    pub template: &'static str,
    pub path: PathBuf,
}

impl Conf {
    pub fn ensure(mut self) -> anyhow::Result<Self> {
        self.path = canonicalize_path(&self.path)?;
        if !self.path.exists() {
            std::fs::create_dir_all(self.path.parent().expect("config path should have parent"))?;
            std::fs::write(&self.path, self.template)?;
        }
        Ok(self)
    }
    pub fn read(self) -> anyhow::Result<String> {
        let mut buf = String::new();
        let mut file = std::fs::File::open(&self.path)?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

#[must_use]
pub fn passed_tutorial(toml: &toml::Value) -> anyhow::Result<()> {
    if let Some(tutorial) = toml.get("tutorial") {
        if let Some(tutorial) = tutorial.as_table() {
            if !tutorial.is_empty() {
                Err(anyhow::anyhow!("tutorial has not been completed yet"))?;
            }
        }
    }
    Ok(())
}

pub fn expand_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(shellexpand::path::tilde(path.as_ref())))
}

pub fn canonicalize_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let mut path = expand_path(path)?;
    path = path.canonicalize()?;
    Ok(path)
}

pub fn trim_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let par = path
        .as_ref()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path should have parent"))?;
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("path should have file name"))?;
    let file_name = file_name
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("file name should be valid utf-8"))?;
    let file_name = file_name.trim_end_matches('/');
    Ok(par.join(file_name))
}

#[derive(Debug, Clone)]
pub struct IgnoreSet {
    globs: GlobSet,
}

impl IgnoreSet {
    pub fn new(ignore: impl Iterator<Item = impl AsRef<str>>) -> Self {
        let mut globs = GlobSetBuilder::new();
        for p in ignore {
            let p = p.as_ref();
            globs.add(
                GlobBuilder::new(p.strip_suffix('/').unwrap_or(p))
                    .build()
                    .expect("invalid glob pattern"),
            );
        }
        Self {
            globs: globs.build().expect("invalid glob pattern set"),
        }
    }
    pub fn is_ignored(&self, path: impl AsRef<Path>) -> bool {
        self.globs.is_match(path.as_ref())
    }
}
