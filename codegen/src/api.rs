use std::{fs, path::Path};

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Api {
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(default)]
    pub attributes: IndexSet<String>,
    #[serde(default)]
    pub events: IndexMap<String, Vec<String>>,
}

impl FromIterator<Api> for Api {
    fn from_iter<T: IntoIterator<Item = Api>>(iter: T) -> Self {
        // TODO: warn about conflicts

        let mut out = Api::default();
        for api in iter {
            out.elements.extend(api.elements);
            out.attributes.extend(api.attributes);
            out.events.extend(api.events);
        }
        out
    }
}

impl Api {
    pub fn parse_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut paths: Vec<_> = path
            .as_ref()
            .read_dir()?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();

        paths.into_iter().map(Self::parse_file).collect()
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        let api = toml::from_str(&source)?;
        Ok(api)
    }
}
