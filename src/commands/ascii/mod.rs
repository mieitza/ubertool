//! ASCII art (figlet) rendering.

use clap::{Args, Subcommand};
use figlet_rs::FIGfont;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct AsciiArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Render text as ASCII art using the figlet standard font.
    #[command(
        long_about = "Render text as ASCII art using the figlet `standard` font.\n\nExamples:\n  ubertool ascii draw 'Hello'\n  ubertool ascii draw 'Hi' --json"
    )]
    Draw(DrawArgs),
}

#[derive(Debug, Args)]
pub struct DrawArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    art: String,
}

pub fn dispatch(args: AsciiArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Draw(a) => run(a, out),
    }
}

fn run(args: DrawArgs, out: &Out) -> Result<(), CliError> {
    let font = FIGfont::standard().map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("could not load figlet font: {e}"),
        )
    })?;
    let art = if args.input.is_empty() {
        String::new()
    } else {
        font.convert(&args.input)
            .ok_or_else(|| {
                CliError::new(ErrorCode::Internal, "figlet failed to render".to_string())
            })?
            .to_string()
    };
    out.emit_value(&Out0 { art })
}
