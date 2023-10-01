use crate::{ArrowSrc, Drip};
use git2::build::RepoBuilder;
use std::path::Path;

pub struct Executor<'a> {
    /// where the root of repo is, globally
    pub repo: &'a Path,
    /// the task to complete
    pub drip: &'a Drip,
}

impl<'a> Executor<'a> {
    pub fn run(self) -> anyhow::Result<()> {
        for arrow in self.drip.arrows.iter() {
            let site = self.drip.site.join(&arrow.site);
            match &arrow.src {
                ArrowSrc::Git(remote) => {
                    log::info!("git clone {} {}", remote, site.display());
                    std::fs::create_dir_all(&site)?;
                    RepoBuilder::new().clone(remote, &site)?;
                }
                ArrowSrc::Link { rel } => {
                    let repo = self.repo.join(&self.drip.rel_repo).join(rel);

                    log::info!("ln -s {} {}", repo.display(), site.display());
                    let repo = crate::utils::path::canonicalize(repo)?;
                    repo.exists()
                        .then_some(())
                        .ok_or(anyhow::anyhow!("repo not exists"))?;
                    let site = crate::utils::path::canonicalize(site)?;
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(repo, site)?;
                    }
                    {}
                    #[cfg(windows)]
                    {
                        std::os::windows::fs::symlink_dir(rel, site)?;
                    }
                    #[cfg(not(any(unix, windows)))]
                    {
                        unimplemented!("symlink not supported on this platform")
                    }
                }
                ArrowSrc::Collector => {
                    log::info!("collector {}", site.display());
                }
            }
        }
        Ok(())
    }
}