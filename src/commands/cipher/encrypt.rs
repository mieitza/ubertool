use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chacha20poly1305::ChaCha20Poly1305;
use clap::Args;
use rand::RngCore;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::{derive_key, Algo, Kdf};

#[derive(Debug, Args)]
pub struct EncryptArgs {
    pub input: String,
    #[arg(long)]
    pub password: String,
    #[arg(long, value_enum, default_value = "aes-gcm")]
    pub algo: Algo,
    #[arg(long, value_enum, default_value = "argon2")]
    pub kdf: Kdf,
}

#[derive(Serialize)]
struct Out0 {
    ciphertext: String,
}

pub fn run(args: EncryptArgs, out: &Out) -> Result<(), CliError> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);
    let key = derive_key(args.kdf, args.password.as_bytes(), &salt, 32)?;
    let ct = match args.algo {
        Algo::AesGcm => {
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES key error: {e}")))?;
            cipher
                .encrypt(Nonce::from_slice(&nonce), args.input.as_bytes())
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES encrypt failed: {e}")))?
        }
        Algo::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(&key)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("ChaCha key error: {e}")))?;
            cipher
                .encrypt(chacha20poly1305::Nonce::from_slice(&nonce), args.input.as_bytes())
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("ChaCha encrypt failed: {e}")))?
        }
    };
    let encoded = format!(
        "{}${}${}${}${}",
        args.algo.label(),
        args.kdf.label(),
        STANDARD.encode(salt),
        STANDARD.encode(nonce),
        STANDARD.encode(&ct)
    );
    out.emit_value(&Out0 { ciphertext: encoded })
}
