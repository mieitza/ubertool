//! Convert `docker run` commands to docker-compose service entries.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod parser;
pub mod to_compose;

#[derive(Debug, Args)]
pub struct DockerRunArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a `docker run` command to a docker-compose service entry.
    #[command(name = "to-compose", long_about = "Convert a `docker run` command to a docker-compose service entry.\n\nSupported flags: --name, -p/--publish, -v/--volume, -e/--env, --restart, --network. Other flags are ignored in this build.\n\nExamples:\n  ubertool docker-run to-compose 'docker run --name web -p 8080:80 nginx'\n  ubertool docker-run to-compose --in ./cmd.txt --json\n\nExit codes:\n  3   malformed docker-run command (e.g., missing image)")]
    ToCompose(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

pub fn dispatch(args: DockerRunArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToCompose(a) => to_compose::run(a, out),
    }
}
