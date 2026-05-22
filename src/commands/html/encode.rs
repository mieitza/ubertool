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
    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

#[derive(Serialize)]
struct EncodeOutput {
    encoded: String,
}

fn encode_html(s: &str) -> String {
    let encoded = html_escape::encode_text(s).into_owned();
    // html_escape's encode_text only escapes <, >, & by default; we also need quotes
    // for attribute safety. Apply an additional pass.
    encoded.replace('"', "&quot;").replace('\'', "&#x27;")
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    if args.batch {
        return crate::core::batch::run_jsonl("encoded", |line| Ok(encode_html(line)));
    }
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let encoded = encode_html(s);
    out.emit_value(&EncodeOutput { encoded })
}
