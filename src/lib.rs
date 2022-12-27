use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub struct DrugStore {
    env: HashMap<String, HashSet<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Pill {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Drip {
    root: PathBuf,
    env: Vec<String>,
    stem: Vec<Atom>,
}

#[derive(Serialize, Deserialize)]
pub struct Atom {
    site: PathBuf,
    repo: PathBuf,
}
