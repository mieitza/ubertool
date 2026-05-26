//! Bcrypt password hashing and verification.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod hash;
pub mod verify;

#[derive(Debug, Args)]
pub struct BcryptArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Hash a password using bcrypt.
    #[command(
        long_about = "Hash a password using bcrypt.\n\nExamples:\n  ubertool bcrypt hash \"my-password\"\n  ubertool bcrypt hash \"my-password\" --cost 12 --json\n  echo -n \"my-password\" | ubertool bcrypt hash\n  ubertool bcrypt hash --in ./password.txt"
    )]
    Hash(hash::HashArgs),
    /// Verify a password against a bcrypt hash.
    #[command(
        long_about = "Verify a password against a bcrypt hash.\n\nExamples:\n  ubertool bcrypt verify \"my-password\" --hash \"$2b$12$...\"\n  ubertool bcrypt verify \"my-password\" --hash \"$2b$12$...\" --json\n  ubertool bcrypt verify --in ./password.txt --hash \"$2b$12$...\"\n\nExit codes specific to this command:\n  3   invalid bcrypt hash format\n  5   password does not match hash (signature_mismatch)"
    )]
    Verify(verify::VerifyArgs),
}

pub fn dispatch(args: BcryptArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Hash(a) => hash::run(a, out),
        Verb::Verify(a) => verify::run(a, out),
    }
}
