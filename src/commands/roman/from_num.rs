use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct FromNumArgs {
    /// Integer in range 1..=3999.
    pub input: u32,
}

#[derive(Serialize)]
struct Out0 {
    roman: String,
}

pub fn run(args: FromNumArgs, out: &Out) -> Result<(), CliError> {
    let r = to_roman(args.input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidRoman,
            format!("value {} is out of range (1..=3999)", args.input),
        )
        .with_input(serde_json::json!(args.input))
    })?;
    out.emit_value(&Out0 { roman: r })
}

pub(super) fn to_roman(n: u32) -> Option<String> {
    if n == 0 || n > 3999 {
        return None;
    }
    const PAIRS: &[(u32, &str)] = &[
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    let mut n = n;
    for &(v, sym) in PAIRS {
        while n >= v {
            out.push_str(sym);
            n -= v;
        }
    }
    Some(out)
}
