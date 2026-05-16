use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    chars: usize,
    words: usize,
    lines: usize,
    bytes: usize,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let bytes = s.len();
    let chars = s.chars().count();
    let words = s.split_whitespace().count();
    let lines = if s.is_empty() { 0 } else { s.lines().count() };
    out.emit_value(&Out0 { chars, words, lines, bytes })
}
