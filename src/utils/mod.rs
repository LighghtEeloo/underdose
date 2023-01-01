pub mod repo;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use toml_edit::Document;

pub const UNDERDOSE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/Underdose.toml"
));
pub const DRUGSTORE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/Drugstore.toml"
));

#[derive(Debug)]
pub struct Conf {
    pub template: String,
    pub path: PathBuf,
}

impl Conf {
    pub fn ensure_exist(&self) -> anyhow::Result<&Self> {
        // self.path = canonicalize_parent(&self.path)?;
        if !self.path.exists() {
            std::fs::create_dir_all(
                self.path.parent().expect("config path should have parent"),
            )?;
            std::fs::write(&self.path, &self.template)?;
        }
        Ok(self)
    }
    pub fn ensure_force(&self) -> anyhow::Result<&Self> {
        // self.path = canonicalize_parent(&self.path)?;
        if !self.path.exists() {
            std::fs::create_dir_all(
                self.path.parent().expect("config path should have parent"),
            )?;
        }
        std::fs::write(&self.path, &self.template)?;
        Ok(self)
    }
    pub fn read(&self) -> anyhow::Result<String> {
        let mut buf = String::new();
        let mut file = std::fs::File::open(&self.path)?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Debug)]
pub struct UnderdoseConf {
    pub template: toml_edit::Document,
}

impl UnderdoseConf {
    pub fn new(name: String) -> Self {
        let toml = UNDERDOSE_TOML;
        let mut template = toml.parse::<Document>().expect("invalid doc");
        template["name"] = toml_edit::value(name);
        Self { template }
    }
    pub fn conf(self, path: PathBuf) -> Conf {
        Conf {
            template: self.template.to_string(),
            path,
        }
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

pub fn ensured_dir<P: AsRef<Path>>(dir_path: P) -> anyhow::Result<PathBuf> {
    let path = canonicalize_parent(dir_path)?;
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn expand_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(shellexpand::path::tilde(path.as_ref())))
}

pub fn canonicalize_parent<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let path = expand_path(path)?;
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!("path <{}> should have parent", path.display())
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        anyhow::anyhow!("path <{}> should have file name", path.display())
    })?;
    let parent = parent.canonicalize()?;
    Ok(parent.join(file_name))
}

pub fn trim_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let par = path.as_ref().parent().ok_or_else(|| {
        anyhow::anyhow!("path <{}> should have parent", path.as_ref().display())
    })?;
    let file_name = path.as_ref().file_name().ok_or_else(|| {
        anyhow::anyhow!(
            "path <{}> should have file name",
            path.as_ref().display()
        )
    })?;
    let file_name = file_name
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("file name should be valid utf-8"))?;
    let file_name = file_name.trim_end_matches('/');
    let res = par.join(file_name);
    Ok(res)
}

#[derive(Debug, Clone)]
pub struct IgnoreSetBuilder {
    globs: GlobSetBuilder,
}

impl IgnoreSetBuilder {
    pub fn new() -> Self {
        let globs = GlobSetBuilder::new();
        Self { globs }
    }
    pub fn chain(
        mut self, ignore: impl Iterator<Item = impl AsRef<str>>,
    ) -> Self {
        for p in ignore {
            let p = p.as_ref();
            self.globs.add(
                GlobBuilder::new(p.strip_suffix('/').unwrap_or(p))
                    .build()
                    .expect("invalid glob pattern"),
            );
        }
        self
    }
    pub fn build(self) -> IgnoreSet {
        IgnoreSet {
            globs: self.globs.build().expect("invalid globset pattern"),
        }
    }
}

#[derive(Clone)]
pub struct IgnoreSet {
    globs: GlobSet,
}

impl IgnoreSet {
    pub fn is_ignored(&self, path: impl AsRef<Path>) -> bool {
        self.globs.is_match(path.as_ref())
    }
}

pub struct Prompt<'a> {
    line: &'a str,
}

impl<'a> Prompt<'a> {
    pub fn new(line: &'a str) -> Self {
        Self { line }
    }

    pub fn process(
        self, cont_lower_trim: impl FnOnce(&str) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut response = String::new();
        print!("{}", self.line);
        io::stdout().flush()?;
        {
            let stdin = io::stdin();
            stdin.read_line(&mut response)?;
        }

        cont_lower_trim(response.to_lowercase().trim())
    }
}
