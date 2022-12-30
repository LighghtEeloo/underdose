use crate::{Atom, AtomMode, Drip, DripVariant};
use git_url_parse::GitUrl;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use toml_edit::Document;

pub enum TaskArrow {
    SiteToRepo,
    RepoToSite,
}
pub use TaskArrow::*;

pub trait Synthesis {
    type Task;
    fn syn(&self, arrow: TaskArrow) -> anyhow::Result<Self::Task>;
}

pub trait Task {
    fn exec(self) -> anyhow::Result<()>;
}

pub enum DripTask {
    GitModule { remote: Box<GitUrl>, root: AtomTask },
    UnderManage { atoms: Vec<AtomTask> },
}

pub struct AtomTask {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub mode: AtomMode,
}

impl Task for DripTask {
    fn exec(self) -> anyhow::Result<()> {
        match self {
            DripTask::GitModule { remote, root } => todo!(),
            DripTask::UnderManage { atoms } => {
                for atom in atoms {
                    atom.exec()?;
                }
            }
        }
        Ok(())
    }
}

impl Synthesis for Drip {
    type Task = DripTask;

    fn syn(&self, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
        match &self.var {
            DripVariant::GitModule { remote } => Ok(DripTask::GitModule {
                remote: match remote.parse() {
                    Ok(url) => Box::new(url),
                    Err(e) => Err(anyhow::anyhow!("{:?}", e))?,
                },
                root: self.root.syn(arrow)?,
            }),
            DripVariant::UnderManage { stem } => Ok(DripTask::UnderManage {
                atoms: Self::resolve_atoms(&self.root, stem, arrow),
            }),
        }
    }
}

impl Drip {
    pub fn check_env(&self, machine_env: &HashSet<String>) -> bool {
        true
    }
    /// Resolve stem atoms to absolute file paths; requires a direction
    pub fn resolve_atoms(root: &Atom, stem: &Vec<Atom>, arrow: TaskArrow) -> Vec<AtomTask> {
        fn atoms_copy(tasks: &mut Vec<AtomTask>, src: &Path, dst: &Path) {
            if src.is_file() {
                tasks.push(AtomTask {
                    src: src.to_owned(),
                    dst: dst.to_owned(),
                    mode: AtomMode::FileCopy,
                })
            } else if src.is_dir() {
                for entry in src.read_dir().expect("read_dir failed") {
                    let entry = entry.expect("entry failed");
                    let path = entry.path();
                    let file_name = path.file_name().expect("file_name failed");
                    let dst_path = dst.join(file_name);
                    atoms_copy(tasks, &path, &dst_path)
                }
            } else {
                // panic!("unsupported file detected")
            }
        }
        let mut tasks = Vec::new();
        for atom in stem {
            if matches!(atom.mode, AtomMode::Link) {
                // Note: symlinks always have repo -> site orientation
                tasks.push(AtomTask {
                    src: root.repo.join(&atom.repo),
                    dst: root.site.join(&atom.site),
                    mode: AtomMode::Link,
                })
            } else {
                let (src, dst) = match arrow {
                    SiteToRepo => (&atom.site, &atom.repo),
                    RepoToSite => (&atom.repo, &atom.site),
                };
                atoms_copy(&mut tasks, &root.site.join(src), &root.repo.join(dst))
            }
        }
        tasks
    }
}

impl Synthesis for Atom {
    type Task = AtomTask;

    fn syn(&self, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
        Ok(match arrow {
            SiteToRepo => AtomTask {
                src: self.site.to_owned(),
                dst: self.repo.to_owned(),
                mode: self.mode,
            },
            RepoToSite => AtomTask {
                src: self.repo.to_owned(),
                dst: self.site.to_owned(),
                mode: self.mode,
            },
        })
    }
}

impl Task for AtomTask {
    fn exec(self) -> anyhow::Result<()> {
        fs::create_dir_all(
            self.dst
                .parent()
                .ok_or_else(|| anyhow::anyhow!("no parent for destination"))?,
        )?;
        match self.mode {
            AtomMode::FileCopy => {
                fs::copy(&self.src, &self.dst)?;
            }
            AtomMode::Link => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&self.src, &self.dst)?;
                }
                #[cfg(windows)]
                {
                    if self.src.is_file() {
                        std::os::windows::fs::symlink_file(&self.src, &self.dst)?;
                    } else if self.src.is_dir() {
                        std::os::windows::fs::symlink_dir(&self.src, &self.dst)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Display for AtomTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}> {} {} {}",
            self.mode,
            self.src.display(),
            self.mode.display_arrow(),
            self.dst.display()
        )
    }
}
