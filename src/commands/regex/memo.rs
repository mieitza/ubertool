use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MEMO: &str = include_str!("../../data/regex_memo.md");

#[derive(Serialize)]
struct Out0 {
    memo: String,
}

pub fn run(out: &Out) -> Result<(), CliError> {
    out.emit_value(&Out0 { memo: MEMO.to_string() })
}
