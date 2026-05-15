//! YAML conversion verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

pub mod to_json;
pub mod to_toml;

#[derive(Debug, Args)]
pub struct YamlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert YAML to JSON.
    #[command(name = "to-json", long_about = "Convert YAML to JSON.\n\nExamples:\n  ubertool yaml to-json 'name: alice'\n  ubertool yaml to-json --in ./config.yaml --pretty\n\nExit codes specific to this command:\n  3   invalid YAML (invalid_yaml)")]
    ToJson(ToJsonArgs),
    /// Convert YAML to TOML.
    #[command(name = "to-toml", long_about = "Convert YAML to TOML via JSON value interchange.\n\nExamples:\n  ubertool yaml to-toml 'k: v'\n\nExit codes specific to this command:\n  3   invalid YAML, or value type not representable in TOML")]
    ToToml(RunArgs),
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
    /// Emit indented JSON instead of compact.
    #[arg(long, default_value_t = false)]
    pub pretty: bool,
}

pub fn dispatch(args: YamlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::ToToml(a) => to_toml::run(a, out),
    }
}

pub(super) fn parse_yaml(s: &str) -> Result<serde_yaml::Value, CliError> {
    serde_yaml::from_str(s).map_err(|e| {
        CliError::new(ErrorCode::InvalidYaml, format!("invalid YAML: {e}"))
            .with_hint("ensure the input is well-formed YAML")
    })
}
