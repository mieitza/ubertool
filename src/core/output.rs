//! Output mode and emission. Single point that enforces the design-spec §5
//! and §7 rules: JSON to stdout in --json mode, nothing else; errors to
//! stderr always plus stdout when --json; bare values for --quiet; binary
//! never lands in a TTY.

use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Text,
    Json,
    Quiet,
}

pub struct Out {
    pub mode: OutputMode,
    pub stdout_is_tty: bool,
}

impl Out {
    pub fn new(mode: OutputMode, stdout_is_tty: bool) -> Self {
        Self { mode, stdout_is_tty }
    }

    pub fn emit_value<T: Serialize>(&self, value: &T) -> Result<(), CliError> {
        let mut stdout = std::io::stdout().lock();
        self.emit_value_to(value, &mut stdout)
    }

    fn emit_value_to<T: Serialize, W: Write>(
        &self,
        value: &T,
        w: &mut W,
    ) -> Result<(), CliError> {
        match self.mode {
            OutputMode::Json => {
                let s = serde_json::to_string(value).map_err(|e| {
                    CliError::new(ErrorCode::Internal, format!("serialization failed: {e}"))
                })?;
                writeln!(w, "{s}").map_err(CliError::from)?;
            }
            OutputMode::Text | OutputMode::Quiet => {
                let v = serde_json::to_value(value).map_err(|e| {
                    CliError::new(ErrorCode::Internal, format!("serialization failed: {e}"))
                })?;
                self.emit_text_repr(&v, w)?;
            }
        }
        Ok(())
    }

    fn emit_text_repr<W: Write>(
        &self,
        v: &serde_json::Value,
        w: &mut W,
    ) -> Result<(), CliError> {
        match v {
            serde_json::Value::Object(map) if map.len() == 1 => {
                // Single-field objects: emit just the value (Text and Quiet identical).
                let (_, val) = map.iter().next().unwrap();
                self.emit_scalar(val, w)
            }
            serde_json::Value::Object(map) => {
                if self.mode == OutputMode::Quiet {
                    for val in map.values() {
                        self.emit_scalar(val, w)?;
                    }
                    Ok(())
                } else {
                    for (k, val) in map {
                        match val {
                            serde_json::Value::String(s) => writeln!(w, "{k}: {s}"),
                            serde_json::Value::Null => writeln!(w, "{k}:"),
                            other => writeln!(w, "{k}: {other}"),
                        }
                        .map_err(CliError::from)?;
                    }
                    Ok(())
                }
            }
            other => self.emit_scalar(other, w),
        }
    }

    fn emit_scalar<W: Write>(
        &self,
        v: &serde_json::Value,
        w: &mut W,
    ) -> Result<(), CliError> {
        match v {
            serde_json::Value::String(s) => writeln!(w, "{s}"),
            serde_json::Value::Null => writeln!(w),
            other => writeln!(w, "{other}"),
        }
        .map_err(CliError::from)
    }

    pub fn emit_binary(&self, bytes: &[u8], out: Option<&Path>) -> Result<(), CliError> {
        if let Some(path) = out {
            std::fs::write(path, bytes).map_err(CliError::from)?;
            return Ok(());
        }
        if self.stdout_is_tty {
            return Err(CliError::new(
                ErrorCode::BinaryToTtyRefused,
                "refusing to write binary data to a terminal",
            )
            .with_hint("redirect stdout to a file or pass --out <path>"));
        }
        std::io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(CliError::from)?;
        Ok(())
    }

    /// Emit a `CliError` to stderr in human form, and to stdout as JSON when
    /// `--json` is on. Always called by `main` on the error path.
    pub fn emit_error(&self, err: &CliError) {
        eprintln!("error: {}: {}", err.code.as_str(), err.message);
        if let Some(hint) = &err.hint {
            eprintln!("hint: {hint}");
        }
        if self.mode == OutputMode::Json {
            if let Ok(s) = serde_json::to_string(err) {
                println!("{s}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Single {
        encoded: String,
    }

    #[derive(Serialize)]
    struct Multi {
        a: String,
        b: i64,
    }

    #[test]
    fn text_mode_single_field_emits_bare_value() {
        let out = Out::new(OutputMode::Text, false);
        let mut buf: Vec<u8> = Vec::new();
        out.emit_value_to(&Single { encoded: "aGVsbG8=".into() }, &mut buf)
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "aGVsbG8=\n");
    }

    #[test]
    fn json_mode_emits_compact_json_with_newline() {
        let out = Out::new(OutputMode::Json, false);
        let mut buf: Vec<u8> = Vec::new();
        out.emit_value_to(&Single { encoded: "aGVsbG8=".into() }, &mut buf)
            .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "{\"encoded\":\"aGVsbG8=\"}\n");
        let _: serde_json::Value =
            serde_json::from_str(s.trim()).expect("must be valid JSON");
    }

    #[test]
    fn text_mode_multi_field_emits_key_value_lines() {
        let out = Out::new(OutputMode::Text, false);
        let mut buf: Vec<u8> = Vec::new();
        out.emit_value_to(&Multi { a: "x".into(), b: 42 }, &mut buf)
            .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("a: x\n"));
        assert!(s.contains("b: 42\n"));
    }

    #[test]
    fn quiet_mode_multi_field_emits_bare_values() {
        let out = Out::new(OutputMode::Quiet, false);
        let mut buf: Vec<u8> = Vec::new();
        out.emit_value_to(&Multi { a: "x".into(), b: 42 }, &mut buf)
            .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "x\n42\n");
    }

    #[test]
    fn binary_to_tty_is_refused() {
        let out = Out::new(OutputMode::Text, true);
        let err = out.emit_binary(b"\x00\x01\x02", None).unwrap_err();
        assert_eq!(err.code, ErrorCode::BinaryToTtyRefused);
    }
}
