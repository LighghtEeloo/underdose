use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub struct UnderdoseStatics {
    pub conf: PathBuf,
    pub dreams: PathBuf,
}

pub static UNDERDOSE_PATH: Lazy<UnderdoseStatics> = Lazy::new(|| {
    let dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
        .expect("No valid config directory fomulated");
    UnderdoseStatics {
        conf: dirs.config_dir().join("Underdose.toml"),
        dreams: dirs.data_dir().join("dreams"),
    }
});
