//! PDF utilities — signature info extraction and cryptographic verification.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod signature;
pub mod verify;

#[derive(Debug, Args)]
pub struct PdfArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Extract signature info from a PDF; optionally verify integrity.
    #[command(
        long_about = "Extract digital signature info from a PDF.\n\nReports signature count and, per signature, signer name (/Name) and signing time (/M) when present.\n\nWith --verify: cryptographically checks each RSA-signed signature for integrity — i.e. (a) the ByteRange digest matches the messageDigest signed attribute, and (b) the RSA signature over the signed attributes is valid against the embedded signer certificate's public key.\n\nScope limits of --verify:\n  - RSA PKCS#1 v1.5 signatures only; ECDSA/DSA → verified: null.\n  - Integrity only: a self-signed cert that correctly signs the PDF reports verified: true.  Certificate chain / trust-anchor validation is NOT performed.\n  - No timestamp or revocation (OCSP/CRL) checking.\n\nExamples:\n  ubertool pdf signature --in ./signed.pdf\n  ubertool pdf signature --verify --in ./signed.pdf --json\n\nExit codes:\n  3   invalid or unreadable PDF (invalid_pdf)\n  4   file not found / permission denied"
    )]
    Signature(SignatureArgs),
}

#[derive(Debug, Args)]
pub struct SignatureArgs {
    /// Path to the PDF file.
    #[arg(long = "in")]
    pub in_path: std::path::PathBuf,

    /// Cryptographically verify each signature's integrity (RSA only; no trust-chain validation).
    #[arg(long, default_value_t = false)]
    pub verify: bool,
}

pub fn dispatch(args: PdfArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Signature(a) => signature::run(a, out),
    }
}
