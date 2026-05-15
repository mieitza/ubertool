use std::path::PathBuf;

use clap::Args;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::JwtAlgo;

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// JWT to verify (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Shared HMAC secret (required for HS* algorithms).
    #[arg(long)]
    pub secret: String,
    /// Signing algorithm.
    #[arg(long, value_enum, default_value = "hs256")]
    pub algo: JwtAlgo,
    /// Validate `exp` claim against the current time (off by default).
    #[arg(long, default_value_t = false)]
    pub validate_exp: bool,
}

#[derive(Serialize)]
struct VerifyOutput {
    verified: bool,
    claims: serde_json::Value,
}

pub fn run(args: VerifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let token = input.as_str()?.trim();

    let key = DecodingKey::from_secret(args.secret.as_bytes());
    let mut validation = Validation::new(args.algo.to_jsonwebtoken());
    validation.validate_exp = args.validate_exp;
    // We never enforce aud/iss/nbf at the CLI level — the user can post-process claims.
    validation.required_spec_claims.clear();

    match decode::<serde_json::Value>(token, &key, &validation) {
        Ok(data) => out.emit_value(&VerifyOutput {
            verified: true,
            claims: data.claims,
        }),
        Err(e) => {
            use jsonwebtoken::errors::ErrorKind::*;
            let (code, msg) = match e.kind() {
                InvalidSignature => (
                    ErrorCode::SignatureMismatch,
                    "JWT signature does not verify with the provided secret",
                ),
                ExpiredSignature => (
                    ErrorCode::SignatureMismatch,
                    "JWT has expired (exp claim < now)",
                ),
                InvalidAlgorithm | InvalidAlgorithmName => (
                    ErrorCode::AlgoNotSupported,
                    "JWT algorithm is not supported by this build",
                ),
                _ => (ErrorCode::InvalidJwt, "JWT is malformed"),
            };
            // Echo the JWT (it's not secret) but NEVER the secret.
            Err(CliError::new(code, msg)
                .with_input(serde_json::json!({"token": token, "secret": "<redacted>"}))
                .with_hint("if the JWT uses RS*/ES*, this build does not support asymmetric keys"))
        }
    }
}
