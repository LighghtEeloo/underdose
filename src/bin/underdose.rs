// 0. Configuration
// 1. Perfect State Executor (Executor)
// 2. Perfect State Detective
// 3. Garbage Collector (Dreamer)

use std::path::PathBuf;
use underdose::{Arrow, ArrowSrc, Drip, Executor};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let repo = PathBuf::from("_ana/drugstore");
    let drips = vec![
        Drip {
            site: "_ana/config/nvim".into(),
            rel_repo: "nvim".into(),
            arrows: vec![Arrow {
                rel_site: ".".into(),
                src: ArrowSrc::Link(".".into()),
            }],
        },
        Drip {
            site: "_ana/config/alacritty-theme".into(),
            rel_repo: "alacritty-theme".into(),
            arrows: vec![Arrow {
                rel_site: ".".into(),
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
