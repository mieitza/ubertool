//! Markdown → HTML conversion via pulldown-cmark.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use pulldown_cmark::{html, Parser};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct MarkdownArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Render Markdown as HTML.
    #[command(name = "to-html", long_about = "Render Markdown as HTML using CommonMark.\n\nExamples:\n  ubertool markdown to-html '# Hello'\n  cat README.md | ubertool markdown to-html\n  ubertool markdown to-html '**bold**' --json")]
    ToHtml(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    html: String,
}

pub fn dispatch(args: MarkdownArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToHtml(a) => run(a, out),
    }
}

fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let mut html_buf = String::new();
    let parser = Parser::new(input.as_str()?);
    html::push_html(&mut html_buf, parser);
    out.emit_value(&Out0 { html: html_buf })
}
