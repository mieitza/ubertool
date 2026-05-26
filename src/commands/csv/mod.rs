//! CSV to JSON conversion.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct CsvArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert CSV to JSON (header row becomes object keys).
    #[command(
        name = "to-json",
        long_about = "Convert CSV to JSON. The first row is treated as the header — its values become object keys for every subsequent row.\n\nExamples:\n  ubertool csv to-json 'name,age\\nalice,30'\n  ubertool csv to-json --in ./data.csv --delimiter ';'\n  ubertool csv to-json --in ./data.csv --json\n\nExit codes specific to this command:\n  2   usage error (e.g., multi-char delimiter)\n  3   malformed CSV (invalid_csv)"
    )]
    ToJson(ToJsonArgs),
}

#[derive(Debug, Args)]
pub struct ToJsonArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Single ASCII character delimiter (default `,`).
    #[arg(long, default_value = ",")]
    pub delimiter: String,
}

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn dispatch(args: CsvArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => run(a, out),
    }
}

fn run(args: ToJsonArgs, out: &Out) -> Result<(), CliError> {
    if args.delimiter.len() != 1 || !args.delimiter.is_ascii() {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "--delimiter must be a single ASCII character",
        ));
    }
    let delim_byte = args.delimiter.as_bytes()[0];
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let bytes = input.as_bytes();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delim_byte)
        .from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|e| CliError::new(ErrorCode::InvalidCsv, format!("invalid CSV header: {e}")))?
        .iter()
        .map(String::from)
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for rec in reader.records() {
        let rec = rec.map_err(|e| {
            CliError::new(ErrorCode::InvalidCsv, format!("invalid CSV row: {e}"))
                .with_hint("ensure rows are well-formed (escaped quotes, matching commas)")
        })?;
        let mut obj = serde_json::Map::new();
        for (i, field) in rec.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| i.to_string());
            obj.insert(key, serde_json::Value::String(field.to_string()));
        }
        rows.push(serde_json::Value::Object(obj));
    }

    let json = serde_json::to_string(&serde_json::Value::Array(rows))
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("json failed: {e}")))?;
    out.emit_value(&Out0 { json })
}
