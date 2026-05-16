use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct ToNumArgs {
    /// Roman numeral input.
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    number: u32,
}

pub fn run(args: ToNumArgs, out: &Out) -> Result<(), CliError> {
    let n = parse_roman(&args.input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidRoman,
            format!("invalid Roman numeral: {}", args.input),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("valid characters are I, V, X, L, C, D, M (uppercase)")
    })?;
    out.emit_value(&Out0 { number: n })
}

fn parse_roman(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let val = |c: char| -> Option<u32> {
        match c {
            'I' => Some(1), 'V' => Some(5), 'X' => Some(10),
            'L' => Some(50), 'C' => Some(100), 'D' => Some(500), 'M' => Some(1000),
            _ => None,
        }
    };
    let vals: Vec<u32> = s.chars().map(val).collect::<Option<Vec<_>>>()?;
    let mut total: i64 = 0;
    let vals_i64: Vec<i64> = vals.iter().map(|&x| x as i64).collect();
    for i in 0..vals_i64.len() {
        let v = vals_i64[i];
        if i + 1 < vals_i64.len() && v < vals_i64[i + 1] {
            total -= v;
        } else {
            total += v;
        }
    }
    if total <= 0 || total > 3999 {
        return None;
    }
    let total = total as u32;
    // Round-trip validation: reject non-canonical input like "IIII".
    let canonical = crate::commands::roman::from_num::to_roman(total)?;
    if canonical != s {
        return None;
    }
    Some(total)
}
