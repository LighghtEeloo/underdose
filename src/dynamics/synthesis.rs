use crate::store::{Atom, AtomMode, DripInner, Pill};
use crate::Machine;
use crate::{
    dynamics::{
        AtomArrow, AtomOb, AtomTask, PillOb, PillObInner, PillTask,
        PillTaskInner, RepoToSite, SiteToRepo, Synthesis,
    },
    utils::{self, IgnoreSet},
};
use std::path::Path;

impl Synthesis for PillOb {
    type Task = PillTask;

    fn synthesis(&self, machine: &Machine) -> anyhow::Result<Self::Task> {
        Ok(PillTask {
            name: self.name.to_owned(),
            root: self.root.synthesis(machine)?,
            inner: match &self.inner {
                PillObInner::GitModule { remote } => PillTaskInner::GitModule {
                    remote: remote.to_owned(),
                },
                PillObInner::Addicted { atoms } => PillTaskInner::Addicted {
                    atoms: atoms
                        .iter()
                        .map(|atom| atom.synthesis(machine))
                        .collect::<anyhow::Result<Vec<_>>>()?,
                },
            },
        })
    }
}

impl Synthesis for AtomOb {
    type Task = AtomTask;

    fn synthesis(&self, machine: &Machine) -> anyhow::Result<Self::Task> {
        Ok(AtomTask {
            src: self.src.0.to_owned(),
            dst: self.dst.0.to_owned(),
            mode: self.mode,
        })
    }
}
