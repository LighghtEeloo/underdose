use crate::store::{Atom, AtomMode, DripInner, Pill};
use crate::Machine;
use crate::{
    dynamics::{
        AtomArrow, AtomTask, PillTask, PillTaskInner, RepoToSite, SiteToRepo,
        Synthesis,
    },
    utils::{self, IgnoreSet},
};
use std::path::Path;

impl Synthesis for Pill {
    type Task = PillTask;

    fn synthesis(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Task> {
        let name = &self.name;
        let drip = &self.drip;
        log::trace!("synthesizing drip <{}>", name);
        let root = drip.root.as_ref().ok_or_else(|| {
            anyhow::anyhow!("no root set for drip <{}>", name)
        })?;
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
                        ignore_set: &machine
                            .ignore
                            .clone()
                            .chain(ignore.iter())
                            .build(),
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
    /// Resolve stem atoms to absolute file paths; requires a direction
    fn resolve_atoms(self, arrow: AtomArrow) -> anyhow::Result<Vec<AtomTask>> {
        let AddictedDrip { root, stem, .. } = self;
        log::trace!(
            "\nresolving site: [{}]\n       && repo: [{}]",
            self.root.site.display(),
            self.root.repo.display()
        );
        let mut tasks = Vec::new();
        for atom in stem {
            if matches!(atom.mode, AtomMode::Link) {
                // Note: symlinks always have repo -> site orientation
                tasks.push(AtomTask {
                    src: utils::canonicalize_path(root.repo.join(&atom.repo))?,
                    dst: utils::trim_path(root.site.join(&atom.site))?,
                    mode: AtomMode::Link,
                })
            } else {
                let (src, dst) = match arrow {
                    SiteToRepo => (&atom.site, &atom.repo),
                    RepoToSite => (&atom.repo, &atom.site),
                };
                self.atoms_copy(
                    &mut tasks,
                    &root.site.join(src),
                    &root.repo.join(dst),
                )?
            }
        }
        Ok(tasks)
    }
    fn atoms_copy(
        self, tasks: &mut Vec<AtomTask>, src: &Path, dst: &Path,
    ) -> anyhow::Result<()> {
        let src = utils::canonicalize_path(src)?;
        let dst = utils::trim_path(dst)?;
        if self.ignore_set.is_ignored(&src) {
            log::debug!("ignoring {}", src.display())
        } else if src.is_file() {
            tasks.push(AtomTask {
                src,
                dst,
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
}

impl Synthesis for Atom {
    type Task = AtomTask;

    fn synthesis(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Task> {
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
