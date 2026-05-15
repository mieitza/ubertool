use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Header value to decode (with or without the leading "Basic ").
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    user: String,
    pass: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let raw = input.as_str()?.trim();
    let b64 = raw.strip_prefix("Basic ").unwrap_or(raw);
    let decoded = STANDARD.decode(b64).map_err(|e| {
        CliError::new(
            ErrorCode::InvalidBase64,
            format!("base64 decode failed: {e}"),
        )
        .with_input(serde_json::json!(b64))
        .with_hint("Basic auth header values are standard base64 of `user:pass`")
    })?;
    let s = String::from_utf8(decoded).map_err(|_| {
        CliError::new(
            ErrorCode::InvalidUtf8,
            "decoded credentials are not valid UTF-8",
        )
    })?;
    let (user, pass) = s.split_once(':').ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidBase64,
            "decoded credentials have no `:` separator",
        )
        .with_hint("expected format is `user:pass`")
    })?;
    out.emit_value(&DecodeOutput {
        user: user.to_string(),
        pass: pass.to_string(),
    })
}
