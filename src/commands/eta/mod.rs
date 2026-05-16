//! ETA from progress info.

use chrono::Utc;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EtaArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Calculate ETA from progress (done/total/elapsed).
    #[command(long_about = "Calculate ETA from progress.\n\nGiven `--done` items completed out of `--total` in `--elapsed` seconds, compute remaining time and the absolute ETA timestamp (UTC, ISO 8601).\n\nExamples:\n  ubertool eta calc --done 50 --total 100 --elapsed 60\n\nExit codes:\n  3   --done is 0 (rate is unknown)")]
    Calc(CalcArgs),
}

#[derive(Debug, Args)]
pub struct CalcArgs {
    #[arg(long)]
    pub done: f64,
    #[arg(long)]
    pub total: f64,
    #[arg(long)]
    pub elapsed: f64,
}

#[derive(Serialize)]
struct Out0 {
    done: f64,
    total: f64,
    elapsed_seconds: f64,
    rate_per_second: f64,
    remaining_seconds: f64,
    eta_iso: String,
}

pub fn dispatch(args: EtaArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Calc(a) => run(a, out),
    }
}

fn run(args: CalcArgs, out: &Out) -> Result<(), CliError> {
    if args.done <= 0.0 {
        return Err(CliError::new(
            ErrorCode::InvalidMath,
            "cannot estimate rate when --done is 0",
        ));
    }
    let rate = args.done / args.elapsed;
    let remaining = if args.done >= args.total {
        0.0
    } else {
        (args.total - args.done) / rate
    };
    let eta = Utc::now() + chrono::Duration::milliseconds((remaining * 1000.0) as i64);
    out.emit_value(&Out0 {
        done: args.done,
        total: args.total,
        elapsed_seconds: args.elapsed,
        rate_per_second: rate,
        remaining_seconds: remaining,
        eta_iso: eta.to_rfc3339(),
    })
}
