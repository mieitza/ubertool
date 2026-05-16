//! List separator conversion + cleanup.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::{Out, OutputMode};
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a list between separator styles, optionally trimming/deduping/sorting.
    #[command(long_about = "Convert a list between separator styles.\n\nSeparators: comma, newline, space, tab, semicolon, pipe.\n\nExamples:\n  ubertool list convert 'a,b,c' --from comma --to newline\n  ubertool list convert ' a , b , a ' --from comma --to comma --trim --dedupe --sort")]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Sep {
    Comma,
    Newline,
    Space,
    Tab,
    Semicolon,
    Pipe,
}

impl Sep {
    fn as_str(self) -> &'static str {
        match self {
            Sep::Comma => ",",
            Sep::Newline => "\n",
            Sep::Space => " ",
            Sep::Tab => "\t",
            Sep::Semicolon => ";",
            Sep::Pipe => "|",
        }
    }
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub from: Sep,
    #[arg(long, value_enum)]
    pub to: Sep,
    #[arg(long, default_value_t = false)]
    pub trim: bool,
    #[arg(long, default_value_t = false)]
    pub dedupe: bool,
    #[arg(long, default_value_t = false)]
    pub sort: bool,
}

#[derive(Serialize)]
struct Out0 {
    items: Vec<String>,
}

pub fn dispatch(args: ListArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let raw = input.as_str()?;
    let mut items: Vec<String> = raw.split(args.from.as_str()).map(|s| s.to_string()).collect();
    if args.trim {
        items = items.iter().map(|s| s.trim().to_string()).collect();
    }
    items.retain(|s| !s.is_empty());
    if args.dedupe {
        let mut seen = std::collections::HashSet::new();
        items.retain(|s| seen.insert(s.clone()));
    }
    if args.sort {
        items.sort();
    }
    match out.mode {
        OutputMode::Json => out.emit_value(&Out0 { items }),
        OutputMode::Text | OutputMode::Quiet => {
            use std::io::Write;
            let joined = items.join(args.to.as_str());
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{joined}").map_err(CliError::from)
        }
    }
}
