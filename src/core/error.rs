//! Typed error envelope. Every CLI failure is a `CliError` with an `ErrorCode`,
//! a human message, optional input-echo, optional hint, and a retriable bool.
//! The mapping ErrorCode -> ExitCode is the single source of truth for the
//! design-spec §7/§8 contract.

use crate::core::exit::ExitCode;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UsageError,
    InvalidJson,
    InvalidYaml,
    InvalidToml,
    InvalidXml,
    InvalidRegex,
    InvalidIban,
    InvalidPhone,
    InvalidJwt,
    InvalidBase64,
    FileNotFound,
    PermissionDenied,
    IoError,
    BinaryToTtyRefused,
    SignatureMismatch,
    DecryptFailed,
    PdfSignatureInvalid,
    AlgoNotSupported,
    Internal,
}

impl ErrorCode {
    pub fn exit(&self) -> ExitCode {
        use ErrorCode::*;
        match self {
            UsageError | BinaryToTtyRefused => ExitCode::Usage,
            InvalidJson | InvalidYaml | InvalidToml | InvalidXml | InvalidRegex | InvalidIban
            | InvalidPhone | InvalidJwt | InvalidBase64 => ExitCode::Invalid,
            FileNotFound | PermissionDenied | IoError => ExitCode::Io,
            SignatureMismatch | DecryptFailed | PdfSignatureInvalid => ExitCode::Crypto,
            AlgoNotSupported => ExitCode::Unsupported,
            Internal => ExitCode::Generic,
        }
    }

    pub fn as_str(&self) -> &'static str {
        use ErrorCode::*;
        match self {
            UsageError => "usage_error",
            InvalidJson => "invalid_json",
            InvalidYaml => "invalid_yaml",
            InvalidToml => "invalid_toml",
            InvalidXml => "invalid_xml",
            InvalidRegex => "invalid_regex",
            InvalidIban => "invalid_iban",
            InvalidPhone => "invalid_phone",
            InvalidJwt => "invalid_jwt",
            InvalidBase64 => "invalid_base64",
            FileNotFound => "file_not_found",
            PermissionDenied => "permission_denied",
            IoError => "io_error",
            BinaryToTtyRefused => "binary_to_tty_refused",
            SignatureMismatch => "signature_mismatch",
            DecryptFailed => "decrypt_failed",
            PdfSignatureInvalid => "pdf_signature_invalid",
            AlgoNotSupported => "algo_not_supported",
            Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CliError {
    #[serde(rename = "error")]
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub retriable: bool,
}

impl CliError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            input: None,
            hint: None,
            retriable: false,
        }
    }

    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = Some(input);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn retriable(mut self) -> Self {
        self.retriable = true;
        self
    }

    pub fn exit(&self) -> ExitCode {
        self.code.exit()
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        let code = match e.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::IoError,
        };
        Self::new(code, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_to_exit_code_mapping() {
        assert_eq!(ErrorCode::UsageError.exit(), ExitCode::Usage);
        assert_eq!(ErrorCode::InvalidBase64.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::FileNotFound.exit(), ExitCode::Io);
        assert_eq!(ErrorCode::SignatureMismatch.exit(), ExitCode::Crypto);
        assert_eq!(ErrorCode::AlgoNotSupported.exit(), ExitCode::Unsupported);
        assert_eq!(ErrorCode::Internal.exit(), ExitCode::Generic);
        assert_eq!(ErrorCode::BinaryToTtyRefused.exit(), ExitCode::Usage);
    }

    #[test]
    fn error_code_strings_are_snake_case() {
        assert_eq!(ErrorCode::InvalidBase64.as_str(), "invalid_base64");
        assert_eq!(ErrorCode::SignatureMismatch.as_str(), "signature_mismatch");
    }

    #[test]
    fn cli_error_serializes_with_envelope_fields() {
        let e = CliError::new(ErrorCode::InvalidBase64, "bad input")
            .with_input(serde_json::json!("xxx"))
            .with_hint("ensure base64");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["error"], "invalid_base64");
        assert_eq!(v["message"], "bad input");
        assert_eq!(v["input"], "xxx");
        assert_eq!(v["hint"], "ensure base64");
        assert_eq!(v["retriable"], false);
    }

    #[test]
    fn cli_error_omits_optional_fields_when_unset() {
        let e = CliError::new(ErrorCode::InvalidBase64, "x");
        let v = serde_json::to_value(&e).unwrap();
        assert!(v.get("input").is_none());
        assert!(v.get("hint").is_none());
    }

    #[test]
    fn io_error_maps_not_found_to_file_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let cli_err: CliError = io_err.into();
        assert_eq!(cli_err.code, ErrorCode::FileNotFound);
    }
}
