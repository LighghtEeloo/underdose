mod execution;
mod observation;
mod probing;
mod synthesis;
mod task;

use crate::Machine;

pub use observation::{AtomOb, PillOb, PillObInner};
pub use task::{AtomTask, PillTask, PillTaskInner};

#[derive(Clone, Copy)]
pub enum AtomArrow {
    SiteToRepo,
    RepoToSite,
}
pub use AtomArrow::*;

pub trait Probing {
    type Observation;
    fn probing(&self, machine: &Machine, arrow: AtomArrow) -> anyhow::Result<Self::Observation>;
}

pub trait Synthesis {
    type Task;
    fn synthesis(&self, machine: &Machine) -> anyhow::Result<Self::Task>;
}

pub trait Execution {
    fn execution(self) -> anyhow::Result<()>;
}
