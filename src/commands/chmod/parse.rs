use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::ParseArgs;

#[derive(Serialize)]
struct ClassFlags {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Serialize)]
struct Out0 {
    octal: String,
    symbolic: String,
    owner: ClassFlags,
    group: ClassFlags,
    other: ClassFlags,
}

pub fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    if args.input.len() != 3 {
        return Err(CliError::new(
            ErrorCode::InvalidChmod,
            format!("expected 3-digit octal, got {}", args.input),
        ));
    }
    let mut digits = [0u32; 3];
    for (i, c) in args.input.chars().enumerate() {
        let d = c.to_digit(8).ok_or_else(|| {
            CliError::new(
                ErrorCode::InvalidChmod,
                format!("invalid octal digit `{c}`"),
            )
        })?;
        if d > 7 {
            return Err(CliError::new(
                ErrorCode::InvalidChmod,
                format!("digit `{c}` is out of range 0-7"),
            ));
        }
        digits[i] = d;
    }
    let symbolic = format!(
        "{}{}{}",
        triplet(digits[0]),
        triplet(digits[1]),
        triplet(digits[2])
    );
    out.emit_value(&Out0 {
        octal: args.input.clone(),
        symbolic,
        owner: bits_to_flags(digits[0]),
        group: bits_to_flags(digits[1]),
        other: bits_to_flags(digits[2]),
    })
}

fn triplet(d: u32) -> String {
    let r = if d & 4 != 0 { 'r' } else { '-' };
    let w = if d & 2 != 0 { 'w' } else { '-' };
    let x = if d & 1 != 0 { 'x' } else { '-' };
    format!("{r}{w}{x}")
}

fn bits_to_flags(d: u32) -> ClassFlags {
    ClassFlags {
        read: d & 4 != 0,
        write: d & 2 != 0,
        execute: d & 1 != 0,
    }
}
