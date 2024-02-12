mod binaryen;
pub mod cargo;
mod wasm_bindgen;

pub use binaryen::*;
pub use cargo::cargo;
pub use wasm_bindgen::*;

use std::{
    collections::HashSet,
    ffi::OsStr,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{anyhow, bail, Result};
use flate2::read::GzDecoder;
use tar::Archive;
use tokio::{process::Command, task};
use which::which;

use crate::shell;

#[derive(Clone, Copy)]
struct Tools {
    name: &'static str,
    version: &'static str,
    url: fn(&'static str, Target) -> Result<String>,
    tools: &'static [&'static str],
}

impl Tools {
    fn cache_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap()
            .join("stardom")
            .join(format!("{}-{}", self.name, self.version))
    }

    fn url(&self) -> Result<String> {
        (self.url)(self.version, Target::new()?)
    }

    async fn download(&'static self) -> Result<()> {
        let start = Instant::now();
        shell().progress("Downloading", format!("{} {}", self.name, self.version));

        let url = self.url()?;

        let cache_dir = self.cache_dir();
        if !cache_dir.exists() {
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        // for use in later logging
        let cache = cache_dir.clone();

        task::spawn_blocking(move || {
            let response = ureq::get(&url).call()?;
            let decoder = GzDecoder::new(response.into_reader());
            let mut archive = Archive::new(decoder);

            let mut needs: HashSet<_> = self
                .tools
                .iter()
                .map(|name| {
                    if cfg!(target_os = "windows") {
                        format!("{name}.exe")
                    } else {
                        name.to_string()
                    }
                })
                .collect();
            for result in archive.entries()? {
                let mut entry = result?;
                let path = entry.path()?;
                let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                    continue;
                };

                if needs.remove(name) {
                    let path = cache_dir.join(name);
                    let mut bytes = vec![];
                    entry.read_to_end(&mut bytes)?;
                    install(path, &bytes)?;
                }
            }

            if !needs.is_empty() {
                Err(anyhow!(
                    "files missing from archive ({})",
                    needs.into_iter().collect::<Vec<_>>().join(", ")
                ))
            } else {
                Ok(())
            }
        })
        .await??;

        shell().status(
            "Downloaded",
            format!(
                "{} to {} (took {}s)",
                self.name,
                cache.display(),
                start.elapsed().as_secs_f32().round()
            ),
        );

        Ok(())
    }

    pub const fn tool(&'static self, name: &'static str) -> Tool {
        Tool { tools: self, name }
    }
}

fn install<P: AsRef<Path>>(path: P, bytes: &[u8]) -> io::Result<()> {
    let mut file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }
    Ok(())
}

pub struct Tool {
    tools: &'static Tools,
    name: &'static str,
}

impl Tool {
    pub async fn command(&self) -> Result<Command> {
        if let Ok(path) = which(self.name) {
            Ok(Command::new(path))
        } else {
            let dir = self.tools.cache_dir();
            if !dir.exists() {
                self.tools.download().await?;
            }
            Ok(Command::new(dir.join(self.name)))
        }
    }
}

#[derive(Clone, Copy)]
struct Target {
    pub os: &'static str,
    pub arch: &'static str,
}

impl Target {
    fn new() -> Result<Self> {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            bail!("unsupported OS")
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            bail!("unsupported architecture")
        };

        Ok(Self { os, arch })
    }
}
