//! ULID generator.

use clap::{Args, Subcommand};
use serde::Serialize;
use ulid::Ulid;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UlidArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a new ULID (lexically-sortable 128-bit identifier).
    #[command(long_about = "Generate a new ULID (lexically-sortable, 128-bit, 26-char Crockford base32).\n\nExamples:\n  ubertool ulid new\n  ubertool ulid new --json")]
    New,
}

#[derive(Serialize)]
struct UlidOutput {
    ulid: String,
}

pub fn dispatch(args: UlidArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => out.emit_value(&UlidOutput {
            ulid: Ulid::new().to_string(),
        }),
    }
}
