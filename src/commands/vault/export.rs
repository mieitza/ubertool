//! `vault export` — dump the full decrypted vault as JSON.
//!
//! Refuses to write to a TTY (to prevent secrets in scrollback) unless --out is used.

use clap::Args;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Write output to this file instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<std::path::PathBuf>,
}

pub fn run(args: ExportArgs, vault_path: &std::path::Path, _out: &Out) -> Result<(), CliError> {
    // Refuse to write plaintext secrets to a TTY.
    if args.out.is_none() && crate::core::tty::is_stdout_tty() {
        return Err(CliError::new(
            ErrorCode::BinaryToTtyRefused,
            "refusing to export plaintext secrets to a terminal",
        )
        .with_hint("use --out <path> to write to a file, or redirect stdout"));
    }

    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    let (_header, plaintext) = storage::read_vault(vault_path, &password)?;

    let json = serde_json::to_string_pretty(&plaintext)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("serialization failed: {e}")))?;

    if let Some(ref path) = args.out {
        std::fs::write(path, json.as_bytes()).map_err(CliError::from)?;
    } else {
        use std::io::Write;
        std::io::stdout()
            .lock()
            .write_all(json.as_bytes())
            .map_err(CliError::from)?;
        // Ensure trailing newline for terminal sanity.
        if !json.ends_with('\n') {
            println!();
        }
    }

    Ok(())
}
