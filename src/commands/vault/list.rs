//! `vault list` — list secret names (never includes values).

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct ListArgs {}

#[derive(Serialize)]
struct ListEntry {
    name: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ListOut {
    secrets: Vec<ListEntry>,
}

pub fn run(_args: ListArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    let (_header, plaintext) = storage::read_vault(vault_path, &password)?;

    let secrets: Vec<ListEntry> = plaintext
        .secrets
        .into_iter()
        .map(|(name, entry)| ListEntry {
            name,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        })
        .collect();

    // In text/quiet mode: one name per line.
    // We handle this manually to avoid leaking values.
    use crate::core::output::OutputMode;
    if out.mode != OutputMode::Json {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        for s in &secrets {
            writeln!(stdout, "{}", s.name).map_err(CliError::from)?;
        }
        return Ok(());
    }

    out.emit_value(&ListOut { secrets })
}
