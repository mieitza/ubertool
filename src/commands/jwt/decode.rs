use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// JWT to decode (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    header: serde_json::Value,
    claims: serde_json::Value,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let token = input.as_str()?.trim();

    // Manual split-and-base64-decode avoids invoking the signature validator.
    let mut parts = token.split('.');
    let header_b64 = parts.next().ok_or_else(|| jwt_invalid("missing header"))?;
    let claims_b64 = parts.next().ok_or_else(|| jwt_invalid("missing claims"))?;
    let _sig = parts
        .next()
        .ok_or_else(|| jwt_invalid("missing signature"))?;
    if parts.next().is_some() {
        return Err(jwt_invalid("too many segments"));
    }

    let header = decode_segment(header_b64).map_err(|_| jwt_invalid("invalid header encoding"))?;
    let claims = decode_segment(claims_b64).map_err(|_| jwt_invalid("invalid claims encoding"))?;

    out.emit_value(&DecodeOutput { header, claims })
}

fn decode_segment(s: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let bytes = URL_SAFE_NO_PAD.decode(s)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn jwt_invalid(msg: &str) -> CliError {
    CliError::new(ErrorCode::InvalidJwt, format!("malformed JWT: {msg}"))
        .with_hint("JWT must be three URL-safe base64 segments joined by `.`")
}
