use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Api {
    pub elements: IndexSet<String>,
    pub void: IndexSet<String>,
    pub attributes: IndexSet<String>,
    pub events: IndexMap<String, Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(default)]
    pub valid_elements: Vec<String>,
}

impl FromIterator<Api> for Api {
    fn from_iter<T: IntoIterator<Item = Api>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|mut api, other| {
                api.elements.extend(other.elements);
                api.void.extend(other.void);
                api.attributes.extend(other.attributes);
                api.events.extend(other.events);
                api
            })
            .unwrap_or_default()
    }
}
