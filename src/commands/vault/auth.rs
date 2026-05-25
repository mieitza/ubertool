//! Password resolution chain for vault operations.
//!
//! Precedence (highest to lowest):
//!   1. Explicit password (e.g. from --password-from-stdin, already obtained by caller).
//!   2. OS keyring (skipped if UBERTOOL_VAULT_NO_KEYRING=1).
//!   3. Live session cache (`vault unlock` writes this).
//!   4. Env var UBERTOOL_VAULT_PASSWORD.
//!   5. Interactive TTY prompt via rpassword.
//!   6. Fail with a clear error.
//!
//! Keyring is checked before the session cache: the keyring is the persistent
//! "I trust this machine" assertion the user made at vault-init time, while
//! the session cache is a short-lived convenience for headless / no-keyring
//! environments. Trying the keyring first means the session cache is only
//! consulted when the keyring is unavailable or explicitly disabled.

use std::path::Path;

use crate::core::error::{CliError, ErrorCode};

/// Resolve the vault password using the priority chain.
pub fn resolve_password(
    _vault_file: &Path,
    explicit: Option<String>,
    prompt_label: &str,
) -> Result<String, CliError> {
    // 1. Explicit (caller already obtained it).
    if let Some(p) = explicit {
        return Ok(p);
    }

    // 2. OS keyring (skip in tests/CI via UBERTOOL_VAULT_NO_KEYRING=1).
    if std::env::var("UBERTOOL_VAULT_NO_KEYRING").as_deref() != Ok("1") {
        if let Ok(p) = keyring_read(_vault_file) {
            return Ok(p);
        }
    }

    // 3. Live session cache.
    if let Some(p) = super::session::read_unexpired()? {
        return Ok(p);
    }

    // 4. Env var.
    if let Ok(p) = std::env::var("UBERTOOL_VAULT_PASSWORD") {
        if !p.is_empty() {
            return Ok(p);
        }
    }

    // 5. Interactive prompt (must be TTY).
    if crate::core::tty::is_stdin_tty() {
        return rpassword::prompt_password(prompt_label)
            .map_err(|e| CliError::new(ErrorCode::IoError, format!("cannot read password: {e}")));
    }

    // 6. Fail.
    Err(
        CliError::new(ErrorCode::UsageError, "no vault password available").with_hint(
            "set UBERTOOL_VAULT_PASSWORD, run `vault unlock`, or run interactively. \
             Testing: set UBERTOOL_VAULT_NO_KEYRING=1 to skip keyring",
        ),
    )
}

/// Try to read the vault password from the OS keyring.
pub fn keyring_read(vault_file: &Path) -> Result<String, keyring::Error> {
    let entry = keyring::Entry::new("ubertool-vault", &keyring_username(vault_file))?;
    entry.get_password()
}

/// Store the vault password in the OS keyring (best-effort).
/// Returns Ok(true) if stored, Ok(false) if skipped/failed (with warning on stderr).
pub fn keyring_store(vault_file: &Path, password: &str) -> bool {
    if std::env::var("UBERTOOL_VAULT_NO_KEYRING").as_deref() == Ok("1") {
        return false;
    }
    match keyring::Entry::new("ubertool-vault", &keyring_username(vault_file)) {
        Ok(entry) => match entry.set_password(password) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("warning: could not store password in keyring: {e}");
                false
            }
        },
        Err(e) => {
            eprintln!("warning: could not access keyring: {e}");
            false
        }
    }
}

/// Delete the vault password from the OS keyring (best-effort).
pub fn keyring_delete(vault_file: &Path) {
    if std::env::var("UBERTOOL_VAULT_NO_KEYRING").as_deref() == Ok("1") {
        return;
    }
    if let Ok(entry) = keyring::Entry::new("ubertool-vault", &keyring_username(vault_file)) {
        let _ = entry.delete_credential();
    }
}

/// Keyring username — currently always "default" but keyed to allow multi-vault later.
fn keyring_username(_vault_file: &Path) -> String {
    "default".to_string()
}
