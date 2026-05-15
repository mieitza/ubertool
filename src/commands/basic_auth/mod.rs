//! HTTP Basic authentication: encode user/pass to a header value, decode back.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct BasicAuthArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode a user/pass pair as an HTTP Basic Authorization header value.
    #[command(
        long_about = "Encode user/password as an HTTP Basic Authorization header value.\n\nExamples:\n  ubertool basic-auth encode --user alice --pass hunter2\n  ubertool basic-auth encode --user alice --pass hunter2 --json"
    )]
    Encode(encode::EncodeArgs),
    /// Decode an HTTP Basic Authorization header value into user/pass.
    #[command(
        long_about = "Decode an HTTP Basic Authorization header value into user/pass. The leading \"Basic \" prefix is optional.\n\nExamples:\n  ubertool basic-auth decode \"Basic dXNlcjpwYXNz\"\n  ubertool basic-auth decode dXNlcjpwYXNz --json\n\nExit codes specific to this command:\n  3   invalid base64, or decoded value has no `:` separator"
    )]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: BasicAuthArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
