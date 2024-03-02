use once_cell::sync::Lazy;
use sculptor::{AppAuthor, ProjectInfo};
use std::path::PathBuf;

pub struct UnderdoseStatics {
    pub conf: PathBuf,
    pub dreams: PathBuf,
}

pub struct ProjectDirs;

impl AppAuthor for ProjectDirs {
    fn author() -> &'static str {
        "LitiaEeloo"
    }
    fn app_name() -> &'static str {
        "Underdose"
    }
}

pub static UNDERDOSE_PATH: Lazy<UnderdoseStatics> = Lazy::new(|| UnderdoseStatics {
    conf: ProjectDirs::config_dir().join("Underdose.toml"),
    dreams: ProjectDirs::data_dir().join("dreams"),
});
