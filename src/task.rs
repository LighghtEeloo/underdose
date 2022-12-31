use crate::drugstore::{Atom, AtomMode, DripInner, Pill};
use crate::utils;
use crate::Machine;
use colored::Colorize;
use git_url_parse::GitUrl;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Copy)]
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

pub struct PillTask {
    pub name: String,
    pub root: AtomTask,
    pub inner: PillTaskInner,
}

pub enum PillTaskInner {
    GitModule { remote: Box<GitUrl> },
    Addicted { atoms: Vec<AtomTask> },
}

impl Display for PillTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n[[{}]]", self.name)?;
        writeln!(f, "|| {}", self.root)?;
        match &self.inner {
            PillTaskInner::GitModule { remote } => {
                writeln!(f, "      || remote-task @ {}", remote)?;
            }
            PillTaskInner::Addicted { atoms } => {
                for atom in atoms {
                    writeln!(f, "      || {}", atom)?;
                }
            }
        }
        Ok(())
    }
}

pub struct AtomTask {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub mode: AtomMode,
}

impl Display for AtomTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = {
            let mode = format!("{}", self.mode);
            match self.mode {
                AtomMode::FileCopy => mode.bright_yellow(),
                AtomMode::Link => mode.blue(),
            }
        };
        let arrow = match self.mode {
            AtomMode::FileCopy => "==>".bright_yellow(),
            AtomMode::Link => "~~>".blue(),
        };
        write!(
            f,
            "{} :: [{}] {} [{}]",
            mode,
            self.src.display(),
            arrow,
            self.dst.display()
        )
    }
}

mod synthesis {
    use crate::utils::{IgnoreSet, IgnoreSetBuilder};

    use super::*;

    impl Synthesis for Pill {
        type Task = PillTask;

        fn synthesis(&self, machine: &Machine, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
            let ref name = self.name;
            let ref drip = self.drip;
            log::trace!("synthesizing drip <{}>", name);
            let root = drip
                .root
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no root set for drip <{}>", name))?;
            match &drip.inner {
                Some(DripInner::GitModule { remote }) => Ok(PillTask {
                    name: name.to_owned(),
                    root: root.synthesis(machine, arrow)?,
                    inner: PillTaskInner::GitModule {
                        remote: match remote.parse() {
                            Ok(url) => Box::new(url),
                            Err(e) => Err(anyhow::anyhow!("{:?}", e))?,
                        },
                    },
                }),
                Some(DripInner::Addicted { stem, ignore }) => Ok(PillTask {
                    name: name.to_owned(),
                    root: root.synthesis(machine, arrow)?,
                    inner: PillTaskInner::Addicted {
                        atoms: AddictedDrip {
                            root,
                            stem,
                            ignore_set: &machine.ignore.clone().chain(ignore.iter()).build(),
                            machine,
                        }
                        .resolve_atoms(arrow)?,
                    },
                }),
                None => Err(anyhow::anyhow!("no variant set")),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct AddictedDrip<'a> {
        root: &'a Atom,
        stem: &'a Vec<Atom>,
        ignore_set: &'a IgnoreSet,
        machine: &'a Machine,
    }

    impl<'a> AddictedDrip<'a> {
        fn atoms_copy(
            self,
            tasks: &mut Vec<AtomTask>,
            src: &Path,
            dst: &Path,
        ) -> anyhow::Result<()> {
            let src = utils::canonicalize_parent(src)?;
            let dst = utils::trim_path(dst)?;
            if self.ignore_set.is_ignored(&src) {
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
                    let src_path = entry.path();
                    let file_name = src_path.file_name().expect("file_name failed");
                    let dst_path = dst.join(file_name);
                    self.atoms_copy(tasks, &src_path, &dst_path)?;
                }
            } else {
                log::warn!("unsupported file detected: {}", src.display())
            }
            Ok(())
        }
        /// Resolve stem atoms to absolute file paths; requires a direction
        fn resolve_atoms(self, arrow: TaskArrow) -> anyhow::Result<Vec<AtomTask>> {
            let AddictedDrip { root, stem, .. } = self;
            let mut tasks = Vec::new();
            for atom in stem {
                if matches!(atom.mode, AtomMode::Link) {
                    // Note: symlinks always have repo -> site orientation
                    tasks.push(AtomTask {
                        src: utils::canonicalize_parent(root.repo.join(&atom.repo))?,
                        dst: utils::trim_path(root.site.join(&atom.site))?,
                        mode: AtomMode::Link,
                    })
                } else {
                    let (src, dst) = match arrow {
                        SiteToRepo => (&atom.site, &atom.repo),
                        RepoToSite => (&atom.repo, &atom.site),
                    };
                    self.atoms_copy(&mut tasks, &root.site.join(src), &root.repo.join(dst))?
                }
            }
            Ok(tasks)
        }
    }

    impl Synthesis for Atom {
        type Task = AtomTask;

        fn synthesis(&self, machine: &Machine, arrow: TaskArrow) -> anyhow::Result<Self::Task> {
            log::trace!("synthesizing atom <{:?}>", self);
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
    use std::io;

    use crate::utils::Prompt;

    use super::*;

    impl Exec for PillTask {
        fn exec(self) -> anyhow::Result<()> {
            match self.inner {
                PillTaskInner::GitModule { remote } => todo!(),
                PillTaskInner::Addicted { atoms } => {
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
            log::debug!("executing atom: {}", self);
            fs::create_dir_all(
                self.dst
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("no parent for destination"))?,
            )?;
            if self.dst.exists() {
                Prompt::new(&format!(
                    "target <{}> already exists.\noverwrite? [N/y/!] ",
                    self.dst.display()
                ))
                .process(|s| {
                    match s {
                        "y" => {
                            if self.dst.is_file() || self.dst.is_symlink() {
                                fs::remove_file(&self.dst)?;
                            } else if self.dst.is_dir() {
                                fs::remove_dir_all(&self.dst)?;
                            }
                        }
                        "!" => {
                            println!("abort!");
                            Err(io::Error::new(io::ErrorKind::Other, "abort!"))?;
                        }
                        _ => {
                            println!("skipping...");
                        }
                    };
                    Ok(())
                })?;
                fs::remove_file(&self.dst)?;
            }
            log::trace!("exec -- {}", self);
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
