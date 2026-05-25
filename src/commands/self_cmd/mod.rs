//! Self-update and version commands.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod update;
pub mod version;

#[derive(Debug, Args)]
pub struct SelfArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Show the running binary's version, optionally check for updates.
    #[command(
        long_about = "Show the current ubertool version.\n\nExamples:\n  ubertool self version\n  ubertool self version --json"
    )]
    Version,
    /// Update the binary in place from the latest GitHub release.
    #[command(
        long_about = "Update the ubertool binary in place from the latest GitHub release.\n\nBy default, downloads and replaces the running binary atomically.\n\nWith --check, just reports current vs latest without updating.\n\nExamples:\n  ubertool self update              # update if newer available\n  ubertool self update --check      # report only, no install\n  ubertool self update --json       # machine-readable update result\n\nExit codes:\n  3   could not parse release metadata or version string\n  4   network failure / file write failure during install"
    )]
    Update(update::UpdateArgs),
}

pub fn dispatch(args: SelfArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Version => version::run(out),
        Verb::Update(a) => update::run(a, out),
    }
}
