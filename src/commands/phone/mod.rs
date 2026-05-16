//! Phone number parser (libphonenumber via phonenumber crate).

use clap::{Args, Subcommand};
use phonenumber::country;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct PhoneArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse a phone number and emit structured info.
    #[command(long_about = "Parse a phone number (E.164 or with --region default) and emit country, national format, E.164.\n\nExamples:\n  ubertool phone parse +14155552671\n  ubertool phone parse '415 555 2671' --region US --json\n\nExit codes:\n  3   invalid phone number (invalid_phone)")]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
    /// Default region (ISO 3166-1 alpha-2, e.g., US, GB).
    #[arg(long)]
    pub region: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    country: String,
    country_code: u16,
    national: String,
    e164: String,
}

pub fn dispatch(args: PhoneArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => run(a, out),
    }
}

fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let region: Option<country::Id> = args
        .region
        .as_deref()
        .map(|r| r.parse::<country::Id>())
        .transpose()
        .map_err(|e| CliError::new(ErrorCode::UsageError, format!("invalid --region: {e:?}")))?;

    let num = phonenumber::parse(region, &args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidPhone, format!("invalid phone: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;

    if !num.is_valid() {
        return Err(CliError::new(ErrorCode::InvalidPhone, "phone number failed validation")
            .with_input(serde_json::json!(args.input)));
    }

    let cc = num.country().code();
    let country_label = num
        .country()
        .id()
        .map(|i| format!("{i:?}"))
        .unwrap_or_default();
    let national = phonenumber::format(&num)
        .mode(phonenumber::Mode::National)
        .to_string();
    let e164 = phonenumber::format(&num)
        .mode(phonenumber::Mode::E164)
        .to_string();

    out.emit_value(&Out0 {
        country: country_label,
        country_code: cc,
        national,
        e164,
    })
}
