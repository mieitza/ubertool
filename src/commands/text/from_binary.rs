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
    let mut bytes = Vec::new();
    for token in s.split_whitespace() {
        if token.len() != 8 || !token.chars().all(|c| c == '0' || c == '1') {
            return Err(CliError::new(
                ErrorCode::InvalidBinary,
                format!("expected 8-bit binary token, got '{token}'"),
            )
            .with_input(serde_json::json!(token)));
        }
        bytes.push(u8::from_str_radix(token, 2).unwrap());
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| CliError::new(ErrorCode::InvalidUtf8, "decoded bytes are not valid UTF-8"))?;
    out.emit_value(&Out0 { text })
}
