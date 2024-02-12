use std::{
    collections::{BTreeSet, HashMap},
    fs,
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use crate::{shell, tools::OptLevel};

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    pub project: Project,
    #[serde(rename = "profile")]
    pub profiles: HashMap<String, Profile>,
    pub build: Build,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct Project {
    pub package: Option<String>,
    pub bin: Option<String>,
    pub default_features: bool,
    pub features: Vec<String>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            package: None,
            bin: None,
            default_features: true,
            features: vec![],
        }
    }
}

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct Profile {
    pub opt_level: OptLevel,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct Build {
    pub out_dir: Utf8PathBuf,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            out_dir: "dist".into(),
        }
    }
}

impl Config {
    pub fn load(path: &Utf8Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("`{path}` is not a valid config file"))?;

        let mut unused = BTreeSet::new();
        let config: Self =
            serde_ignored::deserialize(toml::Deserializer::new(&contents), |path| {
                unused.insert(path.to_string());
            })?;

        for key in unused {
            shell().warn(format!("{path}: unused config key: {key}"));
        }
        Ok(config)
    }
}
