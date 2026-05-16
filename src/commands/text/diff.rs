use serde::Serialize;
use similar::TextDiff;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::DiffArgs;

#[derive(Serialize)]
struct Out0 {
    diff: String,
}

pub fn run(args: DiffArgs, out: &Out) -> Result<(), CliError> {
    let from = read_side(args.from.as_deref(), args.from_file.as_deref(), "from")?;
    let to = read_side(args.to.as_deref(), args.to_file.as_deref(), "to")?;
    let td = TextDiff::from_lines(&from, &to);
    let mut diff_text = String::new();
    for change in td.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        diff_text.push_str(sign);
        diff_text.push_str(change.value());
        if !diff_text.ends_with('\n') {
            diff_text.push('\n');
        }
    }
    out.emit_value(&Out0 { diff: diff_text })
}

fn read_side(
    positional: Option<&str>,
    file: Option<&std::path::Path>,
    side: &str,
) -> Result<String, CliError> {
    if let Some(s) = positional {
        return Ok(s.to_string());
    }
    if let Some(p) = file {
        return std::fs::read_to_string(p).map_err(CliError::from);
    }
    Err(CliError::new(
        ErrorCode::UsageError,
        format!("missing {side} input — pass positional or --{side}-file"),
    ))
}
