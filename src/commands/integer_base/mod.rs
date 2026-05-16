//! Integer base conversion (bases 2-36).

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct IntegerBaseArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert an integer between bases (2-36).
    #[command(long_about = "Convert an integer between bases (2-36).\n\nExamples:\n  ubertool integer-base convert 255 --from 10 --to 16\n  ubertool integer-base convert ff --from 16 --to 2\n\nExit codes specific to this command:\n  2   base out of range (must be 2-36)\n  3   value contains digits invalid for the source base (invalid_integer_base)")]
    Convert(ConvertArgs),
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// Value to convert (digits only, no sign).
    pub input: String,
    /// Source base.
    #[arg(long)]
    pub from: u32,
    /// Target base.
    #[arg(long)]
    pub to: u32,
}

#[derive(Serialize)]
struct Out0 {
    result: String,
}

pub fn dispatch(args: IntegerBaseArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    if !(2..=36).contains(&args.from) || !(2..=36).contains(&args.to) {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "bases must be in range 2..=36",
        ));
    }
    let n = u128::from_str_radix(&args.input, args.from).map_err(|e| {
        CliError::new(
            ErrorCode::InvalidIntegerBase,
            format!("invalid digits for base {}: {e}", args.from),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("verify each digit is valid for the source base")
    })?;
    let result = to_radix(n, args.to);
    out.emit_value(&Out0 { result })
}

fn to_radix(mut n: u128, base: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let chars: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut buf = Vec::new();
    let b = base as u128;
    while n > 0 {
        buf.push(chars[(n % b) as usize]);
        n /= b;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_radix_known() {
        assert_eq!(to_radix(255, 16), "ff");
        assert_eq!(to_radix(10, 2), "1010");
        assert_eq!(to_radix(0, 10), "0");
        assert_eq!(to_radix(35, 36), "z");
    }
}
