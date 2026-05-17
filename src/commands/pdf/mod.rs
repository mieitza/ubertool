//! PDF utilities — signature info extraction.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod signature;

#[derive(Debug, Args)]
pub struct PdfArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Extract signature info from a PDF.
    #[command(
        long_about = "Extract digital signature info from a PDF.\n\nReports signature count and, per signature, signer name (/Name) and signing time (/M) when present. Does NOT cryptographically verify the signature.\n\nExamples:\n  ubertool pdf signature --in ./signed.pdf\n  ubertool pdf signature --in ./signed.pdf --json\n\nExit codes:\n  3   invalid or unreadable PDF (invalid_pdf)\n  4   file not found / permission denied"
    )]
    Signature(SignatureArgs),
}

#[derive(Debug, Args)]
pub struct SignatureArgs {
    #[arg(long = "in")]
    pub in_path: std::path::PathBuf,
}

pub fn dispatch(args: PdfArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Signature(a) => signature::run(a, out),
    }
}
