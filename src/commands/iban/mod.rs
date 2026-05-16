//! IBAN validation.

use clap::{Args, Subcommand};
use iban::{IbanLike, Iban};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct IbanArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Validate an IBAN check digit (mod-97).
    #[command(long_about = "Validate an IBAN using the mod-97 check digit algorithm. Spaces and other separators are stripped.\n\nExamples:\n  ubertool iban validate GB82WEST12345698765432\n  ubertool iban validate 'GB82 WEST 1234 5698 7654 32' --json\n\nExit codes:\n  3   invalid IBAN (invalid_iban)")]
    Validate(ValidateArgs),
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    iban: String,
    country: String,
    valid: bool,
}

pub fn dispatch(args: IbanArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Validate(a) => run(a, out),
    }
}

fn run(args: ValidateArgs, out: &Out) -> Result<(), CliError> {
    let iban = args.input.parse::<Iban>().map_err(|e| {
        CliError::new(ErrorCode::InvalidIban, format!("invalid IBAN: {e}"))
            .with_input(serde_json::json!(args.input))
            .with_hint("verify country code, length, and check digits")
    })?;
    out.emit_value(&Out0 {
        iban: iban.electronic_str().to_string(),
        country: iban.country_code().to_string(),
        valid: true,
    })
}
