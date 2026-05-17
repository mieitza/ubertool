//! Email normalization + validation.

use std::str::FromStr;

use clap::{Args, Subcommand};
use email_address::EmailAddress;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EmailArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Normalize and validate an email address.
    #[command(
        long_about = "Normalize and validate an email address.\n\nExamples:\n  ubertool email normalize Alice@Example.COM\n  ubertool email normalize ' bob@example.com ' --json\n\nExit codes:\n  3   invalid email address (invalid_email)"
    )]
    Normalize(NormalizeArgs),
}

#[derive(Debug, Args)]
pub struct NormalizeArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    email: String,
    valid: bool,
}

pub fn dispatch(args: EmailArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Normalize(a) => run(a, out),
    }
}

fn run(args: NormalizeArgs, out: &Out) -> Result<(), CliError> {
    let trimmed = args.input.trim();
    let parsed = EmailAddress::from_str(trimmed).map_err(|e| {
        CliError::new(ErrorCode::InvalidEmail, format!("invalid email: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    let lower = parsed.to_string().to_lowercase();
    out.emit_value(&Out0 {
        email: lower,
        valid: true,
    })
}
