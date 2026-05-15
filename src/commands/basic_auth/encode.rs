use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Username.
    #[arg(long)]
    pub user: String,
    /// Password.
    #[arg(long)]
    pub pass: String,
}

#[derive(Serialize)]
struct EncodeOutput {
    header: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let credential = format!("{}:{}", args.user, args.pass);
    let b64 = STANDARD.encode(credential.as_bytes());
    out.emit_value(&EncodeOutput {
        header: format!("Basic {b64}"),
    })
}
