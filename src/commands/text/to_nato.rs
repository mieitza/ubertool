use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    nato: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    const LETTERS: [&str; 26] = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
        "India", "Juliet", "Kilo", "Lima", "Mike", "November", "Oscar", "Papa",
        "Quebec", "Romeo", "Sierra", "Tango", "Uniform", "Victor", "Whiskey",
        "X-ray", "Yankee", "Zulu",
    ];
    const DIGITS: [&str; 10] = [
        "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine",
    ];
    let words: Vec<&str> = s
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                let idx = c.to_ascii_uppercase() as usize - 'A' as usize;
                Some(LETTERS[idx])
            } else if c.is_ascii_digit() {
                let idx = c as usize - '0' as usize;
                Some(DIGITS[idx])
            } else {
                None
            }
        })
        .collect();
    out.emit_value(&Out0 { nato: words.join(" ") })
}
