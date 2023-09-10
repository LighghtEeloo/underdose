use std::path::{Path, PathBuf};

pub fn ensured_dir<P: AsRef<Path>>(dir_path: P) -> anyhow::Result<PathBuf> {
    let dir_path = dir_path.as_ref().to_path_buf();
    if !dir_path.exists() {
        std::fs::create_dir_all(&dir_path)?;
    }
    Ok(dir_path)
}

pub fn expand_home<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(shellexpand::path::tilde(path.as_ref())))
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let path = expand_home(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path <{}> should have parent", path.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("path <{}> should have file name", path.display()))?;
    let parent = parent.canonicalize()?;
    Ok(parent.join(file_name))
}

pub fn trim<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let par = path
        .as_ref()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path <{}> should have parent", path.as_ref().display()))?;
    let file_name = path.as_ref().file_name().ok_or_else(|| {
        anyhow::anyhow!("path <{}> should have file name", path.as_ref().display())
    })?;
    let file_name = file_name
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("file name should be valid utf-8"))?;
    let file_name = file_name.trim_end_matches('/');
    let res = par.join(file_name);
    Ok(res)
}
