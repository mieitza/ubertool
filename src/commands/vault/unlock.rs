//! `vault unlock` — authenticate and cache the password for a TTL.

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

use super::auth;
use super::session;
use super::storage;

#[derive(Debug, Args)]
pub struct UnlockArgs {
    /// Session lifetime in minutes (default 15).
    #[arg(long, default_value = "15")]
    pub ttl: u64,
}

#[derive(Serialize)]
struct UnlockOut {
    unlocked_until: String,
}

pub fn run(args: UnlockArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    // Validate password by attempting a decrypt.
    storage::read_vault(vault_path, &password)?;

    // Write session file.
    session::write_session(&password, args.ttl)?;

    let unlocked_until = chrono::Utc::now()
        + chrono::Duration::try_minutes(args.ttl as i64).unwrap_or(chrono::Duration::zero());
    let ts = unlocked_until.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    out.emit_value(&UnlockOut { unlocked_until: ts })
}
