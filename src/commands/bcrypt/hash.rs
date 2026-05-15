use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HashArgs {
    /// Password to hash (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read password from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Bcrypt cost factor (4-31; bcrypt default is 12).
    #[arg(long, default_value_t = bcrypt::DEFAULT_COST)]
    pub cost: u32,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
}

pub fn run(args: HashArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let password = input.as_str()?;
    let hashed = bcrypt::hash(password, args.cost).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("bcrypt hashing failed: {e}"))
            .with_hint("cost must be between 4 and 31")
    })?;
    out.emit_value(&HashOutput { hash: hashed })
}
