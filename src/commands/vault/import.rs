//! `vault import <file>` — import secrets from an export JSON file.

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::auth;
use super::storage::{self, VaultPlaintext};

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// JSON file to import (from `vault export`).
    pub file: std::path::PathBuf,
    /// On name collision: overwrite existing entry.
    #[arg(long)]
    pub replace: bool,
    /// Clear the vault before importing (requires --yes).
    #[arg(long)]
    pub clear: bool,
    /// Confirm clear (required when --clear is used).
    #[arg(long)]
    pub yes: bool,
}

#[derive(Serialize)]
struct ImportOut {
    imported: usize,
    skipped: usize,
    replaced: usize,
}

pub fn run(args: ImportArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    if args.clear && !args.yes {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "--clear requires --yes for confirmation",
        ));
    }

    let import_bytes = std::fs::read(&args.file).map_err(CliError::from)?;
    let import_data: VaultPlaintext = serde_json::from_slice(&import_bytes).map_err(|e| {
        CliError::new(
            ErrorCode::VaultCorrupt,
            format!("malformed import JSON: {e}"),
        )
    })?;

    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;
    let (header, mut plaintext) = storage::read_vault(vault_path, &password)?;

    if args.clear {
        plaintext.secrets.clear();
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut replaced = 0usize;

    for (name, entry) in import_data.secrets {
        match plaintext.secrets.entry(name) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                if args.replace {
                    e.insert(entry);
                    replaced += 1;
                } else {
                    skipped += 1;
                }
            }
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(entry);
                imported += 1;
            }
        }
    }

    storage::write_vault(vault_path, &header, &plaintext, &password)?;

    out.emit_value(&ImportOut {
        imported,
        skipped,
        replaced,
    })
}
