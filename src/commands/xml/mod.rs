//! XML conversion: to-json (compact) and format (pretty-print).

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod format;
pub mod to_json;

#[derive(Debug, Args)]
pub struct XmlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert XML to JSON. Attributes become `@attr` keys; text content becomes a string or `#text`.
    #[command(
        name = "to-json",
        long_about = "Convert XML to JSON.\n\nConvention: attributes become `@attr` keys, text content of leaf elements becomes a string value, mixed content places text under `#text`, repeated children collapse to arrays.\n\nExamples:\n  ubertool xml to-json '<root><name>alice</name></root>'\n  ubertool xml to-json --in ./data.xml --json\n\nExit codes specific to this command:\n  3   invalid XML (invalid_xml)"
    )]
    ToJson(RunArgs),
    /// Pretty-print XML with indentation.
    #[command(
        long_about = "Pretty-print XML with two-space indentation.\n\nExamples:\n  ubertool xml format '<r><a>1</a></r>'\n  ubertool xml format --in ./data.xml --json\n\nExit codes specific to this command:\n  3   invalid XML (invalid_xml)"
    )]
    Format(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

pub fn dispatch(args: XmlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::Format(a) => format::run(a, out),
    }
}
