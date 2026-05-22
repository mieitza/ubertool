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
    InvalidEmail,
    InvalidIban,
    InvalidPhone,
    InvalidJwt,
    InvalidBase64,
    InvalidBcrypt,
    InvalidCsv,
    InvalidIntegerBase,
    InvalidRoman,
    InvalidUtf8,
    InvalidBinary,
    InvalidCodepoint,
    InvalidPdf,
    FileNotFound,
    PermissionDenied,
    IoError,
    BinaryToTtyRefused,
    SignatureMismatch,
    DecryptFailed,
    PdfSignatureInvalid,
    AlgoNotSupported,
    InvalidDockerRun,
    InvalidIp,
    InvalidMac,
    InvalidMath,
    InvalidDate,
    InvalidCron,
    InvalidChmod,
    InvalidCipher,
    InvalidUrl,
    Internal,
    BatchPartialFailure,
}

impl ErrorCode {
    pub fn exit(&self) -> ExitCode {
        use ErrorCode::*;
        match self {
            UsageError | BinaryToTtyRefused => ExitCode::Usage,
            InvalidJson | InvalidYaml | InvalidToml | InvalidXml | InvalidRegex | InvalidIban
            | InvalidEmail | InvalidPhone | InvalidJwt | InvalidBase64 | InvalidBcrypt
            | InvalidCsv | InvalidIntegerBase | InvalidRoman | InvalidUtf8 | InvalidBinary
            | InvalidCodepoint | InvalidPdf | InvalidDockerRun | InvalidIp | InvalidMac
            | InvalidMath | InvalidDate | InvalidCron | InvalidChmod | InvalidCipher
            | InvalidUrl | BatchPartialFailure => ExitCode::Invalid,
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
            InvalidEmail => "invalid_email",
            InvalidIban => "invalid_iban",
            InvalidPhone => "invalid_phone",
            InvalidJwt => "invalid_jwt",
            InvalidBase64 => "invalid_base64",
            InvalidBcrypt => "invalid_bcrypt",
            InvalidCsv => "invalid_csv",
            InvalidIntegerBase => "invalid_integer_base",
            InvalidRoman => "invalid_roman",
            InvalidUtf8 => "invalid_utf8",
            InvalidBinary => "invalid_binary",
            InvalidCodepoint => "invalid_codepoint",
            InvalidPdf => "invalid_pdf",
            FileNotFound => "file_not_found",
            PermissionDenied => "permission_denied",
            IoError => "io_error",
            BinaryToTtyRefused => "binary_to_tty_refused",
            SignatureMismatch => "signature_mismatch",
            DecryptFailed => "decrypt_failed",
            PdfSignatureInvalid => "pdf_signature_invalid",
            AlgoNotSupported => "algo_not_supported",
            InvalidDockerRun => "invalid_docker_run",
            InvalidIp => "invalid_ip",
            InvalidMac => "invalid_mac",
            InvalidMath => "invalid_math",
            InvalidDate => "invalid_date",
            InvalidCron => "invalid_cron",
            InvalidChmod => "invalid_chmod",
            InvalidCipher => "invalid_cipher",
            InvalidUrl => "invalid_url",
            Internal => "internal",
            BatchPartialFailure => "batch_partial_failure",
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
        assert_eq!(ErrorCode::InvalidBinary.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidCodepoint.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::FileNotFound.exit(), ExitCode::Io);
        assert_eq!(ErrorCode::SignatureMismatch.exit(), ExitCode::Crypto);
        assert_eq!(ErrorCode::AlgoNotSupported.exit(), ExitCode::Unsupported);
        assert_eq!(ErrorCode::InvalidPdf.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidDockerRun.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidMac.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidMath.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidDate.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidCron.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidChmod.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidCipher.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::InvalidUrl.exit(), ExitCode::Invalid);
        assert_eq!(ErrorCode::Internal.exit(), ExitCode::Generic);
        assert_eq!(ErrorCode::BinaryToTtyRefused.exit(), ExitCode::Usage);
        assert_eq!(ErrorCode::BatchPartialFailure.exit(), ExitCode::Invalid);
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

    #[test]
    fn error_code_as_str_matches_serde_serialization() {
        // Guard against the as_str() ↔ #[serde(rename_all = "snake_case")] drift.
        // Every variant we test here must produce identical strings via both paths.
        // When you add a new ErrorCode variant, add it to this list.
        let cases = [
            ErrorCode::UsageError,
            ErrorCode::InvalidJson,
            ErrorCode::InvalidYaml,
            ErrorCode::InvalidToml,
            ErrorCode::InvalidXml,
            ErrorCode::InvalidRegex,
            ErrorCode::InvalidEmail,
            ErrorCode::InvalidIban,
            ErrorCode::InvalidPhone,
            ErrorCode::InvalidJwt,
            ErrorCode::InvalidBase64,
            ErrorCode::InvalidBcrypt,
            ErrorCode::InvalidCsv,
            ErrorCode::InvalidIntegerBase,
            ErrorCode::InvalidRoman,
            ErrorCode::InvalidUtf8,
            ErrorCode::InvalidBinary,
            ErrorCode::InvalidCodepoint,
            ErrorCode::InvalidPdf,
            ErrorCode::FileNotFound,
            ErrorCode::PermissionDenied,
            ErrorCode::IoError,
            ErrorCode::BinaryToTtyRefused,
            ErrorCode::SignatureMismatch,
            ErrorCode::DecryptFailed,
            ErrorCode::PdfSignatureInvalid,
            ErrorCode::AlgoNotSupported,
            ErrorCode::InvalidDockerRun,
            ErrorCode::InvalidIp,
            ErrorCode::InvalidMac,
            ErrorCode::InvalidMath,
            ErrorCode::InvalidDate,
            ErrorCode::InvalidCron,
            ErrorCode::InvalidChmod,
            ErrorCode::InvalidCipher,
            ErrorCode::InvalidUrl,
            ErrorCode::Internal,
            ErrorCode::BatchPartialFailure,
        ];
        for code in cases {
            let v = serde_json::to_value(code).unwrap();
            let serde_str = v.as_str().expect("variants serialize to strings");
            assert_eq!(
                code.as_str(),
                serde_str,
                "as_str() drifted from serde for {:?}",
                code
            );
        }
    }
}
