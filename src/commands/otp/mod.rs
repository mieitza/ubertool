//! Time-based One-Time Passwords (RFC 6238).

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod validate;

#[derive(Debug, Args)]
pub struct OtpArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate the current TOTP code from a base32 secret.
    #[command(long_about = "Generate the current TOTP code (RFC 6238) from a base32 secret.\n\nExamples:\n  ubertool otp generate --secret JBSWY3DPEHPK3PXP\n  ubertool otp generate --secret JBSWY3DPEHPK3PXP --period 60 --digits 8 --json")]
    Generate(generate::GenerateArgs),
    /// Validate a TOTP code against the current time window.
    #[command(long_about = "Validate a TOTP code against the current time window. Allows ±1 step skew by default.\n\nExamples:\n  ubertool otp validate 123456 --secret JBSWY3DPEHPK3PXP\n\nExit codes:\n  5   code does not match within the allowed window (signature_mismatch)")]
    Validate(validate::ValidateArgs),
}

pub fn dispatch(args: OtpArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => generate::run(a, out),
        Verb::Validate(a) => validate::run(a, out),
    }
}
