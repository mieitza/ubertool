//! `vault delete <name>` — remove a secret from the vault.

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Secret name (key).
    pub name: String,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Serialize)]
struct DeleteOut {
    name: String,
    deleted: bool,
}

pub fn run(args: DeleteArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    // Require --yes in non-interactive context.
    if !args.yes && !crate::core::tty::is_stdin_tty() {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "non-interactive delete requires --yes",
        )
        .with_hint("pass --yes to confirm deletion without a prompt"));
    }

    // Interactive confirmation when --yes is not given.
    if !args.yes {
        eprint!("Delete secret '{}'? [y/N] ", args.name);
        let mut input = String::new();
        use std::io::BufRead;
        std::io::stdin()
            .lock()
            .read_line(&mut input)
            .map_err(CliError::from)?;
        if input.trim().to_lowercase() != "y" {
            return Err(CliError::new(ErrorCode::UsageError, "deletion cancelled"));
        }
    }

    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    let (header, mut plaintext) = storage::read_vault(vault_path, &password)?;

    if !plaintext.secrets.contains_key(&args.name) {
        return Err(CliError::new(
            ErrorCode::VaultNotFound,
            format!("secret '{}' not found in vault", args.name),
        ));
    }

    plaintext.secrets.remove(&args.name);

    storage::write_vault(vault_path, &header, &plaintext, &password)?;

    out.emit_value(&DeleteOut {
        name: args.name,
        deleted: true,
    })
}
