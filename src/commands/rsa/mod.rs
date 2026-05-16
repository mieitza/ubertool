//! RSA keypair generation.

use clap::{Args, Subcommand};
use pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct RsaArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate an RSA keypair (PKCS#8 PEM).
    #[command(long_about = "Generate an RSA keypair and emit both keys as PKCS#8 PEM.\n\nExamples:\n  ubertool rsa keypair\n  ubertool rsa keypair --bits 4096 --json\n\nExit codes:\n  2   --bits not in {2048, 3072, 4096}")]
    Keypair(KeypairArgs),
}

#[derive(Debug, Args)]
pub struct KeypairArgs {
    /// Key size in bits. Supported: 2048, 3072, 4096.
    #[arg(long, default_value_t = 2048)]
    pub bits: usize,
}

#[derive(Serialize)]
struct Out0 {
    bits: usize,
    private_pem: String,
    public_pem: String,
}

pub fn dispatch(args: RsaArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Keypair(a) => run(a, out),
    }
}

fn run(args: KeypairArgs, out: &Out) -> Result<(), CliError> {
    if !matches!(args.bits, 2048 | 3072 | 4096) {
        return Err(CliError::new(
            ErrorCode::UsageError,
            format!("unsupported --bits {} (must be 2048, 3072, or 4096)", args.bits),
        ));
    }
    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, args.bits)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("RSA keygen failed: {e}")))?;
    let pub_key = RsaPublicKey::from(&priv_key);
    let private_pem = priv_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("private PEM encode: {e}")))?
        .to_string();
    let public_pem = pub_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("public PEM encode: {e}")))?;
    out.emit_value(&Out0 {
        bits: args.bits,
        private_pem,
        public_pem,
    })
}
