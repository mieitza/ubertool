//! UUID generator.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use uuid::Uuid;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UuidArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a new UUID.
    #[command(
        long_about = "Generate a new UUID. Default version is 4 (random).\n\nExamples:\n  ubertool uuid new\n  ubertool uuid new --version 7\n  ubertool uuid new --version 4 --json",
        disable_version_flag = true
    )]
    New(NewArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UuidVersion {
    #[value(name = "4")]
    V4,
    #[value(name = "7")]
    V7,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// UUID version (4 = random, 7 = time-ordered).
    #[arg(long, value_enum, default_value = "4")]
    pub version: UuidVersion,
}

#[derive(Serialize)]
struct UuidOutput {
    uuid: String,
}

pub fn dispatch(args: UuidArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New(a) => run(a, out),
    }
}

fn run(args: NewArgs, out: &Out) -> Result<(), CliError> {
    let u = match args.version {
        UuidVersion::V4 => Uuid::new_v4(),
        UuidVersion::V7 => Uuid::now_v7(),
    };
    out.emit_value(&UuidOutput {
        uuid: u.hyphenated().to_string(),
    })
}
