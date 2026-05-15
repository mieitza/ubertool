use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Literal input to decode (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    decoded: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let decoded = html_escape::decode_html_entities(s).into_owned();
    out.emit_value(&DecodeOutput { decoded })
}
