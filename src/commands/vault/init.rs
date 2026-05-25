//! `vault init` — create a new empty vault.

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Overwrite an existing vault file.
    #[arg(long)]
    pub force: bool,
}

#[derive(Serialize)]
struct InitOut {
    path: String,
}

pub fn run(args: InitArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    if vault_path.exists() && !args.force {
        return Err(CliError::new(
            ErrorCode::VaultExists,
            format!("vault already exists at {}", vault_path.display()),
        )
        .with_hint("use --force to overwrite the existing vault"));
    }

    let password = auth::resolve_password(vault_path, None, "New vault password: ")?;

    storage::create_vault(vault_path, &password)?;

    // Store in keyring best-effort.
    auth::keyring_store(vault_path, &password);

    out.emit_value(&InitOut {
        path: vault_path.display().to_string(),
    })
}
