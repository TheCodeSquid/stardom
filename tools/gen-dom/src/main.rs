use std::{env, fs, path::PathBuf};

use anyhow::Result;
use scraper::{ElementRef, Html};
use serde::Serialize;

const WHATWG_INDEX: &str = "https://html.spec.whatwg.org/multipage/indices.html";

#[derive(Default, Serialize)]
struct Data {
    elements: Vec<String>,
    attributes: Vec<String>,
}

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("missing output path");

    let mut data = Data::default();
    let fetcher = Fetcher::new();

    // Elements
    let html = fetcher.fetch(WHATWG_INDEX)?;

    let select = "table:nth-of-type(1) tbody th code".try_into().unwrap();
    for elem in html.select(&select) {
        let name = text(&elem);

        data.elements.push(name);
    }

    // Attributes

    let select = "table:nth-of-type(3) tbody th code".try_into().unwrap();
    for attr in html.select(&select) {
        let name = text(&attr);

        if !data.attributes.contains(&name) {
            data.attributes.push(name);
        }
    }

    let output = toml::to_string_pretty(&data)?;
    fs::write(path, output)?;
    Ok(())
}

fn text(elem: &ElementRef) -> String {
    elem.text().collect::<Vec<_>>().join(" ")
}

struct Fetcher {
    agent: ureq::Agent,
}

impl Fetcher {
    fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .user_agent(concat!("stardom-gen-dom/", env!("CARGO_PKG_VERSION")))
            .build();
        Self { agent }
    }

    fn fetch(&self, url: &str) -> Result<Html> {
        let document = self.agent.get(url).call()?.into_string()?;
        Ok(Html::parse_document(&document))
    }
}
