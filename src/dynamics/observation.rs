use crate::dynamics::{AtomArrow, Probing};
use crate::store::{Atom, AtomMode, DripInner, Pill};
use crate::utils::{self, conf::IgnoreSet};
use crate::{machine, Machine};
use colored::Colorize;
use git_url_parse::GitUrl;
use std::fmt::Display;
use std::path::{Path, PathBuf};

pub struct PillOb {
    pub name: String,
    pub root: AtomOb,
    pub inner: PillObInner,
}

pub enum PillObInner {
    GitModule { remote: Box<GitUrl> },
    Addicted { atoms: Vec<AtomOb> },
}

impl Display for PillOb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n[[{}]]", self.name)?;
        writeln!(f, "|| {}", self.root)?;
        match &self.inner {
            PillObInner::GitModule { remote } => {
                writeln!(f, "      || remote-task @ {}", remote)?;
            }
            PillObInner::Addicted { atoms } => {
                for atom in atoms {
                    writeln!(f, "      || {}", atom)?;
                }
            }
        }
        Ok(())
    }
}
pub struct AtomOb {
    pub src: (PathBuf, bool),
    pub dst: (PathBuf, bool),
    pub arrow: AtomArrow,
    pub mode: AtomMode,
}

impl Display for AtomOb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = {
            let mode = format!("{}", self.mode);
            match self.mode {
                AtomMode::FileCopy => mode.bright_yellow(),
                AtomMode::Link => mode.blue(),
            }
        };
        let arrow = match self.mode {
            AtomMode::FileCopy => "==>".bright_yellow(),
            AtomMode::Link => "~~>".blue(),
        };
        let path_display =
            |f: &mut std::fmt::Formatter<'_>,
             (path, exists): &(PathBuf, bool)| {
                if *exists {
                    write!(f, "[{}]", path.display())
                } else {
                    write!(f, "[{}]", format!("{}", path.display()).red(),)
                }
            };
        write!(f, "{} :: ", mode)?;
        path_display(f, &self.src)?;
        write!(f, " {} ", arrow)?;
        path_display(f, &self.dst)?;
        Ok(())
    }
}
