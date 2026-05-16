//! Percentage calculations.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod change;
pub mod of;
pub mod of_total;

#[derive(Debug, Args)]
pub struct PercentageArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Compute "<percent>% of <value>".
    #[command(
        long_about = "Compute `<percent>% of <value>`.\n\nExamples:\n  ubertool percentage of 20 50    # 10\n  ubertool percentage of 15 80 --json"
    )]
    Of(OfArgs),
    /// Compute percentage change from <from> to <to>.
    #[command(
        long_about = "Compute percentage change from <from> to <to>: `(to - from) / from * 100`.\n\nExamples:\n  ubertool percentage change 100 125   # 25\n\nExit codes:\n  3   from value is zero (cannot compute percentage change)"
    )]
    Change(ChangeArgs),
    /// Compute what percent <part> is of <total>.
    #[command(
        name = "of-total",
        long_about = "Compute what percent <part> is of <total>.\n\nExamples:\n  ubertool percentage of-total 20 50   # 40"
    )]
    OfTotal(OfTotalArgs),
}

#[derive(Debug, Args)]
pub struct OfArgs {
    pub percent: f64,
    pub value: f64,
}

#[derive(Debug, Args)]
pub struct ChangeArgs {
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Args)]
pub struct OfTotalArgs {
    pub part: f64,
    pub total: f64,
}

pub fn dispatch(args: PercentageArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Of(a) => of::run(a, out),
        Verb::Change(a) => change::run(a, out),
        Verb::OfTotal(a) => of_total::run(a, out),
    }
}
