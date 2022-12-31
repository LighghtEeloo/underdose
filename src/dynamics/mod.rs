pub mod task;

use crate::Machine;

pub use task::PillTask;

#[derive(Clone, Copy)]
pub enum TaskArrow {
    SiteToRepo,
    RepoToSite,
}
pub use TaskArrow::*;

pub trait Synthesis {
    type Task;
    fn synthesis(
        &self, machine: &Machine, arrow: TaskArrow,
    ) -> anyhow::Result<Self::Task>;
}

pub trait Execution {
    fn execution(self) -> anyhow::Result<()>;
}
