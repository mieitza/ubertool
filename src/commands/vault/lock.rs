//! `vault lock` — delete the session cache (idempotent).

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

use super::session;

#[derive(Debug, Args)]
pub struct LockArgs {}

#[derive(Serialize)]
struct LockOut {
    locked: bool,
}

pub fn run(_args: LockArgs, _vault_path: &std::path::Path, out: &Out) -> Result<(), CliError> {
    session::delete_session()?;
    out.emit_value(&LockOut { locked: true })
}
