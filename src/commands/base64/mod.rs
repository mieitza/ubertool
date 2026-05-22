//! Base64 encoding and decoding.

use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct Base64Args {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode input as base64 (standard alphabet, with padding).
    #[command(
        long_about = "Encode input as base64 (standard alphabet, with padding).\n\
                            \n\
                            Examples:\n  \
                            ubertool base64 encode \"hello\"\n  \
                            echo -n hello | ubertool base64 encode\n  \
                            ubertool base64 encode \"hello\" --json\n  \
                            printf 'alice\\nbob\\n' | ubertool base64 encode --batch"
    )]
    Encode(EncodeArgs),

    /// Decode base64 input (standard alphabet, with padding).
    #[command(
        long_about = "Decode base64 input (standard alphabet, with padding).\n\
                            \n\
                            Examples:\n  \
                            ubertool base64 decode aGVsbG8=\n  \
                            ubertool base64 decode aGVsbG8= --json\n  \
                            printf 'aGVsbG8=\\nd29ybGQ=\\n' | ubertool base64 decode --batch\n\
                            \n\
                            Exit codes specific to this command:\n  \
                            3   invalid base64 (input could not be decoded)"
    )]
    Decode(DecodeArgs),
}

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Literal input to encode (omit to read from --in or stdin).
    pub input: Option<String>,

    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,

    /// Write encoded output to file. If absent, output goes to stdout.
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,

    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Literal base64 input to decode (omit to read from --in or stdin).
    pub input: Option<String>,

    /// Read base64 input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,

    /// Write decoded bytes to file. If absent and bytes are valid UTF-8,
    /// emits as text; otherwise refuses to write binary to a TTY.
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,

    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

pub fn dispatch(args: Base64Args, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
