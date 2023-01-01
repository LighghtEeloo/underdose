use super::*;
use crate::store::AtomMode;
use crate::{dynamics::Execution, utils::Prompt};
use std::{fs, io};

impl Execution for PillTask {
    fn execution(self) -> anyhow::Result<()> {
        match self.inner {
            PillTaskInner::GitModule { remote } => todo!(),
            PillTaskInner::Addicted { atoms } => {
                for atom in atoms {
                    atom.execution()?;
                }
            }
        }
        Ok(())
    }
}

impl Execution for AtomTask {
    fn execution(self) -> anyhow::Result<()> {
        fs::create_dir_all(
            self.dst
                .parent()
                .ok_or_else(|| anyhow::anyhow!("no parent for destination"))?,
        )?;
        if self.dst.exists() {
            Prompt::new(&format!(
                "target <{}> already exists.\noverwrite? [N/y/!] ",
                self.dst.display()
            ))
            .process(|s| {
                match s {
                    "y" => {
                        if self.dst.is_file() || self.dst.is_symlink() {
                            fs::remove_file(&self.dst)?;
                        } else if self.dst.is_dir() {
                            fs::remove_dir_all(&self.dst)?;
                        }
                    }
                    "!" => {
                        println!("abort!");
                        Err(io::Error::new(io::ErrorKind::Other, "abort!"))?;
                    }
                    _ => {
                        println!("skipping...");
                    }
                };
                Ok(())
            })?;
        }
        if self.dst.exists() {
            // meant to skip
            return Ok(());
        }
        log::trace!("exec -- {}", self);
        match self.mode {
            AtomMode::FileCopy => {
                fs::copy(&self.src, &self.dst)?;
            }
            AtomMode::Link => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&self.src, &self.dst)?;
                }
                #[cfg(windows)]
                {
                    if self.src.is_file() {
                        std::os::windows::fs::symlink_file(
                            &self.src, &self.dst,
                        )?;
                    } else if self.src.is_dir() {
                        std::os::windows::fs::symlink_dir(
                            &self.src, &self.dst,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}
