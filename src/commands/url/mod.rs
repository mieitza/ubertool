//! URL percent-encoding and decoding.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct UrlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Percent-encode a string for use in URLs.
    #[command(long_about = "Percent-encode a string for use in URLs (non-ASCII + reserved chars become %XX).\n\nExamples:\n  ubertool url encode \"hello world\"\n  ubertool url encode \"a+b=c\" --json")]
    Encode(encode::EncodeArgs),
    /// Decode a percent-encoded string.
    #[command(long_about = "Decode a percent-encoded string.\n\nExamples:\n  ubertool url decode \"hello%20world\"\n  ubertool url decode \"a%2Bb%3Dc\" --json\n\nExit codes specific to this command:\n  3   decoded bytes are not valid UTF-8")]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: UrlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
