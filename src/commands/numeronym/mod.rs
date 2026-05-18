//! Numeronym (i18n-style contraction) generator.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct NumeronymArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a numeronym (e.g., 'internationalization' -> 'i18n').
    #[command(long_about = "Generate a numeronym: first char + count of middle chars + last char.\n\nWords \u{2264} 2 chars are passed through unchanged.\n\nExamples:\n  ubertool numeronym generate internationalization   # i18n\n  ubertool numeronym generate kubernetes              # k8s\n  ubertool numeronym generate accessibility           # a11y")]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    numeronym: String,
}

pub fn dispatch(args: NumeronymArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let chars: Vec<char> = args.input.chars().collect();
    let result = if chars.len() <= 2 {
        args.input.clone()
    } else {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        let middle = chars.len() - 2;
        format!("{first}{middle}{last}")
    };
    out.emit_value(&Out0 { numeronym: result })
}
