//! RFC4193 Unique Local Address generator.

use clap::{Args, Subcommand};
use rand::RngCore;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct Ipv6UlaArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random IPv6 ULA /48 prefix.
    #[command(
        long_about = "Generate a random RFC4193 Unique Local IPv6 Address prefix (fd00::/8 with random 40-bit Global ID).\n\nExamples:\n  ubertool ipv6-ula new\n  ubertool ipv6-ula new --json"
    )]
    New,
}

#[derive(Serialize)]
struct Out0 {
    prefix: String,
    global_id: String,
}

#[derive(Serialize)]
struct Out0Short {
    prefix: String,
}

pub fn dispatch(args: Ipv6UlaArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => run(out),
    }
}

fn run(out: &Out) -> Result<(), CliError> {
    let mut buf = [0u8; 5];
    rand::thread_rng().fill_bytes(&mut buf);
    let global_id = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3], buf[4]
    );
    let prefix = format!(
        "fd{:02x}:{:02x}{:02x}:{:02x}{:02x}::/48",
        buf[0], buf[1], buf[2], buf[3], buf[4]
    );
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { prefix, global_id })
    } else {
        out.emit_value(&Out0Short { prefix })
    }
}
