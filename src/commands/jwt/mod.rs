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
        long_about = "Decode a JWT without verifying its signature — useful for inspection.\n\nExamples:\n  ubertool jwt decode \"eyJhbGc...\"\n  ubertool jwt decode \"eyJhbGc...\" --json\n  ubertool jwt decode --in ./token.txt\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)"
    )]
    Decode(decode::DecodeArgs),
    /// Verify a JWT signature (HS256/384/512 or RS256/384/512 or ES256/384).
    #[command(
        long_about = "Verify a JWT signature.\n\nSymmetric (HS*): requires --secret.\nAsymmetric (RS*/ES*): requires --key-file (path to a PEM public key).\n\nExamples:\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\"\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\" --algo hs512 --json\n  ubertool jwt verify \"eyJhbGc...\" --algo rs256 --key-file /path/to/pub.pem\n  ubertool jwt verify \"eyJhbGc...\" --algo es256 --key-file /path/to/ec_pub.pem --json\n  ubertool jwt verify --in ./token.txt --secret \"my-secret\" --validate-exp\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)\n  5   signature mismatch / wrong key (signature_mismatch)\n  2   missing required flag for the chosen algorithm (usage_error)"
    )]
    Verify(verify::VerifyArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum JwtAlgo {
    Hs256,
    Hs384,
    Hs512,
    Rs256,
    Rs384,
    Rs512,
    Es256,
    Es384,
}

impl JwtAlgo {
    pub fn to_jsonwebtoken(self) -> jsonwebtoken::Algorithm {
        match self {
            JwtAlgo::Hs256 => jsonwebtoken::Algorithm::HS256,
            JwtAlgo::Hs384 => jsonwebtoken::Algorithm::HS384,
            JwtAlgo::Hs512 => jsonwebtoken::Algorithm::HS512,
            JwtAlgo::Rs256 => jsonwebtoken::Algorithm::RS256,
            JwtAlgo::Rs384 => jsonwebtoken::Algorithm::RS384,
            JwtAlgo::Rs512 => jsonwebtoken::Algorithm::RS512,
            JwtAlgo::Es256 => jsonwebtoken::Algorithm::ES256,
            JwtAlgo::Es384 => jsonwebtoken::Algorithm::ES384,
        }
    }

    /// Returns true for symmetric HMAC algorithms (HS*).
    pub fn is_symmetric(self) -> bool {
        matches!(self, JwtAlgo::Hs256 | JwtAlgo::Hs384 | JwtAlgo::Hs512)
    }
}

pub fn dispatch(args: JwtArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Decode(a) => decode::run(a, out),
        Verb::Verify(a) => verify::run(a, out),
    }
}
