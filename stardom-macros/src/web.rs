use std::sync::OnceLock;

use serde::Deserialize;

const DATA_SOURCE: &str = include_str!("../web.json");

static DATA: OnceLock<Data> = OnceLock::new();

#[derive(Deserialize)]
pub struct Data {
    pub elements: Vec<String>,
    pub attrs: Vec<String>,
    pub events: Vec<(String, String)>,
}

impl Data {
    pub fn get() -> &'static Data {
        DATA.get_or_init(|| serde_json::from_str(DATA_SOURCE).unwrap())
    }
}
