//! HTTP status code lookup.

use std::collections::HashMap;
use std::sync::OnceLock;

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::core::error::CliError;
use crate::core::output::Out;

const HTTP_STATUS_JSON: &str = include_str!("../../data/http_status.json");

#[derive(Debug, Args)]
pub struct HttpStatusArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Look up the name and category for an HTTP status code.
    #[command(long_about = "Look up an HTTP status code.\n\nReturns `valid: false` and null name/category for unknown codes (no error).\n\nExamples:\n  ubertool http-status lookup 200\n  ubertool http-status lookup 404 --json\n\nExit codes:\n  2   input is not a valid integer")]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    pub code: u16,
}

#[derive(Deserialize, Serialize, Clone)]
struct StatusEntry {
    name: String,
    category: String,
}

#[derive(Serialize)]
struct Out0 {
    code: u16,
    name: Option<String>,
    category: Option<String>,
    valid: bool,
}

fn status_table() -> &'static HashMap<u16, StatusEntry> {
    static TABLE: OnceLock<HashMap<u16, StatusEntry>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let raw: HashMap<String, StatusEntry> =
            serde_json::from_str(HTTP_STATUS_JSON).expect("http_status.json must parse");
        raw.into_iter()
            .filter_map(|(k, v)| k.parse::<u16>().ok().map(|n| (n, v)))
            .collect()
    })
}

pub fn dispatch(args: HttpStatusArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Lookup(a) => run(a, out),
    }
}

fn run(args: LookupArgs, out: &Out) -> Result<(), CliError> {
    let entry = status_table().get(&args.code).cloned();
    let (name, category, valid) = match entry {
        Some(e) => (Some(e.name), Some(e.category), true),
        None => (None, None, false),
    };
    out.emit_value(&Out0 {
        code: args.code,
        name,
        category,
        valid,
    })
}
