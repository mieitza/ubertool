//! Shared --batch JSONL driver. Reads newline-delimited stdin, applies a
//! per-line transform, emits one JSON object per line.
//!
//! Exit-code contract: returns `Ok(())` if every line succeeded; returns a
//! `BatchPartialFailure` error (exit 3) if any line produced an error record.
//! The JSONL is always written to stdout in full regardless.
//!
//! `--batch` + `--json` behaviour: the final `BatchPartialFailure` error is
//! printed to stderr by `main`'s normal error path and, when `--json` is also
//! set, one extra JSON object is appended to stdout after the JSONL. That extra
//! object is itself valid JSONL (one more line), so the output remains machine-
//! readable and is the simplest approach that requires no special-casing.

use std::io::{BufRead, Write};

use serde_json::json;

use crate::core::error::{CliError, ErrorCode};
use crate::core::tty::is_stdin_tty;

/// Run a per-line transform over stdin in JSONL mode.
///
/// `result_field` is the JSON key for a successful result (e.g. `"hash"`).
/// `transform` maps one input line to either the result string or a `CliError`.
///
/// Returns `Ok(())` if all lines succeeded, or a `BatchPartialFailure`
/// `CliError` (exit 3) if any line failed (after all JSONL has been written).
pub fn run_jsonl<F>(result_field: &str, transform: F) -> Result<(), CliError>
where
    F: Fn(&str) -> Result<String, CliError>,
{
    if is_stdin_tty() {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "batch mode reads newline-delimited input from stdin",
        )
        .with_hint("pipe data in, e.g. `printf 'a\\nb\\n' | ubertool hash sha256 --batch`"));
    }

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let mut any_error = false;

    for line in stdin.lock().lines() {
        let line = line.map_err(CliError::from)?;
        if line.is_empty() {
            continue;
        }
        let record = match transform(&line) {
            Ok(value) => json!({ "input": line, result_field: value }),
            Err(e) => {
                any_error = true;
                json!({
                    "input": line,
                    "error": e.code.as_str(),
                    "message": e.message,
                })
            }
        };
        let s = serde_json::to_string(&record)
            .map_err(|e| CliError::new(ErrorCode::Internal, format!("jsonl encode: {e}")))?;
        writeln!(stdout, "{s}").map_err(CliError::from)?;
    }

    if any_error {
        // All JSONL already written; this just sets the exit code to 3.
        Err(CliError::new(
            ErrorCode::BatchPartialFailure,
            "one or more batch inputs failed (see JSONL error records on stdout)",
        ))
    } else {
        Ok(())
    }
}
