//! Roman numeral conversion.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod from_num;
pub mod to_num;

#[derive(Debug, Args)]
pub struct RomanArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert Roman numeral → integer.
    #[command(
        name = "to-num",
        long_about = "Convert a Roman numeral to an integer (1-3999).\n\nExamples:\n  ubertool roman to-num MCMXCIX\n  ubertool roman to-num IV --json\n\nExit codes:\n  3   invalid Roman numeral (invalid_roman)"
    )]
    ToNum(to_num::ToNumArgs),
    /// Convert integer → Roman numeral.
    #[command(
        name = "from-num",
        long_about = "Convert an integer (1-3999) to its Roman numeral form.\n\nExamples:\n  ubertool roman from-num 1999\n  ubertool roman from-num 42 --json\n\nExit codes:\n  3   integer out of range (must be 1..=3999)"
    )]
    FromNum(from_num::FromNumArgs),
}

pub fn dispatch(args: RomanArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToNum(a) => to_num::run(a, out),
        Verb::FromNum(a) => from_num::run(a, out),
    }
}
