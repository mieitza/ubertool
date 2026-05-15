use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::parse_json;
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    yaml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_json(input.as_str()?)?;
    let yaml = serde_yaml::to_string(&v).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("yaml serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { yaml })
}
