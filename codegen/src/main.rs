mod api;
mod generate;

use std::fs::File;

use anyhow::Result;

use api::Api;

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");
const INCLUDE_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../stardom-macros/src/include.rs"
);

fn main() -> Result<()> {
    let api = Api::parse_dir(DATA_DIR)?;

    let file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(INCLUDE_FILE)?;
    generate::include(file, &api)?;

    Ok(())
}
