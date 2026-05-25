//! Encrypted local secrets store.
//!
//! Stores secrets in a single encrypted file (default: `~/.config/ubertool/vault.enc`).
//! Override via `--vault-file <path>` or `UBERTOOL_VAULT_FILE` env var.
//!
//! Encryption: AES-256-GCM with Argon2id key derivation.
//!
//! Password resolution order:
//!   1. OS keyring (skip with UBERTOOL_VAULT_NO_KEYRING=1 for testing/CI).
//!   2. Live session cache (`vault unlock` stores this).
//!   3. UBERTOOL_VAULT_PASSWORD environment variable.
//!   4. Interactive TTY prompt.
//!
//! Testing escape hatches (not for regular use):
//!   UBERTOOL_VAULT_NO_KEYRING=1   — skip OS keyring lookup
//!   UBERTOOL_VAULT_SESSION_DIR=<dir> — override session file directory

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod auth;
pub mod delete;
pub mod export;
pub mod get;
pub mod import;
pub mod init;
pub mod list;
pub mod lock;
pub mod session;
pub mod set;
pub mod storage;
pub mod unlock;

#[derive(Debug, Args)]
#[command(long_about = "Encrypted local secrets store.\n\
                  \n\
                  Secrets are stored in a single AES-256-GCM encrypted file (default:\n\
                  ~/.config/ubertool/vault.enc). Override the path with --vault-file or\n\
                  UBERTOOL_VAULT_FILE.\n\
                  \n\
                  Password resolution order:\n\
                  1. OS keyring (skip with UBERTOOL_VAULT_NO_KEYRING=1)\n\
                  2. Live session cache (written by `vault unlock`)\n\
                  3. UBERTOOL_VAULT_PASSWORD environment variable\n\
                  4. Interactive TTY prompt\n\
                  \n\
                  Usage pattern: --secret \"$(ubertool vault get my-key)\"\n\
                  \n\
                  Testing/CI escape hatches:\n\
                  UBERTOOL_VAULT_NO_KEYRING=1       skip OS keyring\n\
                  UBERTOOL_VAULT_SESSION_DIR=<dir>  override session file location")]
pub struct VaultArgs {
    /// Path to the vault file (overrides UBERTOOL_VAULT_FILE and default).
    #[arg(long, value_name = "PATH", global = true)]
    pub vault_file: Option<PathBuf>,

    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Create a new empty vault.
    #[command(long_about = "Create a new empty encrypted vault.\n\
                      \n\
                      Reads the master password via the auth chain (will prompt if interactive;\n\
                      uses UBERTOOL_VAULT_PASSWORD env in non-TTY contexts). Stores the\n\
                      password in the OS keyring (best-effort — if keyring fails, a warning\n\
                      is printed but the vault is still created).\n\
                      \n\
                      Exit codes: 2 if vault already exists without --force; 4 on I/O failure.")]
    Init(init::InitArgs),
    /// Create or update a secret.
    #[command(long_about = "Create or update a secret in the vault.\n\
                      \n\
                      Value source priority: positional argument > --in <file> > stdin pipe >\n\
                      interactive prompt (when stdin is a TTY or --prompt is given).\n\
                      \n\
                      The value is never echoed in command output.\n\
                      \n\
                      Exit codes: 2 (no value in non-prompt context); 4 (I/O); 5 (wrong password).")]
    Set(set::SetArgs),
    /// Retrieve a secret value.
    #[command(long_about = "Retrieve a secret value from the vault.\n\
                      \n\
                      In text/quiet mode: bare value on stdout (with newline).\n\
                      In --json mode: {\"name\": \"...\", \"value\": \"...\"}.\n\
                      \n\
                      The value is NEVER emitted on stderr or in error JSON.\n\
                      \n\
                      Exit codes: 3 (secret not found); 5 (wrong password).")]
    Get(get::GetArgs),
    /// List secret names (values are never included).
    #[command(long_about = "List all secret names in the vault.\n\
                      \n\
                      Values are NEVER included in list output.\n\
                      Text mode: one name per line.\n\
                      JSON mode: {\"secrets\": [{\"name\": ..., \"created_at\": ..., \"updated_at\": ...}]}\n\
                      \n\
                      Exit codes: 5 (wrong password).")]
    List(list::ListArgs),
    /// Delete a secret (requires --yes in non-interactive contexts).
    #[command(long_about = "Delete a secret from the vault.\n\
                      \n\
                      Requires --yes in non-interactive (non-TTY) contexts. Without --yes on\n\
                      a TTY, prompts for confirmation.\n\
                      \n\
                      Exit codes: 2 (no --yes on non-TTY); 3 (not found); 5 (wrong password).")]
    Delete(delete::DeleteArgs),
    /// Export the full decrypted vault as JSON.
    #[command(long_about = "Export the full decrypted vault as pretty JSON.\n\
                      \n\
                      WARNING: output includes all secret values in plaintext.\n\
                      Refuses to write to a terminal (to prevent secrets in scrollback).\n\
                      Use --out <path> to write to a file, or pipe/redirect stdout.\n\
                      \n\
                      Output format matches the `vault import` input format.\n\
                      \n\
                      Exit codes: 2 (TTY output refused); 5 (wrong password).")]
    Export(export::ExportArgs),
    /// Import secrets from a JSON file (from `vault export`).
    #[command(
        long_about = "Import secrets from a JSON file produced by `vault export`.\n\
                      \n\
                      Default (--merge): add to existing vault; skip name collisions.\n\
                      --replace: overwrite existing entries on collision.\n\
                      --clear: wipe vault before import (requires --yes).\n\
                      \n\
                      Exit codes: 3 (malformed import JSON); 5 (wrong password)."
    )]
    Import(import::ImportArgs),
    /// Authenticate and cache the password for a session.
    #[command(
        long_about = "Authenticate against the vault and cache the password for a session.\n\
                      \n\
                      Writes a session file (mode 0600) so subsequent vault commands don't\n\
                      need the password for the duration of the TTL.\n\
                      \n\
                      Session file location:\n\
                      - UBERTOOL_VAULT_SESSION_DIR env var (testing override)\n\
                      - $XDG_RUNTIME_DIR/ubertool-vault.session (Linux)\n\
                      - $TMPDIR/ubertool-vault-<uid>.session (macOS / fallback)\n\
                      \n\
                      Exit codes: 5 (wrong password); 4 (cannot write session file)."
    )]
    Unlock(unlock::UnlockArgs),
    /// Delete the session cache (idempotent).
    #[command(long_about = "Delete the vault session cache.\n\
                      \n\
                      After locking, subsequent vault commands will require re-authentication.\n\
                      Idempotent — locking an already-locked vault returns success.\n\
                      \n\
                      Exit codes: 0 always (unless I/O error on session file).")]
    Lock(lock::LockArgs),
}

/// Resolve the vault file path from the --vault-file flag, UBERTOOL_VAULT_FILE env,
/// or the default location (~/.config/ubertool/vault.enc).
pub fn resolve_vault_path(override_path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p.clone();
    }
    if let Ok(p) = std::env::var("UBERTOOL_VAULT_FILE") {
        return PathBuf::from(p);
    }
    // Default: ~/.config/ubertool/vault.enc
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ubertool")
        .join("vault.enc")
}

pub fn dispatch(args: VaultArgs, out: &Out) -> Result<(), CliError> {
    let vault_path = resolve_vault_path(args.vault_file.as_ref());
    match args.verb {
        Verb::Init(a) => init::run(a, &vault_path, out),
        Verb::Set(a) => set::run(a, &vault_path, out),
        Verb::Get(a) => get::run(a, &vault_path, out),
        Verb::List(a) => list::run(a, &vault_path, out),
        Verb::Delete(a) => delete::run(a, &vault_path, out),
        Verb::Export(a) => export::run(a, &vault_path, out),
        Verb::Import(a) => import::run(a, &vault_path, out),
        Verb::Unlock(a) => unlock::run(a, &vault_path, out),
        Verb::Lock(a) => lock::run(a, &vault_path, out),
    }
}
