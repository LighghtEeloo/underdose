// 1. Perfect State Executor
// 2. Perfect State Detective
// 3. Garbage Collector

use git2::build::RepoBuilder;
use std::path::{Path, PathBuf};

pub struct Drip {
    /// where the root of site is, globally
    pub site: PathBuf,
    /// where the root of drip is, relative to repo root
    pub rel_repo: PathBuf,
    /// tasks to complete
    pub arrows: Vec<Arrow>,
}

pub struct Arrow {
    pub site: PathBuf,
    pub src: ArrowSrc,
}

pub enum ArrowSrc {
    Git(String),
    Link { rel: PathBuf },
    Collector,
}

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
                    let repo = underdose::utils::path::canonicalize(repo)?;
                    repo.exists()
                        .then_some(())
                        .ok_or(anyhow::anyhow!("repo not exists"))?;
                    let site = underdose::utils::path::canonicalize(site)?;
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

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let repo = PathBuf::from("_ana/drugstore");
    let drips = vec![
        Drip {
            site: "_ana/config/nvim".into(),
            rel_repo: "nvim".into(),
            arrows: vec![Arrow {
                site: ".".into(),
                src: ArrowSrc::Link { rel: ".".into() },
            }],
        },
        Drip {
            site: "_ana/config/alacritty-theme".into(),
            rel_repo: "alacritty-theme".into(),
            arrows: vec![Arrow {
                site: ".".into(),
                src: ArrowSrc::Git("https://github.com/alacritty/alacritty-theme".to_owned()),
            }],
        },
    ];

    for drip in drips {
        let executor = Executor {
            repo: &repo,
            drip: &drip,
        };
        executor.run()?;
    }

    Ok(())
}
