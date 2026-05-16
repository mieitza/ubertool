use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    obfuscated: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    out.emit_value(&Out0 { obfuscated: obfuscate(s) })
}

fn obfuscate(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let n = w.chars().count();
            if n == 0 {
                String::new()
            } else if n == 1 {
                w.to_string()
            } else if n == 2 {
                let first = w.chars().next().unwrap();
                format!("{first}*")
            } else {
                let chars: Vec<char> = w.chars().collect();
                let first = chars[0];
                let last = chars[n - 1];
                let stars = "*".repeat(n - 2);
                format!("{first}{stars}{last}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
