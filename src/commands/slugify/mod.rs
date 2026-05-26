//! URL slug generation.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SlugifyArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a URL-safe slug.
    #[command(
        long_about = "Generate a URL-safe slug from text. Unicode is transliterated to ASCII, non-alphanumeric characters become hyphens, runs of hyphens collapse.\n\nExamples:\n  ubertool slugify generate \"Hello, World!\"\n  echo -n \"Café Résumé\" | ubertool slugify generate --json\n  ubertool slugify generate --in ./title.txt"
    )]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    slug: String,
}

pub fn dispatch(args: SlugifyArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    out.emit_value(&Out0 {
        slug: slug::slugify(s),
    })
}
