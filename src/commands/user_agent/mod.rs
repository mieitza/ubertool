//! User-agent string parser (woothee).

use clap::{Args, Subcommand};
use serde::Serialize;
use woothee::parser::Parser;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UserAgentArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse a user-agent string into structured fields.
    #[command(
        long_about = "Parse a user-agent string (browser, OS, version, etc.) using the woothee parser.\n\nExamples:\n  ubertool user-agent parse 'Mozilla/5.0 ...'\n  ubertool user-agent parse '...' --json"
    )]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    browser: String,
    browser_version: String,
    os: String,
    os_version: String,
    category: String,
    vendor: String,
}

pub fn dispatch(args: UserAgentArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => run(a, out),
    }
}

fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let parser = Parser::new();
    let result = parser.parse(&args.input);
    match result {
        Some(r) => out.emit_value(&Out0 {
            browser: r.name.to_string(),
            browser_version: r.version.to_string(),
            os: r.os.to_string(),
            os_version: r.os_version.to_string(),
            category: r.category.to_string(),
            vendor: r.vendor.to_string(),
        }),
        None => out.emit_value(&Out0 {
            browser: "UNKNOWN".into(),
            browser_version: "UNKNOWN".into(),
            os: "UNKNOWN".into(),
            os_version: "UNKNOWN".into(),
            category: "UNKNOWN".into(),
            vendor: "UNKNOWN".into(),
        }),
    }
}
