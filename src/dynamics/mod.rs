mod execution;
pub mod observation;
mod probing;
mod synthesis;
pub mod task;

use crate::Machine;

pub use task::{AtomTask, PillTask, PillTaskInner};

#[derive(Clone, Copy)]
pub enum AtomArrow {
    SiteToRepo,
    RepoToSite,
}
pub use AtomArrow::*;

pub trait Probing {
    type Observation;
    fn probing(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Observation>;
}

pub trait Synthesis {
    type Task;
    fn synthesis(
        &self, machine: &Machine, arrow: AtomArrow,
    ) -> anyhow::Result<Self::Task>;
}

pub trait Execution {
    fn execution(self) -> anyhow::Result<()>;
}
