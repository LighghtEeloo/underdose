use crate::utils::UNDERDOSE_STATICS;
use std::path::Path;

pub struct DumpManager {}

impl DumpManager {
    pub fn new() -> Self {
        DumpManager {}
    }
    pub fn path() -> &'static Path {
        &UNDERDOSE_STATICS.dump.as_path()
    }
}
