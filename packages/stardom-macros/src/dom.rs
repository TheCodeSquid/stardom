use std::sync::OnceLock;

use serde::Deserialize;

const DATA_SOURCE: &str = include_str!("../dom.toml");

static DOM: OnceLock<Dom> = OnceLock::new();

#[derive(Deserialize)]
struct Dom {
    pub elements: Vec<String>,
    pub attributes: Vec<String>,
}

impl Dom {
    pub fn get() -> &'static Dom {
        DOM.get_or_init(|| toml::from_str(DATA_SOURCE).unwrap())
    }
}

pub fn elements() -> &'static [String] {
    &Dom::get().elements
}

pub fn attributes() -> &'static [String] {
    &Dom::get().attributes
}
