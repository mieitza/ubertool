use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::EncodeArgs;

#[derive(Serialize)]
struct EncodeOutput<'a> {
    encoded: &'a str,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(args.input.as_deref(), args.in_path.as_deref(), is_stdin_tty())?;
    let encoded = STANDARD.encode(input.as_bytes());

    if let Some(path) = &args.out_path {
        std::fs::write(path, encoded.as_bytes()).map_err(CliError::from)?;
        return Ok(());
    }
    out.emit_value(&EncodeOutput { encoded: &encoded })
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn standard_alphabet_with_padding() {
        assert_eq!(STANDARD.encode(b"hello"), "aGVsbG8=");
        assert_eq!(STANDARD.encode(b""), "");
        assert_eq!(STANDARD.encode(b"\x00\x01\x02"), "AAEC");
    }
}
