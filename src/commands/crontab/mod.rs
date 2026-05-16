//! Crontab expression utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod describe;
pub mod next;

#[derive(Debug, Args)]
pub struct CrontabArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Describe a crontab expression in plain English.
    #[command(long_about = "Describe a crontab expression in plain English.\n\nSupports the common Unix 5-field format: minute hour day-of-month month day-of-week.\n\nExamples:\n  ubertool crontab describe '* * * * *'\n  ubertool crontab describe '30 14 * * *'")]
    Describe(DescribeArgs),
    /// Emit the next N matching timestamps.
    #[command(long_about = "Emit the next N matching timestamps for a crontab expression (defaults to 5).\n\nExamples:\n  ubertool crontab next '0 * * * *'\n  ubertool crontab next '*/15 * * * *' --count 10 --json\n\nExit codes:\n  3   invalid cron expression (invalid_cron)")]
    Next(NextArgs),
}

#[derive(Debug, Args)]
pub struct DescribeArgs {
    pub input: String,
}

#[derive(Debug, Args)]
pub struct NextArgs {
    pub input: String,
    #[arg(long, default_value_t = 5)]
    pub count: usize,
}

pub fn dispatch(args: CrontabArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Describe(a) => describe::run(a, out),
        Verb::Next(a) => next::run(a, out),
    }
}
