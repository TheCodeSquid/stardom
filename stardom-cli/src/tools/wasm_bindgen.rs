use anyhow::{bail, Result};
use camino::Utf8Path;

use crate::util::ExitStatusExt;

use super::{Target, Tool, Tools};

const TOOLS: Tools = Tools {
    name: "wasm-bindgen",
    version: "0.2.91",
    url: |version, Target { os, arch }| {
        let target = match (os, arch) {
            ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
            ("macos", "x86_64" | "aarch64") => format!("{arch}-apple-darwin"),
            ("linux", "x86_64" | "aarch64") => format!("{arch}-unknown-linux-musl"),
            _ => bail!("unable to download wasm-bindgen for {os} {arch}"),
        };
        Ok(format!("https://github.com/rustwasm/wasm-bindgen/releases/download/{version}/wasm-bindgen-{version}-{target}.tar.gz"))
    },
    tools: &["wasm-bindgen"],
};

const WASM_BINDGEN: Tool = TOOLS.tool("wasm-bindgen");

pub async fn wasm_bindgen(input: &Utf8Path, out_dir: &Utf8Path) -> Result<()> {
    WASM_BINDGEN
        .command()
        .await?
        .kill_on_drop(true)
        .args([
            input.as_str(),
            "--target=web",
            &format!("--out-dir={}", out_dir),
        ])
        .status()
        .await?
        .exit_ok()?;
    Ok(())
}
