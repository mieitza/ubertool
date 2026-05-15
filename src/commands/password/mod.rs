//! Password strength scoring via zxcvbn.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct PasswordArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Score a password's strength (zxcvbn).
    #[command(long_about = "Score a password's strength using zxcvbn.\n\nReturns a score 0-4 (very-weak..very-strong), a log10 estimate of guesses required, and feedback strings. The password itself is never echoed in output.\n\nExamples:\n  ubertool password score \"P@ssw0rd!\"\n  echo -n \"P@ssw0rd!\" | ubertool password score --json")]
    Score(ScoreArgs),
}

#[derive(Debug, Args)]
pub struct ScoreArgs {
    /// Password (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct ScoreOutput {
    score: u8,
    strength: &'static str,
    guesses_log10: f64,
    warning: Option<String>,
    suggestions: Vec<String>,
}

pub fn dispatch(args: PasswordArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Score(a) => run(a, out),
    }
}

fn run(args: ScoreArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    // Strip trailing newline from piped/file input — common ergonomic snag.
    let password = input.as_str()?.trim_end_matches(['\r', '\n']);

    let entropy = zxcvbn::zxcvbn(password, &[]);
    let score = match &entropy {
        Ok(e) => e.score(),
        Err(_) => 0,
    };
    let strength = match score {
        0 => "very-weak",
        1 => "weak",
        2 => "fair",
        3 => "strong",
        _ => "very-strong",
    };
    let guesses_log10 = match &entropy {
        Ok(e) => e.guesses_log10(),
        Err(_) => 0.0,
    };
    let (warning, suggestions) = match &entropy {
        Ok(e) => match e.feedback() {
            Some(fb) => (
                fb.warning().map(|w| w.to_string()),
                fb.suggestions().iter().map(|s| s.to_string()).collect(),
            ),
            None => (None, Vec::new()),
        },
        Err(_) => (None, Vec::new()),
    };

    out.emit_value(&ScoreOutput {
        score,
        strength,
        guesses_log10,
        warning,
        suggestions,
    })
}
