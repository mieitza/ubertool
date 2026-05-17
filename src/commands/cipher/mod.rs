//! Authenticated encryption / decryption with password-based key derivation.

use clap::{Args, Subcommand, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decrypt;
pub mod encrypt;

#[derive(Debug, Args)]
pub struct CipherArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encrypt input with password-derived AEAD.
    #[command(
        long_about = "Encrypt input using AES-256-GCM or ChaCha20-Poly1305 with a key derived from the password via Argon2id (default) or PBKDF2-SHA256.\n\nOutput format: <algo>$<kdf>$<salt>$<nonce>$<ciphertext> with all components base64-encoded.\n\nExamples:\n  ubertool cipher encrypt 'hello' --password 'pw'\n  ubertool cipher encrypt 'msg' --password 'pw' --algo chacha20-poly1305\n  ubertool cipher encrypt 'msg' --password 'pw' --kdf pbkdf2 --json"
    )]
    Encrypt(encrypt::EncryptArgs),
    /// Decrypt input produced by `cipher encrypt`.
    #[command(
        long_about = "Decrypt input previously produced by `cipher encrypt`.\n\nThe algo and KDF are auto-detected from the self-describing format.\n\nExamples:\n  ubertool cipher decrypt 'aes-gcm$argon2$...' --password 'pw'\n\nExit codes:\n  3   malformed cipher format (invalid_cipher)\n  5   authentication tag failed — wrong password or tampered ciphertext (decrypt_failed)"
    )]
    Decrypt(decrypt::DecryptArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Algo {
    #[value(name = "aes-gcm")]
    AesGcm,
    #[value(name = "chacha20-poly1305")]
    ChaCha20Poly1305,
}

impl Algo {
    pub fn label(self) -> &'static str {
        match self {
            Algo::AesGcm => "aes-gcm",
            Algo::ChaCha20Poly1305 => "chacha20-poly1305",
        }
    }
    pub fn parse(s: &str) -> Option<Algo> {
        match s {
            "aes-gcm" => Some(Algo::AesGcm),
            "chacha20-poly1305" => Some(Algo::ChaCha20Poly1305),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Kdf {
    Argon2,
    Pbkdf2,
}

impl Kdf {
    pub fn label(self) -> &'static str {
        match self {
            Kdf::Argon2 => "argon2",
            Kdf::Pbkdf2 => "pbkdf2",
        }
    }
    pub fn parse(s: &str) -> Option<Kdf> {
        match s {
            "argon2" => Some(Kdf::Argon2),
            "pbkdf2" => Some(Kdf::Pbkdf2),
            _ => None,
        }
    }
}

pub fn dispatch(args: CipherArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encrypt(a) => encrypt::run(a, out),
        Verb::Decrypt(a) => decrypt::run(a, out),
    }
}

pub(super) fn derive_key(
    kdf: Kdf,
    password: &[u8],
    salt: &[u8],
    key_len: usize,
) -> Result<Vec<u8>, CliError> {
    use crate::core::error::ErrorCode;
    let mut key = vec![0u8; key_len];
    match kdf {
        Kdf::Argon2 => {
            let argon = argon2::Argon2::default();
            argon
                .hash_password_into(password, salt, &mut key)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("argon2 failed: {e}")))?;
        }
        Kdf::Pbkdf2 => {
            pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, 600_000, &mut key);
        }
    }
    Ok(key)
}
