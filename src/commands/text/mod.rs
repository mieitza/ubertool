//! Text encoding and analysis verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod diff;
pub mod from_binary;
pub mod from_unicode;
pub mod obfuscate;
pub mod stats;
pub mod to_binary;
pub mod to_nato;
pub mod to_unicode;

#[derive(Debug, Args)]
pub struct TextArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode text as ASCII binary (space-separated bytes).
    #[command(
        name = "to-binary",
        long_about = "Encode text as space-separated 8-bit binary representations of each byte.\n\nExamples:\n  ubertool text to-binary 'A'      # 01000001\n  ubertool text to-binary 'Hi'     # 01001000 01101001\n  ubertool text to-binary --in ./input.txt --json"
    )]
    ToBinary(RunArgs),
    /// Decode space-separated binary back to text.
    #[command(
        name = "from-binary",
        long_about = "Decode space-separated 8-bit binary back to UTF-8 text.\n\nExamples:\n  ubertool text from-binary '01001000 01101001'\n  ubertool text from-binary '01000001' --json\n  ubertool text from-binary --in ./binary.txt\n\nExit codes:\n  3   non-binary tokens in input (invalid_binary)"
    )]
    FromBinary(RunArgs),
    /// Encode text as space-separated Unicode codepoints (U+NNNN).
    #[command(
        name = "to-unicode",
        long_about = "Encode text as space-separated Unicode codepoints in U+NNNN form.\n\nExamples:\n  ubertool text to-unicode 'Hi'\n  ubertool text to-unicode 'Hi' --json\n  ubertool text to-unicode --in ./text.txt"
    )]
    ToUnicode(RunArgs),
    /// Decode space-separated U+NNNN codepoints to text.
    #[command(
        name = "from-unicode",
        long_about = "Decode space-separated U+NNNN codepoints to UTF-8 text.\n\nExamples:\n  ubertool text from-unicode 'U+0048 U+0069'\n  ubertool text from-unicode 'U+0048 U+0069' --json\n  ubertool text from-unicode --in ./codepoints.txt\n\nExit codes:\n  3   invalid codepoint syntax or out-of-range codepoint"
    )]
    FromUnicode(RunArgs),
    /// Convert text to NATO phonetic spelling.
    #[command(
        name = "to-nato",
        long_about = "Convert text to NATO phonetic spelling. Letters → NATO word, digits → spelled name, other chars are dropped.\n\nExamples:\n  ubertool text to-nato 'SOS'\n  ubertool text to-nato 'Hi' --json\n  ubertool text to-nato --in ./text.txt"
    )]
    ToNato(RunArgs),
    /// Emit text statistics (chars/words/lines/bytes).
    #[command(
        long_about = "Emit chars, words, lines, and bytes counts as JSON or key:value lines.\n\nExamples:\n  ubertool text stats 'hello world'\n  ubertool text stats --in ./README.md --json"
    )]
    Stats(RunArgs),
    /// Unified diff between two text inputs.
    #[command(
        long_about = "Unified diff between two text inputs.\n\nExamples:\n  ubertool text diff 'hello' 'world'\n  ubertool text diff --from-file a.txt --to-file b.txt --json"
    )]
    Diff(DiffArgs),
    /// Obfuscate words by replacing middle chars with *.
    #[command(
        long_about = "Obfuscate text by replacing each word's middle characters with `*`. Preserves first character, and last character if word length > 2.\n\nExamples:\n  ubertool text obfuscate 'hello world'   # h***o w***d\n  ubertool text obfuscate 'hello world' --json\n  ubertool text obfuscate --in ./text.txt"
    )]
    Obfuscate(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// First text (or use --from-file).
    pub from: Option<String>,
    /// Second text (or use --to-file).
    pub to: Option<String>,
    #[arg(long = "from-file")]
    pub from_file: Option<PathBuf>,
    #[arg(long = "to-file")]
    pub to_file: Option<PathBuf>,
}

pub fn dispatch(args: TextArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToBinary(a) => to_binary::run(a, out),
        Verb::FromBinary(a) => from_binary::run(a, out),
        Verb::ToUnicode(a) => to_unicode::run(a, out),
        Verb::FromUnicode(a) => from_unicode::run(a, out),
        Verb::ToNato(a) => to_nato::run(a, out),
        Verb::Stats(a) => stats::run(a, out),
        Verb::Diff(a) => diff::run(a, out),
        Verb::Obfuscate(a) => obfuscate::run(a, out),
    }
}
