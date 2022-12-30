use crate::drugstore::{Atom, AtomMode, Drip, DripVariant};
use crate::Machine;
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

pub enum TaskArrow {
    SiteToRepo,
    RepoToSite,
}
pub use TaskArrow::*;

pub trait Synthesis {
    type Task;
    fn synthesis(&self, machine: &Machine, arrow: TaskArrow) -> anyhow::Result<Self::Task>;
}

pub trait Exec {
    fn exec(self) -> anyhow::Result<()>;
}

pub enum DripTask {
    GitModule { remote: Box<GitUrl>, root: AtomTask },
    Addicted { atoms: Vec<AtomTask> },
}

pub struct AtomTask {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub mode: AtomMode,
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

mod synthesis {
    use super::*;

    impl Synthesis for Drip {
        type Task = DripTask;

        fn synthesis(&self, machine: &Machine, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
            let root = self
                .root
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no root set"))?;
            match &self.var {
                Some(DripVariant::GitModule { remote }) => Ok(DripTask::GitModule {
                    remote: match remote.parse() {
                        Ok(url) => Box::new(url),
                        Err(e) => Err(anyhow::anyhow!("{:?}", e))?,
                    },
                    root: root.synthesis(machine, arrow)?,
                }),
                Some(DripVariant::Addicted { stems }) => Ok(DripTask::Addicted {
                    atoms: AddictedDrip {
                        root,
                        stems,
                        machine,
                    }
                    .resolve_atoms(arrow),
                }),
                None => Err(anyhow::anyhow!("no variant set")),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct AddictedDrip<'a> {
        root: &'a Atom,
        stems: &'a Vec<Atom>,
        machine: &'a Machine,
    }

    impl<'a> AddictedDrip<'a> {
        fn atoms_copy(self, tasks: &mut Vec<AtomTask>, src: &Path, dst: &Path) {
            if self.machine.ignore.is_ignored(src) {
                log::debug!("ignoring {}", src.display())
            } else if src.is_file() {
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
                    self.atoms_copy(tasks, &path, &dst_path)
                }
            } else {
                log::warn!("unsupported file detected: {}", src.display())
            }
        }
        /// Resolve stem atoms to absolute file paths; requires a direction
        fn resolve_atoms(self, arrow: TaskArrow) -> Vec<AtomTask> {
            let AddictedDrip {
                root,
                stems,
                machine,
            } = self;
            let mut tasks = Vec::new();
            for atom in stems {
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
                    self.atoms_copy(&mut tasks, &root.site.join(src), &root.repo.join(dst))
                }
            }
            tasks
        }
    }

    impl Synthesis for Atom {
        type Task = AtomTask;

        fn synthesis(&self, machine: &Machine, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
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
}

mod exec {
    use super::*;

    impl Exec for DripTask {
        fn exec(self) -> anyhow::Result<()> {
            match self {
                DripTask::GitModule { remote, root } => todo!(),
                DripTask::Addicted { atoms } => {
                    for atom in atoms {
                        atom.exec()?;
                    }
                }
            }
            Ok(())
        }
    }

    impl Exec for AtomTask {
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
}
