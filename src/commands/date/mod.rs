//! Date / time format conversion.

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct DateArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert between date/time formats.
    #[command(
        long_about = "Convert between unix timestamp, ISO 8601, and RFC 2822 representations of a moment in time.\n\nExamples:\n  ubertool date convert 1700000000 --from unix\n  ubertool date convert '2023-11-14T22:13:20Z' --from iso8601 --json\n  ubertool date convert 1700000000 --from unix --tz America/New_York\n\nExit codes:\n  3   invalid date/time input (invalid_date)"
    )]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Unix,
    Iso8601,
    Rfc2822,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: String,
    #[arg(long, value_enum)]
    pub from: Format,
    #[arg(long)]
    pub tz: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    unix: i64,
    iso8601: String,
    rfc2822: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    local: Option<String>,
}

pub fn dispatch(args: DateArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let dt: DateTime<Utc> = match args.from {
        Format::Unix => {
            let ts: i64 = args.input.parse().map_err(|_| {
                CliError::new(
                    ErrorCode::InvalidDate,
                    format!("invalid unix timestamp: {}", args.input),
                )
            })?;
            Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
                CliError::new(
                    ErrorCode::InvalidDate,
                    format!("timestamp out of range: {ts}"),
                )
            })?
        }
        Format::Iso8601 => DateTime::parse_from_rfc3339(&args.input)
            .map_err(|e| {
                CliError::new(ErrorCode::InvalidDate, format!("invalid ISO 8601: {e}"))
                    .with_input(serde_json::json!(args.input))
            })?
            .with_timezone(&Utc),
        Format::Rfc2822 => DateTime::parse_from_rfc2822(&args.input)
            .map_err(|e| CliError::new(ErrorCode::InvalidDate, format!("invalid RFC 2822: {e}")))?
            .with_timezone(&Utc),
    };
    let local = if let Some(tz_name) = &args.tz {
        let tz: Tz = tz_name.parse().map_err(|_| {
            CliError::new(
                ErrorCode::InvalidDate,
                format!("unknown timezone: {tz_name}"),
            )
            .with_hint("use IANA timezone names like America/New_York or Europe/Berlin")
        })?;
        Some(dt.with_timezone(&tz).to_rfc3339())
    } else {
        None
    };
    out.emit_value(&Out0 {
        unix: dt.timestamp(),
        iso8601: dt.to_rfc3339(),
        rfc2822: dt.to_rfc2822(),
        local,
    })
}
