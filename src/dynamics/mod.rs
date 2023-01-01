mod execution;
mod probing;
mod synthesis;
pub mod task;

use crate::Machine;

pub use task::{AtomTask, PillTask, PillTaskInner};

#[derive(Clone, Copy)]
pub enum TaskArrow {
    SiteToRepo,
    RepoToSite,
}
pub use TaskArrow::*;

pub trait Probing {
    type Observation;
    fn probing(&self) -> Self::Observation;
}

pub trait Synthesis {
    type Task;
    fn synthesis(
        &self, machine: &Machine, arrow: TaskArrow,
    ) -> anyhow::Result<Self::Task>;
}

pub trait Execution {
    fn execution(self) -> anyhow::Result<()>;
}
