//! JSON Web Token decoding and verification.

use clap::{Args, Subcommand, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod verify;

#[derive(Debug, Args)]
pub struct JwtArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Decode a JWT without verifying its signature.
    #[command(
        long_about = "Decode a JWT without verifying its signature — useful for inspection.\n\nExamples:\n  ubertool jwt decode \"eyJhbGc...\"\n  ubertool jwt decode \"eyJhbGc...\" --json\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)"
    )]
    Decode(decode::DecodeArgs),
    /// Verify a JWT signature (HS256/HS384/HS512 only in this build).
    #[command(
        long_about = "Verify a JWT signature against a shared secret. HS256, HS384, HS512 only.\n\nExamples:\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\"\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\" --algo hs512 --json\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)\n  5   signature mismatch / wrong secret (signature_mismatch)\n  6   asymmetric algorithm not supported in this build (algo_not_supported)"
    )]
    Verify(verify::VerifyArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum JwtAlgo {
    Hs256,
    Hs384,
    Hs512,
}

impl JwtAlgo {
    pub fn to_jsonwebtoken(self) -> jsonwebtoken::Algorithm {
        match self {
            JwtAlgo::Hs256 => jsonwebtoken::Algorithm::HS256,
            JwtAlgo::Hs384 => jsonwebtoken::Algorithm::HS384,
            JwtAlgo::Hs512 => jsonwebtoken::Algorithm::HS512,
        }
    }
}

pub fn dispatch(args: JwtArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Decode(a) => decode::run(a, out),
        Verb::Verify(a) => verify::run(a, out),
    }
}
