use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::DecodeArgs;

#[derive(Serialize)]
struct DecodeOutput<'a> {
    decoded: &'a str,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let trimmed = input.as_str()?.trim();
    let bytes = STANDARD.decode(trimmed).map_err(|e| {
        let echo: String = trimmed.chars().take(64).collect();
        CliError::new(ErrorCode::InvalidBase64, format!("invalid base64: {e}"))
            .with_input(serde_json::Value::String(echo))
            .with_hint("ensure the input contains only base64 characters and padding")
    })?;

    if let Some(path) = &args.out_path {
        std::fs::write(path, &bytes).map_err(CliError::from)?;
        return Ok(());
    }

    if let Ok(s) = std::str::from_utf8(&bytes) {
        out.emit_value(&DecodeOutput { decoded: s })
    } else {
        out.emit_binary(&bytes, None)
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn decodes_hello() {
        assert_eq!(STANDARD.decode("aGVsbG8=").unwrap(), b"hello");
    }

    #[test]
    fn rejects_invalid() {
        assert!(STANDARD.decode("!!!").is_err());
    }
}
