pub mod cli {
    pub mod app;
    pub mod interface;
}
pub use cli::interface::Cli;

mod machine;
mod store;
pub use machine::Machine;
pub use store::Drugstore;

mod executor;
mod observor;
mod dreamer;
pub use dreamer::Dreamer;
pub use executor::Executor;

mod drip;
pub use drip::{Arrow, ArrowSrc, Drip};

pub mod utils {
    pub mod conf;
    pub mod repo;
    pub mod path;

    pub mod global;
    #[cfg(test)]
    pub mod tests;
}
