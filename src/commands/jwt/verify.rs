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
    /// Shared HMAC secret (for HS* algorithms).
    #[arg(long)]
    pub secret: Option<String>,
    /// Path to a PEM public key (for RS*/ES* algorithms).
    #[arg(long = "key-file")]
    pub key_file: Option<PathBuf>,
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

    let key = if args.algo.is_symmetric() {
        // HS* — require --secret
        let secret = args.secret.as_deref().ok_or_else(|| {
            CliError::new(ErrorCode::UsageError, "HS* algorithms require --secret")
                .with_hint("provide --secret <shared-secret> for HS256/HS384/HS512")
        })?;
        DecodingKey::from_secret(secret.as_bytes())
    } else {
        // RS* / ES* — require --key-file
        let key_path = args.key_file.as_deref().ok_or_else(|| {
            CliError::new(
                ErrorCode::UsageError,
                "RS*/ES* algorithms require --key-file",
            )
            .with_hint("provide --key-file <path-to-pem-public-key> for RS*/ES* algorithms")
        })?;
        let pem_bytes = std::fs::read(key_path).map_err(|e| {
            let code = match e.kind() {
                std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
                std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
                _ => ErrorCode::IoError,
            };
            CliError::new(code, format!("could not read key file: {e}"))
                .with_input(serde_json::json!({"key_file": key_path.display().to_string()}))
        })?;

        match args.algo {
            JwtAlgo::Rs256 | JwtAlgo::Rs384 | JwtAlgo::Rs512 => {
                DecodingKey::from_rsa_pem(&pem_bytes).map_err(|e| {
                    CliError::new(
                        ErrorCode::InvalidJwt,
                        format!("failed to parse RSA PEM public key: {e}"),
                    )
                    .with_input(serde_json::json!({"key_file": key_path.display().to_string()}))
                    .with_hint("ensure the file is a PKCS#8 or PKCS#1 RSA public key in PEM format")
                })?
            }
            JwtAlgo::Es256 | JwtAlgo::Es384 => {
                DecodingKey::from_ec_pem(&pem_bytes).map_err(|e| {
                    CliError::new(
                        ErrorCode::InvalidJwt,
                        format!("failed to parse EC PEM public key: {e}"),
                    )
                    .with_input(serde_json::json!({"key_file": key_path.display().to_string()}))
                    .with_hint("ensure the file is an EC public key in PEM format")
                })?
            }
            // Symmetric arms are handled above; this branch is unreachable.
            _ => unreachable!("symmetric algo reached asymmetric key-file branch"),
        }
    };

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
                    "JWT signature does not verify with the provided key",
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
            // Echo the JWT (it's not secret). Redact --secret; --key-file path is safe to echo.
            let input_echo = if let Some(kf) = &args.key_file {
                serde_json::json!({"token": token, "key_file": kf.display().to_string()})
            } else {
                serde_json::json!({"token": token, "secret": "<redacted>"})
            };
            Err(CliError::new(code, msg).with_input(input_echo))
        }
    }
}
