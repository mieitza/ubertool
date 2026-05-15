use std::path::PathBuf;

use clap::Args;
use percent_encoding::percent_decode_str;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Literal input to decode (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    decoded: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let decoded = percent_decode_str(s).decode_utf8().map_err(|_| {
        CliError::new(
            ErrorCode::InvalidUtf8,
            "decoded bytes are not valid UTF-8",
        )
        .with_input(serde_json::json!(s))
        .with_hint("the input contains percent-escapes that produce non-UTF-8 bytes")
    })?;
    out.emit_value(&DecodeOutput {
        decoded: decoded.into_owned(),
    })
}
