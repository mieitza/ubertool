//! TOML conversion verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

pub mod to_json;
pub mod to_yaml;

#[derive(Debug, Args)]
pub struct TomlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert TOML to JSON.
    #[command(
        name = "to-json",
        long_about = "Convert TOML to JSON.\n\nExamples:\n  ubertool toml to-json 'k = \"v\"'\n  ubertool toml to-json --in ./Cargo.toml --pretty\n\nExit codes specific to this command:\n  3   invalid TOML (invalid_toml)"
    )]
    ToJson(ToJsonArgs),
    /// Convert TOML to YAML.
    #[command(
        name = "to-yaml",
        long_about = "Convert TOML to YAML.\n\nExamples:\n  ubertool toml to-yaml 'k = \"v\"'"
    )]
    ToYaml(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ToJsonArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub pretty: bool,
}

pub fn dispatch(args: TomlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::ToYaml(a) => to_yaml::run(a, out),
    }
}

pub(super) fn parse_toml(s: &str) -> Result<toml::Value, CliError> {
    s.parse::<toml::Value>().map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("invalid TOML: {e}"))
            .with_hint("ensure the input is well-formed TOML")
    })
}
