use std::io::Cursor;

use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    xml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let formatted = format_xml(input.as_str()?)?;
    out.emit_value(&Out0 { xml: formatted })
}

fn format_xml(xml: &str) -> Result<String, CliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let mut buf = Vec::new();
    // Track open/close balance so we can detect truncated input like `<bad`.
    // quick-xml is lenient and will emit a Start event for `<bad` (treating EOF
    // as the end of the tag) without emitting a matching End event.
    let mut depth: i64 = 0;
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(CliError::new(
                    ErrorCode::InvalidXml,
                    format!("invalid XML: {e}"),
                ));
            }
            Ok(Event::Eof) => break,
            Ok(ref ev) => {
                match ev {
                    Event::Start(_) => depth += 1,
                    Event::End(_) => depth -= 1,
                    _ => {}
                }
                writer.write_event(ev.clone()).map_err(|e| {
                    CliError::new(ErrorCode::Internal, format!("xml write failed: {e}"))
                })?;
            }
        }
        buf.clear();
    }
    if depth != 0 {
        return Err(CliError::new(
            ErrorCode::InvalidXml,
            "invalid XML: unclosed elements (truncated input)",
        ));
    }
    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("xml output not UTF-8: {e}")))
}
