mod data;
mod gen;

use std::{
    env,
    fs::{self, File},
};

use anyhow::Result;
use data::Api;

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
const OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/include.rs");

fn main() -> Result<()> {
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| OUTPUT_PATH.to_string());

    let data_dir = fs::read_dir(DATA_DIR)?;

    let api = data_dir
        .map(|entry| {
            let path = entry?.path();
            let text = fs::read_to_string(path)?;
            let api = toml::from_str(&text)?;
            Ok::<_, anyhow::Error>(api)
        })
        .collect::<Result<Api>>()?;

    let include_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;
    gen::stardom_macro_include(include_file, &api)?;

    Ok(())
}
