use std::path::PathBuf;

pub struct Drip {
    /// where the root of site is, globally
    pub site: PathBuf,
    /// where the root of drip is, relative to repo root
    pub rel_repo: PathBuf,
    /// tasks to complete
    pub arrows: Vec<Arrow>,
}

pub struct Arrow {
    pub site: PathBuf,
    pub src: ArrowSrc,
}

pub enum ArrowSrc {
    Git(String),
    Link { rel: PathBuf },
    Collector,
}
