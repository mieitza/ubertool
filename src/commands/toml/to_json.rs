use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::{parse_toml, ToJsonArgs};

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: ToJsonArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_toml(input.as_str()?)?;
    let json = if args.pretty {
        serde_json::to_string_pretty(&v)
    } else {
        serde_json::to_string(&v)
    }
    .map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("json serialization failed: {e}"),
        )
    })?;
    out.emit_value(&Out0 { json })
}
