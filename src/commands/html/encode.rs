use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Literal input to encode (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct EncodeOutput {
    encoded: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let encoded = html_escape::encode_text(s).into_owned();
    // html_escape's encode_text only escapes <, >, & by default; we also need quotes
    // for attribute safety. Apply an additional pass.
    let encoded = encoded.replace('"', "&quot;").replace('\'', "&#x27;");
    out.emit_value(&EncodeOutput { encoded })
}
