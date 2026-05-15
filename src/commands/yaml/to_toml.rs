use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use crate::commands::json::convert::json_to_toml;

use super::{parse_yaml, RunArgs};

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
    let yaml_val = parse_yaml(input.as_str()?)?;
    // Bridge via serde_json::Value to reuse the json_to_toml machinery.
    let json_val: serde_json::Value = serde_json::to_value(yaml_val).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("yaml→json bridge failed: {e}"))
    })?;
    let toml_val = json_to_toml(json_val)?;
    let s = toml::to_string_pretty(&toml_val).map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("TOML serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { toml: s })
}
