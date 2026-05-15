use std::path::PathBuf;

use bcrypt::BcryptError;
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Password to check (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read password from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Bcrypt hash to verify against (required).
    #[arg(long)]
    pub hash: String,
}

#[derive(Serialize)]
struct VerifyOutput {
    verified: bool,
}

pub fn run(args: VerifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let password = input.as_str()?;
    match bcrypt::verify(password, &args.hash) {
        Ok(true) => out.emit_value(&VerifyOutput { verified: true }),
        Ok(false) => Err(CliError::new(
            ErrorCode::SignatureMismatch,
            "bcrypt verify: password does not match the supplied hash",
        )
        .with_hint("confirm the password is correct")),
        Err(BcryptError::InvalidHash(msg)) => Err(CliError::new(
            ErrorCode::InvalidBcrypt,
            format!("malformed bcrypt hash: {msg}"),
        )
        .with_input(serde_json::json!(args.hash))
        .with_hint("bcrypt hashes start with $2a$, $2b$, $2x$, or $2y$ followed by cost and salt")),
        Err(BcryptError::InvalidPrefix(msg)) => Err(CliError::new(
            ErrorCode::InvalidBcrypt,
            format!("malformed bcrypt hash: {msg}"),
        )
        .with_input(serde_json::json!(args.hash))
        .with_hint("bcrypt hashes start with $2a$, $2b$, $2x$, or $2y$ followed by cost and salt")),
        Err(BcryptError::InvalidCost(msg)) => Err(CliError::new(
            ErrorCode::InvalidBcrypt,
            format!("invalid bcrypt cost: {msg}"),
        )),
        Err(e) => Err(CliError::new(
            ErrorCode::Internal,
            format!("bcrypt verify failed: {e}"),
        )),
    }
}
