use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::{
    fmt::{self, Debug},
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
    pub buffer: String,
    pub path: PathBuf,
}

impl Conf {
    pub fn ensure_exist(&self) -> anyhow::Result<&Self> {
        if !self.path.exists() {
            std::fs::create_dir_all(self.path.parent().expect("config path should have parent"))?;
            std::fs::write(&self.path, &self.buffer)?;
        }
        Ok(self)
    }
    pub fn ensure_forced(&self) -> anyhow::Result<&Self> {
        if !self.path.exists() {
            std::fs::create_dir_all(self.path.parent().expect("config path should have parent"))?;
        }
        std::fs::write(&self.path, &self.buffer)?;
        Ok(self)
    }
    pub fn read(&self) -> anyhow::Result<String> {
        let mut buf = String::new();
        let mut file = std::fs::File::open(&self.path)?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    pub fn edit(&self) -> anyhow::Result<()> {
        let editor =
            std::env::var("EDITOR").map_err(|_| anyhow::anyhow!("$EDITOR envvar not set"))?;
        let status = std::process::Command::new(editor)
            .arg(&self.path)
            .status()
            .map_err(|_| anyhow::anyhow!("failed to execute process"))?;
        if !status.success() {
            Err(anyhow::anyhow!("failed to edit config file"))?;
        }
        Ok(())
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
        template["repo"]["name"] = toml_edit::value(name);
        Self { template }
    }
    /// convert to Conf whose buffer is well formatted
    pub fn conf(self, path: PathBuf) -> Conf {
        Conf {
            buffer: self.template.to_string(),
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

#[derive(Clone)]
pub struct IgnoreSetBuilder {
    globs: GlobSetBuilder,
}
impl Default for IgnoreSetBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl Debug for IgnoreSetBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IgnoreSetBuilder(..)")
    }
}

impl IgnoreSetBuilder {
    pub fn new() -> Self {
        let globs = GlobSetBuilder::new();
        Self { globs }
    }
    pub fn chain(mut self, ignore: impl Iterator<Item = impl AsRef<str>>) -> Self {
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
