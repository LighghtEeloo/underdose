use crate::utils::path;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[derive(Default, Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub env: HashSet<String>,
    pub local: PathBuf,
}

mod parse {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Machine {
        pub env: HashSet<String>,
        pub repo: Repo,
        pub tutorial: Option<()>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct Repo {
        pub name: String,
        pub local: PathBuf,
    }
}

impl TryFrom<&str> for Machine {
    type Error = anyhow::Error;

    fn try_from(buf: &str) -> anyhow::Result<Self> {
        let conf: parse::Machine = toml::from_str(buf)?;
        conf.try_into()
    }
}

impl TryFrom<parse::Machine> for Machine {
    type Error = anyhow::Error;

    fn try_from(
        parse::Machine {
            env,
            repo: parse::Repo { name, local },
            tutorial,
        }: parse::Machine,
    ) -> Result<Self, Self::Error> {
        if let Some(_) = tutorial {
            Err(anyhow::anyhow!("tutorial has not been completed yet"))?;
        }

        Ok(Self {
            name,
            env,
            local: path::expand_home(&local),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_machine() {
        let content = crate::utils::tests::remove_tutorial(crate::utils::conf::UNDERDOSE_TOML);

        let machine = Machine::try_from(&content[..]).unwrap();
        println!("{:#?}", machine);
    }
}
