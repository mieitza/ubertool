//! MIME type lookup from extension.

use std::collections::HashMap;
use std::sync::OnceLock;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MIME_CSV: &str = include_str!("../../data/mime_types.csv");

#[derive(Debug, Args)]
pub struct MimeArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Look up the MIME type for a file extension or filename.
    #[command(long_about = "Look up the MIME type for a file extension or filename.\n\nAccepts the extension alone (e.g., `json`), with a leading dot (`.json`), or a full filename (`report.pdf`). Lookup is case-insensitive.\n\nReturns `mime: null` for unknown extensions (no error).\n\nExamples:\n  ubertool mime lookup json\n  ubertool mime lookup report.pdf --json")]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    extension: String,
    mime: Option<String>,
}

fn mime_table() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::new();
        for line in MIME_CSV.lines().skip(1) {
            if let Some((ext, mime)) = line.split_once(',') {
                map.insert(ext.trim().to_lowercase(), mime.trim().to_string());
            }
        }
        map
    })
}

pub fn dispatch(args: MimeArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Lookup(a) => run(a, out),
    }
}

fn run(args: LookupArgs, out: &Out) -> Result<(), CliError> {
    let ext = extract_extension(&args.input);
    let mime = mime_table().get(&ext).cloned();
    out.emit_value(&Out0 { extension: ext, mime })
}

fn extract_extension(input: &str) -> String {
    let trimmed = input.trim();
    let no_dot = trimmed.strip_prefix('.').unwrap_or(trimmed);
    let ext = match no_dot.rsplit_once('.') {
        Some((_, e)) => e,
        None => no_dot,
    };
    ext.to_lowercase()
}
