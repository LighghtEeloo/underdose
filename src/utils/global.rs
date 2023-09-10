use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub struct UnderdoseStatics {
    pub conf: PathBuf,
    pub dump: PathBuf,
    pub dreams: PathBuf,
}

pub static UNDERDOSE_STATICS: Lazy<UnderdoseStatics> = Lazy::new(|| {
    let dirs = ProjectDirs::from("", "LitiaEeloo", "Underdose")
        .expect("No valid config directory fomulated");
    UnderdoseStatics {
        conf: dirs.config_dir().join("Underdose.toml"),
        dump: dirs.cache_dir().join("dump"),
        dreams: dirs.data_dir().join("dreams"),
    }
});
