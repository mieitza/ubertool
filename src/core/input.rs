//! Input resolution: positional > --in file > stdin pipe > fail fast on TTY.
//! Single rule used identically by every command — design-spec §6.

use std::io::Read;
use std::path::Path;

use crate::core::error::{CliError, ErrorCode};

#[derive(Debug)]
pub enum Input {
    Bytes(Vec<u8>),
}

impl Input {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Input::Bytes(b) => b,
        }
    }

    pub fn as_str(&self) -> Result<&str, CliError> {
        match self {
            Input::Bytes(b) => std::str::from_utf8(b).map_err(|_| {
                CliError::new(ErrorCode::UsageError, "input is not valid UTF-8")
            }),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Input::Bytes(b) => b,
        }
    }
}

/// Resolves the canonical input source.
///
/// Precedence:
///   1. `positional` if `Some`.
///   2. `in_path` if `Some` — reads file.
///   3. stdin if it is *not* a TTY (i.e., piped/redirected).
///   4. Otherwise, returns a `UsageError` with a hint. Never blocks waiting
///      for interactive stdin.
pub fn resolve_input(
    positional: Option<&str>,
    in_path: Option<&Path>,
    stdin_is_tty: bool,
) -> Result<Input, CliError> {
    resolve_input_with_reader(positional, in_path, stdin_is_tty, &mut std::io::stdin())
}

/// Test seam: same as `resolve_input` but takes the stdin reader explicitly.
pub fn resolve_input_with_reader<R: Read>(
    positional: Option<&str>,
    in_path: Option<&Path>,
    stdin_is_tty: bool,
    stdin: &mut R,
) -> Result<Input, CliError> {
    if let Some(p) = positional {
        return Ok(Input::Bytes(p.as_bytes().to_vec()));
    }
    if let Some(path) = in_path {
        let bytes = std::fs::read(path).map_err(CliError::from)?;
        return Ok(Input::Bytes(bytes));
    }
    if stdin_is_tty {
        return Err(CliError::new(ErrorCode::UsageError, "no input provided")
            .with_hint(
                "pass a positional argument, use --in <path>, or pipe data via stdin",
            ));
    }
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf).map_err(CliError::from)?;
    Ok(Input::Bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional_wins() {
        let mut empty: &[u8] = b"";
        let r = resolve_input_with_reader(Some("hello"), None, false, &mut empty).unwrap();
        assert_eq!(r.as_bytes(), b"hello");
    }

    #[test]
    fn in_path_reads_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"file-bytes").unwrap();
        let mut empty: &[u8] = b"";
        let r =
            resolve_input_with_reader(None, Some(tmp.path()), false, &mut empty).unwrap();
        assert_eq!(r.as_bytes(), b"file-bytes");
    }

    #[test]
    fn in_path_missing_returns_io_error() {
        let mut empty: &[u8] = b"";
        let err = resolve_input_with_reader(
            None,
            Some(Path::new("/nonexistent/path/here")),
            false,
            &mut empty,
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::FileNotFound);
    }

    #[test]
    fn stdin_pipe_reads_to_end() {
        let mut stdin: &[u8] = b"piped";
        let r = resolve_input_with_reader(None, None, false, &mut stdin).unwrap();
        assert_eq!(r.as_bytes(), b"piped");
    }

    #[test]
    fn tty_no_input_fails_with_usage_error() {
        let mut stdin: &[u8] = b"";
        let err = resolve_input_with_reader(None, None, true, &mut stdin).unwrap_err();
        assert_eq!(err.code, ErrorCode::UsageError);
        assert!(err.hint.is_some());
    }
}
