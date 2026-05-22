use std::path::PathBuf;

use clap::Args;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
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
    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

#[derive(Serialize)]
struct EncodeOutput {
    encoded: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    if args.batch {
        return crate::core::batch::run_jsonl("encoded", |line| {
            Ok(utf8_percent_encode(line, NON_ALPHANUMERIC).to_string())
        });
    }
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let encoded = utf8_percent_encode(s, NON_ALPHANUMERIC).to_string();
    out.emit_value(&EncodeOutput { encoded })
}
