use std::{
    io::{self, Read, Write},
    path::PathBuf,
};
use toml_edit::DocumentMut;

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
    pub template: DocumentMut,
}

impl UnderdoseConf {
    pub fn new(name: String, repo: PathBuf) -> Self {
        let toml = UNDERDOSE_TOML;
        let mut template = toml.parse::<DocumentMut>().expect("invalid doc");
        template["repo"]["name"] = toml_edit::value(name);
        template["repo"]["local"] = toml_edit::value(repo.to_string_lossy().to_string());
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
