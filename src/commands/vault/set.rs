//! `vault set <name> [value]` — create or update a secret.

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::auth;
use super::storage;

#[derive(Debug, Args)]
pub struct SetArgs {
    /// Secret name (key).
    pub name: String,
    /// Secret value (optional positional; use --prompt, --in, or pipe instead).
    pub value: Option<String>,
    /// Read secret value from file.
    #[arg(long, value_name = "PATH")]
    pub r#in: Option<std::path::PathBuf>,
    /// Prompt for the secret value interactively (no-echo).
    #[arg(long)]
    pub prompt: bool,
}

#[derive(Serialize)]
struct SetOut {
    name: String,
    created_at: String,
    updated_at: String,
}

pub fn run(args: SetArgs, vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    // Resolve secret value.
    let value = resolve_value(&args)?;

    let password = auth::resolve_password(vault_path, None, "Vault password: ")?;

    let (header, mut plaintext) = storage::read_vault(vault_path, &password)?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let (created_at, updated_at) = if let Some(existing) = plaintext.secrets.get(&args.name) {
        (existing.created_at.clone(), now.clone())
    } else {
        (now.clone(), now.clone())
    };

    plaintext.secrets.insert(
        args.name.clone(),
        storage::SecretEntry {
            value,
            created_at: created_at.clone(),
            updated_at: updated_at.clone(),
        },
    );

    storage::write_vault(vault_path, &header, &plaintext, &password)?;

    out.emit_value(&SetOut {
        name: args.name,
        created_at,
        updated_at,
    })
}

fn resolve_value(args: &SetArgs) -> Result<String, CliError> {
    // 1. Positional value argument.
    if let Some(ref v) = args.value {
        return Ok(v.clone());
    }

    // 2. --in <file>.
    if let Some(ref path) = args.r#in {
        let bytes = std::fs::read(path).map_err(CliError::from)?;
        return String::from_utf8(bytes).map_err(|_| {
            CliError::new(
                ErrorCode::InvalidUtf8,
                "secret value file is not valid UTF-8",
            )
        });
    }

    // 3. --prompt flag or implicit prompt when stdin is TTY.
    if args.prompt || crate::core::tty::is_stdin_tty() {
        return rpassword::prompt_password("Secret value: ").map_err(|e| {
            CliError::new(ErrorCode::IoError, format!("cannot read secret value: {e}"))
        });
    }

    // 4. stdin pipe.
    if !crate::core::tty::is_stdin_tty() {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(CliError::from)?;
        // Strip trailing newline that shells typically add.
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        return Ok(buf);
    }

    Err(
        CliError::new(ErrorCode::UsageError, "no secret value provided")
            .with_hint("pass a positional value, use --in <path>, pipe via stdin, or use --prompt"),
    )
}
