use anyhow::Result;
use tokio::process::Command;

use crate::{shell, util::ExitStatusExt};

pub fn cargo(command: &str) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.kill_on_drop(true).args([
        if shell().no_color() {
            "--color=never"
        } else {
            "--color=always"
        },
        command,
    ]);
    cmd
}

pub fn build() -> Builder {
    Builder {
        cmd: cargo("build"),
    }
}

pub struct Builder {
    cmd: Command,
}

impl Builder {
    pub fn target(mut self, target: &str) -> Self {
        self.cmd.args(["--target", target]);
        self
    }

    pub fn profile(mut self, profile: &str) -> Self {
        self.cmd.args(["--profile", profile]);
        self
    }

    pub fn package(mut self, package: &str) -> Self {
        self.cmd.args(["--package", package]);
        self
    }

    pub fn bin(mut self, bin: &str) -> Self {
        self.cmd.args(["--bin", bin]);
        self
    }

    pub fn default_features(mut self, enabled: bool) -> Self {
        if !enabled {
            self.cmd.arg("--no-default-features");
        }
        self
    }

    pub fn features(mut self, features: &[String]) -> Self {
        if !features.is_empty() {
            self.cmd.arg(format!("--features={}", features.join(",")));
        }
        self
    }

    pub async fn build(mut self) -> Result<()> {
        self.cmd.status().await?.exit_ok()?;
        Ok(())
    }
}
