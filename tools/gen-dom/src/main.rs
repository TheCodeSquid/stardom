use std::{env, fs, path::PathBuf};

use anyhow::Result;
use scraper::{ElementRef, Html};
use serde::Serialize;

const MDN_ELEMENTS: &str = "https://developer.mozilla.org/en-US/docs/Web/HTML/Element";

#[derive(Default, Serialize)]
struct Data {
    elements: Vec<String>,
}

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("missing output path");

    let mut data = Data::default();

    let fetcher = Fetcher::new();
    let html = fetcher.fetch(MDN_ELEMENTS)?;

    let skip = ["web_components", "obsolete", "see_also"];
    let sections_select = format!(
        "article section{}",
        skip.map(|s| format!(":not([aria-labelledby^='{s}'])"))
            .join("")
    )
    .as_str()
    .try_into()
    .unwrap();

    let elem_select = "tr td:first-child code".try_into().unwrap();

    for section in html.select(&sections_select) {
        for elem in section.select(&elem_select) {
            let untrimmed = text(&elem);
            let name = &untrimmed[1..untrimmed.len() - 1];

            data.elements.push(name.to_string());
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
