//! Random port generator.

use clap::{Args, Subcommand};
use rand::Rng;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct PortArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random port number in a range.
    #[command(long_about = "Generate a random TCP/UDP port number.\n\nDefault range is the IANA ephemeral range 49152-65535. Override with --min/--max.\n\nExamples:\n  ubertool port random\n  ubertool port random --min 8000 --max 8099\n\nExit codes:\n  2   --min greater than --max")]
    Random(RandomArgs),
}

#[derive(Debug, Args)]
pub struct RandomArgs {
    #[arg(long, default_value_t = 49152)]
    pub min: u16,
    #[arg(long, default_value_t = 65535)]
    pub max: u16,
}

#[derive(Serialize)]
struct Out0 {
    port: u16,
}

pub fn dispatch(args: PortArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Random(a) => run(a, out),
    }
}

fn run(args: RandomArgs, out: &Out) -> Result<(), CliError> {
    if args.min > args.max {
        return Err(CliError::new(
            ErrorCode::UsageError,
            format!("--min ({}) must be \u{2264} --max ({})", args.min, args.max),
        ));
    }
    let port = rand::thread_rng().gen_range(args.min..=args.max);
    out.emit_value(&Out0 { port })
}
