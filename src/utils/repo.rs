use git2::{Delta, DiffFile, Statuses};
use std::{fmt::Display, path::Path};

pub struct Dirt<'a> {
    old: &'a Path,
    new: &'a Path,
    delta: Delta,
}

impl<'a> Display for Dirt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ===[{:?}]==> {}",
            self.old.display(),
            self.delta,
            self.new.display(),
        )
    }
}

impl<'a> Dirt<'a> {
    pub fn of_repo_status(statuses: &'a Statuses) -> anyhow::Result<Vec<Dirt<'a>>> {
        let mut dirts = Vec::new();
        let file_to_path =
            |file: DiffFile<'a>| file.path().ok_or_else(|| anyhow::anyhow!("file path err"));
        for status in statuses.iter() {
            match status.index_to_workdir() {
                | None => (),
                | Some(status) => {
                    let delta = status.status();
                    match delta {
                        | Delta::Unmodified | Delta::Ignored => (),
                        | Delta::Added
                        | Delta::Deleted
                        | Delta::Modified
                        | Delta::Renamed
                        | Delta::Copied
                        | Delta::Untracked
                        | Delta::Typechange
                        | Delta::Unreadable
                        | Delta::Conflicted => dirts.push(Dirt {
                            old: file_to_path(status.old_file())?,
                            new: file_to_path(status.new_file())?,
                            delta,
                        }),
                    }
                }
            }
        }
        Ok(dirts)
    }
}
