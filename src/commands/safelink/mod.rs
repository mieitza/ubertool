//! Decode Outlook/Google/etc. safelinks back to the wrapped URL.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use percent_encoding::percent_decode_str;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SafelinkArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Unwrap a wrapped URL (Outlook safelink, Google redirect, etc.).
    #[command(long_about = "Unwrap a wrapped URL. Checks `?url=...`, `?q=...`, `?u=...` query parameters in order. If none match, returns the original URL unchanged.\n\nExamples:\n  ubertool safelink decode 'https://safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com'\n  ubertool safelink decode --in ./url.txt --json")]
    Decode(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    url: String,
}

pub fn dispatch(args: SafelinkArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Decode(a) => run(a, out),
    }
}

fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim();
    let url = unwrap_safelink(s);
    out.emit_value(&Out0 { url })
}

fn unwrap_safelink(s: &str) -> String {
    let (_, query) = match s.split_once('?') {
        Some(parts) => parts,
        None => return s.to_string(),
    };
    for key in &["url=", "q=", "u="] {
        if let Some(found) = query.split('&').find(|pair| pair.starts_with(key)) {
            let val = &found[key.len()..];
            return percent_decode_str(val)
                .decode_utf8()
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| s.to_string());
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlook_form() {
        let s = unwrap_safelink(
            "https://safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com",
        );
        assert_eq!(s, "https://example.com");
    }

    #[test]
    fn passes_through_when_no_known_key() {
        let s = unwrap_safelink("https://example.com/?foo=bar");
        assert_eq!(s, "https://example.com/?foo=bar");
    }
}
