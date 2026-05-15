use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::parse_json;
use super::PrettifyArgs;

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: PrettifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_json(input.as_str()?)?;
    let indent_bytes = " ".repeat(args.indent);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes.as_bytes());
    let mut buf = Vec::new();
    {
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        use serde::Serialize as _;
        v.serialize(&mut ser).map_err(|e| {
            CliError::new(ErrorCode::Internal, format!("json prettify failed: {e}"))
        })?;
    }
    let pretty = String::from_utf8(buf).map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("pretty buffer is not UTF-8: {e}"),
        )
    })?;
    out.emit_value(&Out0 { json: pretty })
}
