//! HTML entity encoding/decoding.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct HtmlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode HTML-special characters into entities.
    #[command(
        long_about = "Encode HTML-special characters into entities (<, >, &, \", ').\n\nExamples:\n  ubertool html encode \"<b>hi</b>\"\n  ubertool html encode \"<b>hi</b>\" --json\n  ubertool html encode --in ./snippet.html\n  printf '<b>\\n&\\n' | ubertool html encode --batch"
    )]
    Encode(encode::EncodeArgs),
    /// Decode HTML entities back to characters.
    #[command(
        long_about = "Decode HTML entities back to characters. Unknown entities pass through unchanged.\n\nExamples:\n  ubertool html decode \"&lt;b&gt;hi&lt;/b&gt;\"\n  ubertool html decode \"&amp;\" --json\n  ubertool html decode --in ./entities.txt\n  printf '&lt;b&gt;\\n&amp;\\n' | ubertool html decode --batch"
    )]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: HtmlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
