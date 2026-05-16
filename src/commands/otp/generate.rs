use clap::Args;
use serde::Serialize;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(long)]
    pub secret: String,
    #[arg(long, default_value_t = 6)]
    pub digits: usize,
    #[arg(long, default_value_t = 30)]
    pub period: u64,
}

#[derive(Serialize)]
struct Out0 {
    code: String,
}

pub fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let secret_bytes = Secret::Encoded(args.secret.clone())
        .to_bytes()
        .map_err(|e| {
            CliError::new(ErrorCode::UsageError, format!("invalid base32 secret: {e:?}"))
                .with_input(serde_json::json!({"secret": "<redacted>"}))
        })?;
    let totp = TOTP::new_unchecked(Algorithm::SHA1, args.digits, 1, args.period, secret_bytes);
    let code = totp
        .generate_current()
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("TOTP generate: {e}")))?;
    out.emit_value(&Out0 { code })
}
