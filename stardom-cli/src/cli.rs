use std::process;

use anyhow::Result;
use camino::Utf8Path;
use clap::{ArgMatches, Args, FromArgMatches, Parser, Subcommand};

use crate::{project::Project, shell};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build { project, profile } => project.build(&profile.0).await,
        Command::Watch { project, profile } => project.watch(&profile.0).await,
        Command::Clean { project } => project.clean().await,
    }
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Build {
        #[command(flatten)]
        project: Project,
        #[command(flatten)]
        profile: Profile,
    },
    Watch {
        #[command(flatten)]
        project: Project,
        #[command(flatten)]
        profile: Profile,
    },
    Clean {
        #[command(flatten)]
        project: Project,
    },
}

impl Args for Project {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.arg(clap::arg!(-c --config <PATH> "Path to the stardom.toml file"))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for Project {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let config_path = matches.get_one::<String>("config").map(Utf8Path::new);
        let project = Self::from_env(config_path).unwrap_or_else(|err| {
            shell().error(format!("{err}"));
            process::exit(1);
        });
        Ok(project)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

struct Profile(String);

impl Args for Profile {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.args([
            clap::arg!(-r --release "Build with the release profile"),
            clap::arg!(--profile <NAME> "Build with the given profile").conflicts_with("release"),
        ])
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for Profile {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        if matches.get_flag("release") {
            Ok(Self("release".to_string()))
        } else if let Some(profile) = matches.get_one::<String>("profile") {
            Ok(Self(profile.clone()))
        } else {
            Ok(Self("dev".to_string()))
        }
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}
