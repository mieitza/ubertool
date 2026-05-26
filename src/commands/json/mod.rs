//! JSON conversion verbs: to-yaml, to-toml, minify, prettify.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod convert;
pub mod minify;
pub mod prettify;
pub mod to_toml;
pub mod to_yaml;

#[derive(Debug, Args)]
pub struct JsonArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert JSON to YAML.
    #[command(
        name = "to-yaml",
        long_about = "Convert JSON to YAML.\n\nExamples:\n  ubertool json to-yaml '{\"k\":\"v\"}'\n  ubertool json to-yaml --in ./data.json\n  ubertool json to-yaml --in ./data.json --json   # wraps result in {\"yaml\":\"...\"}"
    )]
    ToYaml(RunArgs),
    /// Convert JSON to TOML.
    #[command(
        name = "to-toml",
        long_about = "Convert JSON to TOML.\n\nTOML does not support null values; nulls fail with exit 3 (invalid_toml).\n\nExamples:\n  ubertool json to-toml '{\"a\":1}'\n  ubertool json to-toml --in ./data.json\n  ubertool json to-toml --in ./data.json --json\n\nExit codes specific to this command:\n  3   invalid JSON, or value type not representable in TOML"
    )]
    ToToml(RunArgs),
    /// Minify JSON (strip whitespace).
    #[command(
        long_about = "Minify JSON (strip insignificant whitespace).\n\nExamples:\n  ubertool json minify '{ \"a\": 1 }'\n  cat data.json | ubertool json minify\n  ubertool json minify --in ./data.json --json"
    )]
    Minify(RunArgs),
    /// Prettify JSON with indentation.
    #[command(
        long_about = "Prettify JSON with indentation.\n\nExamples:\n  ubertool json prettify '{\"a\":1}'\n  ubertool json prettify '{\"a\":1}' --indent 4\n  ubertool json prettify --in ./data.json --indent 2 --json"
    )]
    Prettify(PrettifyArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct PrettifyArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Indent width in spaces (default: 2).
    #[arg(long, default_value_t = 2)]
    pub indent: usize,
}

pub fn dispatch(args: JsonArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToYaml(a) => to_yaml::run(a, out),
        Verb::ToToml(a) => to_toml::run(a, out),
        Verb::Minify(a) => minify::run(a, out),
        Verb::Prettify(a) => prettify::run(a, out),
    }
}
