//! Unix file mode conversion (symbolic ↔ octal).

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod calc;
pub mod parse;

#[derive(Debug, Args)]
pub struct ChmodArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert symbolic mode (rwxr-xr-x) → octal (755).
    #[command(long_about = "Convert symbolic mode like `rwxr-xr-x` to octal `755`.\n\nExamples:\n  ubertool chmod calc rwxr-xr-x\n\nExit codes:\n  3   invalid symbolic input (invalid_chmod)")]
    Calc(CalcArgs),
    /// Convert octal mode (755) → symbolic + per-class breakdown.
    #[command(long_about = "Convert octal mode like `755` to symbolic `rwxr-xr-x` plus per-class (owner/group/other) read/write/execute flags.\n\nExamples:\n  ubertool chmod parse 755\n\nExit codes:\n  3   octal must be 3 digits each 0-7 (invalid_chmod)")]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct CalcArgs {
    pub input: String,
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

pub fn dispatch(args: ChmodArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Calc(a) => calc::run(a, out),
        Verb::Parse(a) => parse::run(a, out),
    }
}
