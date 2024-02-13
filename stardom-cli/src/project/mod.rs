mod watch;

use std::env;

use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, MetadataCommand, Package, Target};
use thiserror::Error;
use tokio::{fs, task};

use crate::{
    config::{Config, Profile},
    shell,
    tools::{self, OptLevel},
    util::*,
};
use watch::Watcher;

#[derive(Clone)]
pub struct Project {
    pub root: Utf8PathBuf,
    pub meta: Metadata,
    pub config: Config,
    pub config_path: Option<Utf8PathBuf>,
    watcher: Watcher,
}

impl Project {
    // Commands

    pub async fn clean(&self) -> Result<()> {
        let out_dir = self.out_dir();
        if out_dir.exists() {
            fs::remove_dir_all(self.out_dir()).await?;
        }

        tools::cargo("clean").status().await?.exit_ok()?;

        Ok(())
    }

    pub async fn build(&self, profile: &str) -> Result<()> {
        let _build = self.watcher.build_lock();

        let profile_config = self.profile_config(profile);
        let package = self.primary_package()?;
        let bin = self.primary_bin(package)?;
        let target_dir = self
            .meta
            .target_directory
            .join("wasm32-unknown-unknown")
            .join(match profile {
                "dev" => "debug",
                _ => profile,
            });

        // cargo build

        tools::cargo::build()
            .target("wasm32-unknown-unknown")
            .profile(profile)
            .package(&package.name)
            .bin(bin)
            .default_features(self.config.project.default_features)
            .features(&self.config.project.features)
            .build()
            .await?;

        // wasm-bindgen

        let wasm_file = target_dir.join(bin).with_extension("wasm");
        let wasm_dir = target_dir.join("wasm-bindgen");
        tools::wasm_bindgen(&wasm_file, &wasm_dir).await?;

        // wasm-opt

        let bg_file = wasm_dir.join(format!("{bin}_bg.wasm"));
        let temp_file = wasm_dir.join(format!("{bin}_temp.wasm"));

        tools::wasm_opt(profile_config.opt_level, &bg_file, &temp_file).await?;
        fs::rename(temp_file, bg_file).await?;

        // finalization

        let out_dir = self.out_dir();
        if out_dir.exists() {
            fs::remove_dir_all(&out_dir).await?;
        }
        fs::rename(wasm_dir, out_dir).await?;

        Ok(())
    }

    pub async fn watch(mut self, profile: &str) -> Result<()> {
        let out_dir = self.out_dir();
        let target_dir = self.meta.target_directory.clone();
        self.watcher = Watcher::watch(".", move |path| {
            !path.starts_with(&out_dir) && !path.starts_with(&target_dir)
        })?;

        loop {
            let profile = profile.to_string();
            let project = self.clone();

            task::spawn(async move {
                if let Err(err) = project.build(&profile).await {
                    if !is_error_silent(&err) {
                        shell().error(err);
                    }
                }
            });

            self.watcher.recv().await?;
        }
    }

    // Utilities

    fn profile_config(&self, profile: &str) -> Profile {
        let mut config = self
            .config
            .profiles
            .get(profile)
            .cloned()
            .unwrap_or_default();
        if config.opt_level == OptLevel::Default {
            config.opt_level = if profile == "dev" {
                OptLevel::One
            } else {
                OptLevel::S
            };
        }
        config
    }

    fn primary_package(&self) -> Result<&Package> {
        if let Some(name) = &self.config.project.package {
            let package = self
                .meta
                .packages
                .iter()
                .find(|package| &package.name == name)
                .ok_or_else(|| anyhow!("package {name} not found in workspace"))?;
            Ok(package)
        } else {
            self.meta
                .primary_package()
                .ok_or_else(|| anyhow!("could not determine primary package"))
        }
    }

    fn primary_bin<'a>(&'a self, package: &'a Package) -> Result<&'a str> {
        if let Some(bin) = &self.config.project.bin {
            Ok(bin)
        } else {
            let targets = bin_targets(package);
            match targets.len() {
                0 => Err(anyhow!("package {} has no binary targets", package.name)),
                1 => Ok(&targets[0].name),
                _ => {
                    if let Some(bin) = &package.default_run {
                        Ok(bin)
                    } else {
                        Err(anyhow!("could not determine primary binary target"))
                    }
                }
            }
        }
    }

    fn out_dir(&self) -> Utf8PathBuf {
        self.root.join(&self.config.build.out_dir)
    }

    pub fn from_env(config: Option<&Utf8Path>) -> Result<Self> {
        if let Some(path) = config {
            let root = path.canonicalize_utf8()?.parent().unwrap().to_path_buf();
            let meta = MetadataCommand::new().current_dir(&root).no_deps().exec()?;
            let config = Config::load(path)?;
            Ok(Self {
                root,
                meta,
                config,
                config_path: Some(path.to_path_buf()),
                watcher: Watcher::off(),
            })
        } else {
            let meta = MetadataCommand::new().no_deps().exec()?;
            if let Some(path) = find_file(&meta)? {
                let config = Config::load(&path)?;
                let root = path.parent().unwrap().to_path_buf();
                Ok(Self {
                    root,
                    meta,
                    config,
                    config_path: Some(path),
                    watcher: Watcher::off(),
                })
            } else {
                let config = Config::default();
                let root = meta.workspace_root.clone();
                Ok(Self {
                    root,
                    meta,
                    config,
                    config_path: None,
                    watcher: Watcher::off(),
                })
            }
        }
    }
}

fn find_file(meta: &Metadata) -> Result<Option<Utf8PathBuf>> {
    let root = &meta.workspace_root;
    let start = if let Some(package) = meta.primary_package() {
        package.manifest_path.clone()
    } else {
        env::current_dir()?.try_into()?
    };

    for path in start.ancestors() {
        if !path.starts_with(root) {
            break;
        }
        let config_path = path.join("stardom.toml");
        if config_path.is_file() {
            return Ok(Some(config_path));
        }
    }
    Ok(None)
}

fn bin_targets(package: &Package) -> Vec<&Target> {
    package
        .targets
        .iter()
        .filter(|target| target.kind.iter().any(|k| k == "bin"))
        .collect()
}

#[derive(Clone, Copy, Error, Debug)]
#[error("command aborted")]
pub(crate) struct WatchAbortError;
