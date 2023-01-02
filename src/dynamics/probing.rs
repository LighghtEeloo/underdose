use crate::dynamics::{AtomArrow, Probing, PillOb, PillObInner, AtomOb};
use crate::store::{Atom, AtomMode, DripInner, Pill};
use crate::utils::{self, IgnoreSet};
use crate::{machine, Machine};
use git_url_parse::GitUrl;
use std::path::{Path, PathBuf};

impl Probing for Pill {
    type Observation = PillOb;

    fn probing(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Observation> {
        log::trace!("probing pill <{}>", self.name);
        let root = self.drip.root.as_ref().ok_or_else(|| {
            anyhow::anyhow!("no root set for pill <{}>", self.name)
        })?;
        Ok(PillOb {
            name: self.name.to_owned(),
            root: root.probing(machine, arrow)?,
            inner: match &self.drip.inner {
                Some(DripInner::GitModule { remote }) => {
                    PillObInner::GitModule {
                        remote: match remote.parse() {
                            Ok(url) => Box::new(url),
                            Err(e) => Err(anyhow::anyhow!("{:?}", e))?,
                        },
                    }
                }
                Some(DripInner::Addicted { stem, ignore }) => {
                    PillObInner::Addicted {
                        atoms: AddictedDrip {
                            root,
                            ignore_set: &machine
                                .ignore
                                .clone()
                                .chain(ignore.iter())
                                .build(),
                            machine,
                            arrow,
                        }
                        .resolve_atoms(stem)?,
                    }
                }
                None => Err(anyhow::anyhow!("no variant set"))?,
            },
        })
    }
}

#[derive(Clone, Copy)]
struct AddictedDrip<'a> {
    root: &'a Atom,
    ignore_set: &'a IgnoreSet,
    machine: &'a Machine,
    arrow: AtomArrow,
}

impl<'a> AddictedDrip<'a> {
    /// Resolve stem atoms to absolute file paths; requires a direction
    fn resolve_atoms(self, stem: &Vec<Atom>) -> anyhow::Result<Vec<AtomOb>> {
        log::trace!(
            "\nresolving site: [{}]\n       && repo: [{}]",
            self.root.site.display(),
            self.root.repo.display()
        );
        let mut atoms = Vec::new();
        for atom in stem {
            match self
                .atom_append(atom)
                .probing(self.machine, self.arrow)?
                .mode
            {
                AtomMode::Link => {
                    // Note: symlinks always have repo -> site orientation
                    atoms.push(
                        self.atom_append(atom)
                            .probing(self.machine, self.arrow)?,
                    )
                }
                AtomMode::FileCopy => {
                    let atom = &self
                        .atom_append(atom)
                        .probing(self.machine, self.arrow)?;
                    self.atoms_copy(&mut atoms, &atom.src, &atom.dst)?
                }
            }
        }
        Ok(atoms)
    }
    fn atoms_copy(
        self, atoms: &mut Vec<AtomOb>, src: &(PathBuf, bool),
        dst: &(PathBuf, bool),
    ) -> anyhow::Result<()> {
        let src_p = &src.0;
        if self.ignore_set.is_ignored(src_p) {
            log::debug!("ignoring {}", src_p.display())
        } else if src_p.is_file() {
            atoms.push(AtomOb {
                src: src.to_owned(),
                dst: dst.to_owned(),
                mode: AtomMode::FileCopy,
                arrow: self.arrow,
            })
        } else if src_p.is_dir() {
            for entry in src_p.read_dir().expect("read_dir failed") {
                let entry = entry.expect("entry failed");
                let src_path = entry.path();
                let dst_path = dst.0.join(entry.file_name());

                let peek = |p: PathBuf| -> anyhow::Result<_> {
                    let exists = p.exists();
                    let p = if exists {
                        utils::canonicalize_path(p)?
                    } else {
                        utils::trim_path(p)?
                    };
                    Ok((p, exists))
                };

                self.atoms_copy(atoms, &peek(src_path)?, &peek(dst_path)?)?;
            }
        } else {
            log::warn!("unsupported file detected: {}", src_p.display())
        }
        Ok(())
    }
    fn atom_append(&self, atom: &Atom) -> Atom {
        self.root.append(atom)
    }
}

impl Atom {
    fn append(&self, atom: &Atom) -> Atom {
        Atom {
            site: self.site.join(&atom.site),
            repo: self.repo.join(&atom.repo),
            mode: atom.mode,
        }
    }
}

impl Probing for Atom {
    type Observation = AtomOb;

    fn probing(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Observation> {
        log::trace!("probing atom <{:?}>", self);
        let (src, dst) = match (self.mode, arrow) {
            (AtomMode::Link, _) => (self.repo.to_owned(), self.site.to_owned()),
            (_, AtomArrow::SiteToRepo) => {
                (self.site.to_owned(), self.repo.to_owned())
            }
            (_, AtomArrow::RepoToSite) => {
                (self.repo.to_owned(), self.site.to_owned())
            }
        };
        let peek = |p: PathBuf| -> anyhow::Result<_> {
            let exists = p.exists();
            let p = if exists {
                utils::canonicalize_path(p)?
            } else {
                utils::trim_path(p)?
            };
            Ok((p, exists))
        };
        Ok(AtomOb {
            src: peek(src)?,
            dst: peek(dst)?,
            mode: self.mode,
            arrow,
        })
    }
}
