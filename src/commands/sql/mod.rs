//! SQL formatter.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;
use sqlformat::{FormatOptions, Indent, QueryParams};

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SqlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Pretty-print SQL with consistent indentation.
    #[command(long_about = "Pretty-print SQL.\n\nExamples:\n  ubertool sql format 'SELECT * FROM users WHERE id=1'\n  ubertool sql format 'select 1' --uppercase --json")]
    Format(FormatArgs),
}

#[derive(Debug, Args)]
pub struct FormatArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Uppercase SQL keywords.
    #[arg(long, default_value_t = false)]
    pub uppercase: bool,
    /// Indent width in spaces (default 2).
    #[arg(long, default_value_t = 2)]
    pub indent: u8,
}

#[derive(Serialize)]
struct Out0 {
    sql: String,
}

pub fn dispatch(args: SqlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Format(a) => run(a, out),
    }
}

fn run(args: FormatArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let opts = FormatOptions {
        indent: Indent::Spaces(args.indent),
        uppercase: args.uppercase,
        lines_between_queries: 1,
    };
    let formatted = sqlformat::format(input.as_str()?, &QueryParams::None, opts);
    out.emit_value(&Out0 { sql: formatted })
}
