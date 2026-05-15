use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::{json_to_toml, parse_json};
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    toml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let json_val = parse_json(input.as_str()?)?;
    let toml_val = json_to_toml(json_val)?;
    let s = toml::to_string_pretty(&toml_val).map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("TOML serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { toml: s })
}
