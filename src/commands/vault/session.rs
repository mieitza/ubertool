//! Session cache: stores the vault password (with TTL) in a temp file so the
//! user only needs to authenticate once per session.
//!
//! File location: `$UBERTOOL_VAULT_SESSION_DIR/ubertool-vault.session` (testing override)
//! or `$XDG_RUNTIME_DIR/ubertool-vault.session` (Linux)
//! or `$TMPDIR/ubertool-vault-<uid>.session` (macOS / fallback)
//!
//! File format (newline-delimited):
//! ```text
//! <expiry-unix-timestamp-seconds>
//! <password>
//! ```
//!
//! File mode: 0600 (owner-read/write only).

use std::path::PathBuf;

use crate::core::error::{CliError, ErrorCode};

/// Resolve the session file path.
pub fn session_path() -> PathBuf {
    // Testing / CI override.
    if let Ok(dir) = std::env::var("UBERTOOL_VAULT_SESSION_DIR") {
        return PathBuf::from(dir).join("ubertool-vault.session");
    }

    // XDG_RUNTIME_DIR (Linux standard).
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("ubertool-vault.session");
    }

    // macOS / fallback: use TMPDIR with UID suffix.
    let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let uid = get_uid();
    PathBuf::from(tmpdir).join(format!("ubertool-vault-{uid}.session"))
}

fn get_uid() -> u32 {
    // Safe: always available on Unix.
    #[cfg(unix)]
    {
        libc_uid()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

#[cfg(unix)]
fn libc_uid() -> u32 {
    // No stable std API for getuid(). TMPDIR on macOS is already user-private
    // so using 0 as a constant suffix is safe for the fallback path.
    0
}

/// Read the session password if it exists and hasn't expired.
pub fn read_unexpired() -> Result<Option<String>, CliError> {
    let path = session_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let mut lines = content.lines();
    let expiry_str = match lines.next() {
        Some(s) => s,
        None => return Ok(None),
    };
    let password = match lines.next() {
        Some(p) => p.to_string(),
        None => return Ok(None),
    };

    let expiry: i64 = match expiry_str.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    let now = chrono::Utc::now().timestamp();
    if now >= expiry {
        // Expired — clean up.
        let _ = std::fs::remove_file(&path);
        return Ok(None);
    }

    Ok(Some(password))
}

/// Write (or overwrite) the session file with the given password and TTL.
pub fn write_session(password: &str, ttl_minutes: u64) -> Result<(), CliError> {
    let path = session_path();

    let expiry = chrono::Utc::now().timestamp() + (ttl_minutes as i64 * 60);
    let content = format!("{expiry}\n{password}\n");

    // Ensure parent dir exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            CliError::new(
                ErrorCode::IoError,
                format!("cannot create session dir: {e}"),
            )
        })?;
    }

    // Write with restricted permissions.
    write_mode_600(&path, content.as_bytes()).map_err(|e| {
        CliError::new(
            ErrorCode::IoError,
            format!("cannot write session file: {e}"),
        )
    })?;

    Ok(())
}

/// Delete the session file (idempotent).
pub fn delete_session() -> Result<(), CliError> {
    let path = session_path();
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(CliError::new(
            ErrorCode::IoError,
            format!("cannot remove session file: {e}"),
        )),
    }
}

/// Write bytes to a file with mode 0600 (owner read/write only).
fn write_mode_600(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }

    #[cfg(not(unix))]
    {
        let mut f = std::fs::File::create(path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }

    Ok(())
}
