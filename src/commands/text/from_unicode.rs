use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    text: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let mut result = String::new();
    for token in s.split_whitespace() {
        let hex = token
            .strip_prefix("U+")
            .or_else(|| token.strip_prefix("u+"))
            .ok_or_else(|| {
                CliError::new(
                    ErrorCode::InvalidCodepoint,
                    format!("expected U+NNNN form, got '{token}'"),
                )
            })?;
        let n = u32::from_str_radix(hex, 16).map_err(|_| {
            CliError::new(
                ErrorCode::InvalidCodepoint,
                format!("hex parse failed for '{token}'"),
            )
        })?;
        let ch = char::from_u32(n).ok_or_else(|| {
            CliError::new(
                ErrorCode::InvalidCodepoint,
                format!("not a valid Unicode codepoint: U+{n:04X}"),
            )
        })?;
        result.push(ch);
    }
    out.emit_value(&Out0 { text: result })
}
