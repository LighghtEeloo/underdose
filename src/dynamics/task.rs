use crate::store::AtomMode;
use colored::Colorize;
use git_url_parse::GitUrl;
use std::fmt::Display;
use std::path::PathBuf;

pub struct PillTask {
    pub name: String,
    pub root: AtomTask,
    pub inner: PillTaskInner,
}

pub enum PillTaskInner {
    GitModule { remote: Box<GitUrl> },
    Addicted { atoms: Vec<AtomTask> },
}

impl Display for PillTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n[[{}]]", self.name)?;
        writeln!(f, "|| {}", self.root)?;
        match &self.inner {
            PillTaskInner::GitModule { remote } => {
                writeln!(f, "      || remote-task @ {}", remote)?;
            }
            PillTaskInner::Addicted { atoms } => {
                for atom in atoms {
                    writeln!(f, "      || {}", atom)?;
                }
            }
        }
        Ok(())
    }
}

pub struct AtomTask {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub mode: AtomMode,
}

impl Display for AtomTask {
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
        write!(
            f,
            "{} :: [{}] {} [{}]",
            mode,
            self.src.display(),
            arrow,
            self.dst.display()
        )
    }
}
