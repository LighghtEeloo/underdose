use std::{io::Read, path::PathBuf};

pub struct Conf {
    pub name: String,
    pub template: &'static str,
    pub path: PathBuf,
}

impl Conf {
    pub fn ensure(self) -> anyhow::Result<Self> {
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
