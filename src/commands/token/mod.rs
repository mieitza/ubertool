//! Random token generator — hex, base64, or alphanumeric.

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::{Args, Subcommand, ValueEnum};
use rand::Rng;
use rand::RngCore;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::hex;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct TokenArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random token.
    #[command(long_about = "Generate a random token from a CSPRNG.\n\nFor hex and base64, --length is the number of bytes of entropy. For alphanumeric, --length is the number of output characters.\n\nExamples:\n  ubertool token new                              # 32 bytes → 64 hex chars\n  ubertool token new --length 16 --format hex     # 32 hex chars\n  ubertool token new --format base64\n  ubertool token new --format alphanumeric --length 24 --json")]
    New(NewArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Hex,
    Base64,
    Alphanumeric,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Length: bytes of entropy (for hex/base64), or character count (for alphanumeric).
    #[arg(long, default_value_t = 32)]
    pub length: usize,
    /// Output format.
    #[arg(long, value_enum, default_value = "hex")]
    pub format: Format,
}

#[derive(Serialize)]
struct TokenOutput {
    token: String,
}

const ALPHANUMERIC: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn dispatch(args: TokenArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New(a) => run(a, out),
    }
}

fn run(args: NewArgs, out: &Out) -> Result<(), CliError> {
    let mut rng = rand::thread_rng();
    let token = match args.format {
        Format::Hex => {
            let mut buf = vec![0u8; args.length];
            rng.fill_bytes(&mut buf);
            hex::encode(&buf)
        }
        Format::Base64 => {
            let mut buf = vec![0u8; args.length];
            rng.fill_bytes(&mut buf);
            STANDARD.encode(&buf)
        }
        Format::Alphanumeric => (0..args.length)
            .map(|_| ALPHANUMERIC[rng.gen_range(0..ALPHANUMERIC.len())] as char)
            .collect(),
    };
    out.emit_value(&TokenOutput { token })
}
