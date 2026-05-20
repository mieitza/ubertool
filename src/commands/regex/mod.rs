//! Regular expression utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod memo;
pub mod test;

#[derive(Debug, Args)]
pub struct RegexArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Test whether a pattern matches text; emit groups.
    #[command(
        long_about = "Test whether a regex pattern matches text and emit any capture groups.\n\nNote: Rust's regex crate does not support lookaround or backreferences.\nUse --fancy to switch to the fancy-regex engine which supports both.\n\nExamples:\n  ubertool regex test --pattern '\\d+' --text '42'\n  ubertool regex test --pattern '(\\w+)@(\\w+)' --text 'a@b' --json\n  ubertool regex test --pattern 'foo(?=bar)' --text 'foobar' --fancy\n  ubertool regex test --pattern '(\\w+) \\1' --text 'hello hello' --fancy\n\nExit codes:\n  3   pattern does not compile (invalid_regex)"
    )]
    Test(TestArgs),
    /// Generate a string matching a pattern.
    #[command(
        long_about = "Generate a string that matches a regex pattern.\n\nLimitations: lookaround, backreferences, and anchors are not supported.\n\nExamples:\n  ubertool regex generate '\\d{3}-\\d{4}'\n  ubertool regex generate '[A-Z]{2,4}'"
    )]
    Generate(GenerateArgs),
    /// Print the bundled regex cheat sheet (Markdown).
    #[command(
        long_about = "Print the bundled regex cheat sheet as Markdown.\n\nCovers anchors, character classes, quantifiers, groups, lookaround, common patterns, and inline flags. Notes which features are not supported by Rust's `regex` crate.\n\nExamples:\n  ubertool regex memo\n  ubertool regex memo --json"
    )]
    Memo,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(long)]
    pub pattern: String,
    #[arg(long)]
    pub text: String,
    /// Use the fancy-regex engine (supports lookaround and backreferences).
    #[arg(long, default_value_t = false)]
    pub fancy: bool,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub pattern: String,
}

pub fn dispatch(args: RegexArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Test(a) => test::run(a, out),
        Verb::Generate(a) => generate::run(a, out),
        Verb::Memo => memo::run(out),
    }
}
