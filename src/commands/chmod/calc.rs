use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::CalcArgs;

#[derive(Serialize)]
struct Out0 {
    octal: String,
}

pub fn run(args: CalcArgs, out: &Out) -> Result<(), CliError> {
    let s = args.input.as_str();
    if s.len() != 9 {
        return Err(CliError::new(
            ErrorCode::InvalidChmod,
            format!("symbolic mode must be 9 chars, got {}", s.len()),
        )
        .with_input(serde_json::json!(args.input)));
    }
    let bytes = s.as_bytes();
    let owner = class_digit(&bytes[0..3])?;
    let group = class_digit(&bytes[3..6])?;
    let other = class_digit(&bytes[6..9])?;
    out.emit_value(&Out0 {
        octal: format!("{owner}{group}{other}"),
    })
}

fn class_digit(triplet: &[u8]) -> Result<u32, CliError> {
    let mut bits = 0u32;
    let expected: [(u8, u8); 3] = [(b'r', 4), (b'w', 2), (b'x', 1)];
    for (i, (sym, bit)) in expected.iter().enumerate() {
        match triplet[i] {
            c if c == *sym => bits |= u32::from(*bit),
            b'-' => {}
            other => {
                return Err(CliError::new(
                    ErrorCode::InvalidChmod,
                    format!(
                        "unexpected char `{}` at position {i} in triplet",
                        other as char
                    ),
                ));
            }
        }
    }
    Ok(bits)
}
