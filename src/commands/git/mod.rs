//! Git cheat-sheet memo.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MEMO: &str = include_str!("../../data/git_memo.md");

#[derive(Debug, Args)]
pub struct GitArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Print the bundled git cheat sheet (Markdown).
    #[command(long_about = "Print the bundled git cheat sheet as Markdown.\n\nCovers setup, basic workflow, branches, remotes, undo/inspect, stash, search, tags.\n\nExamples:\n  ubertool git memo\n  ubertool git memo --json   # wraps Markdown in {\"memo\": \"...\"}")]
    Memo,
}

#[derive(Serialize)]
struct Out0 {
    memo: String,
}

pub fn dispatch(args: GitArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Memo => out.emit_value(&Out0 { memo: MEMO.to_string() }),
    }
}
