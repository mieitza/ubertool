//! String case conversion.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use convert_case::{Case, Casing};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct CaseArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a string between case styles.
    #[command(
        long_about = "Convert a string between case styles.\n\nExamples:\n  ubertool case convert \"hello world\" --style snake\n  ubertool case convert \"helloWorld\" --style kebab\n  ubertool case convert \"my var\" --style screaming-snake --json\n  ubertool case convert --in ./text.txt --style camel"
    )]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Style {
    Snake,
    Kebab,
    Camel,
    Pascal,
    #[value(name = "screaming-snake")]
    ScreamingSnake,
    Upper,
    Lower,
    Title,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Target case style.
    #[arg(long, value_enum)]
    pub style: Style,
}

#[derive(Serialize)]
struct Out0 {
    result: String,
}

pub fn dispatch(args: CaseArgs, out: &Out) -> Result<(), CliError> {
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
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    let case = match args.style {
        Style::Snake => Case::Snake,
        Style::Kebab => Case::Kebab,
        Style::Camel => Case::Camel,
        Style::Pascal => Case::Pascal,
        Style::ScreamingSnake => Case::ScreamingSnake,
        Style::Upper => Case::Upper,
        Style::Lower => Case::Lower,
        Style::Title => Case::Title,
    };
    out.emit_value(&Out0 {
        result: s.to_case(case),
    })
}
