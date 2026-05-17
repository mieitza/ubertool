use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chacha20poly1305::ChaCha20Poly1305;
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::{derive_key, Algo, Kdf};

#[derive(Debug, Args)]
pub struct DecryptArgs {
    pub input: String,
    #[arg(long)]
    pub password: String,
}

#[derive(Serialize)]
struct Out0 {
    plaintext: String,
}

pub fn run(args: DecryptArgs, out: &Out) -> Result<(), CliError> {
    let parts: Vec<&str> = args.input.split('$').collect();
    if parts.len() != 5 {
        return Err(CliError::new(
            ErrorCode::InvalidCipher,
            format!("expected 5 $-separated parts, got {}", parts.len()),
        )
        .with_hint("format: <algo>$<kdf>$<salt>$<nonce>$<ciphertext>"));
    }
    let algo = Algo::parse(parts[0]).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidCipher,
            format!("unknown algo: {}", parts[0]),
        )
    })?;
    let kdf = Kdf::parse(parts[1]).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidCipher,
            format!("unknown kdf: {}", parts[1]),
        )
    })?;
    let salt = STANDARD
        .decode(parts[2])
        .map_err(|_| CliError::new(ErrorCode::InvalidCipher, "salt is not valid base64"))?;
    let nonce = STANDARD
        .decode(parts[3])
        .map_err(|_| CliError::new(ErrorCode::InvalidCipher, "nonce is not valid base64"))?;
    let ct = STANDARD
        .decode(parts[4])
        .map_err(|_| CliError::new(ErrorCode::InvalidCipher, "ciphertext is not valid base64"))?;
    let key = derive_key(kdf, args.password.as_bytes(), &salt, 32)?;
    let pt = match algo {
        Algo::AesGcm => {
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES key error: {e}")))?;
            cipher
                .decrypt(Nonce::from_slice(&nonce), ct.as_ref())
                .map_err(|_| {
                    CliError::new(
                        ErrorCode::DecryptFailed,
                        "authentication failed — wrong password or tampered ciphertext",
                    )
                    .with_input(serde_json::json!({"password": "<redacted>"}))
                    .with_hint("confirm the password matches the one used to encrypt")
                })?
        }
        Algo::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| {
                CliError::new(ErrorCode::Internal, format!("ChaCha key error: {e}"))
            })?;
            cipher
                .decrypt(chacha20poly1305::Nonce::from_slice(&nonce), ct.as_ref())
                .map_err(|_| {
                    CliError::new(ErrorCode::DecryptFailed, "authentication failed")
                        .with_input(serde_json::json!({"password": "<redacted>"}))
                })?
        }
    };
    let plaintext = String::from_utf8(pt)
        .map_err(|_| CliError::new(ErrorCode::InvalidUtf8, "decrypted plaintext is not UTF-8"))?;
    out.emit_value(&Out0 { plaintext })
}
