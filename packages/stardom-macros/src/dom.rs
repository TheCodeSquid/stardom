use std::sync::OnceLock;

use serde::Deserialize;

const DATA_SOURCE: &str = include_str!("../dom.json");

static DOM: OnceLock<Dom> = OnceLock::new();

#[derive(Deserialize)]
struct Dom {
    elements: Vec<String>,
    attributes: Vec<String>,
    events: Vec<Event>,
}

#[derive(Deserialize, Clone)]
pub struct Event {
    pub name: String,
    pub interface: String,
}

impl Dom {
    pub fn get() -> &'static Dom {
        DOM.get_or_init(|| serde_json::from_str(DATA_SOURCE).unwrap())
    }
}

pub fn elements() -> &'static [String] {
    &Dom::get().elements
}

pub fn attributes() -> &'static [String] {
    &Dom::get().attributes
}

pub fn events() -> &'static [Event] {
    &Dom::get().events
}
