//! `vault get <name>` — retrieve a secret value.

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct GetArgs {
    /// Secret name (key).
    pub name: String,
}

#[derive(Serialize)]
struct GetOut {
    name: String,
    value: String,
}

pub fn run(args: GetArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    let (_header, plaintext) = storage::read_vault(vault_path, &password)?;

    let entry = plaintext.secrets.get(&args.name).ok_or_else(|| {
        CliError::new(
            ErrorCode::VaultNotFound,
            format!("secret '{}' not found in vault", args.name),
        )
    })?;

    match out.mode {
        OutputMode::Json => {
            // Structured output: {"name": "...", "value": "..."}
            out.emit_value(&GetOut {
                name: args.name,
                value: entry.value.clone(),
            })
        }
        OutputMode::Text | OutputMode::Quiet => {
            // Bare value to stdout with newline.
            use std::io::Write;
            writeln!(std::io::stdout().lock(), "{}", entry.value).map_err(CliError::from)
        }
    }
}
