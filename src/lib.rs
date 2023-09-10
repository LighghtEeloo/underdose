pub mod cli {
    pub mod app;
    pub mod interface;
    pub mod proc;

    pub use interface::Cli;
}
pub mod machine;
pub mod store;
pub mod dump {
    pub mod man;
    pub mod item;
}
pub mod dynamics;
pub mod utils {
    pub mod conf;
    pub mod repo;
    pub mod path;

    pub mod global;
    pub use global::UNDERDOSE_STATICS;
}

pub use machine::Machine;
pub use store::Drugstore;
