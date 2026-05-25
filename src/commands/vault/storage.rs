//! Vault file format: read, write, encrypt, decrypt.
//!
//! Binary layout:
//! ```text
//! Offset  Size  Field
//! 0       8     Magic       b"UBERVLT1"
//! 8       1     Version     0x01
//! 9       1     KDF id      0x01 (Argon2id)
//! 10      16    Salt        random per-vault, NEVER rotated on save
//! 26      1     Argon2 t_cost   default 3
//! 27      4     Argon2 m_cost   little-endian u32, default 65536
//! 31      12    Nonce       random per-SAVE (rotated on every write)
//! 43      ..    Ciphertext  AES-256-GCM of plaintext JSON (includes auth tag)
//! ```

use std::io::Write;
use std::path::Path;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::core::error::{CliError, ErrorCode};

pub const MAGIC: &[u8; 8] = b"UBERVLT1";
pub const VERSION: u8 = 0x01;
pub const KDF_ARGON2ID: u8 = 0x01;
pub const DEFAULT_T_COST: u8 = 3;
pub const DEFAULT_M_COST: u32 = 65536;

/// Header parsed from a vault file.
pub struct VaultHeader {
    pub version: u8,
    pub kdf_id: u8,
    pub salt: [u8; 16],
    pub t_cost: u8,
    pub m_cost: u32,
    pub nonce: [u8; 12],
}

/// In-memory plaintext vault.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultPlaintext {
    pub version: u32,
    pub secrets: std::collections::BTreeMap<String, SecretEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

impl VaultPlaintext {
    pub fn new() -> Self {
        Self {
            version: 1,
            secrets: std::collections::BTreeMap::new(),
        }
    }
}

impl Default for VaultPlaintext {
    fn default() -> Self {
        Self::new()
    }
}

/// Derive a 32-byte AES-256 key using Argon2id.
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    t_cost: u32,
    m_cost: u32,
) -> Result<[u8; 32], CliError> {
    let params = argon2::Params::new(m_cost, t_cost, 1, Some(32))
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("argon2 params error: {e}")))?;
    let argon = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = [0u8; 32];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("argon2 failed: {e}")))?;
    Ok(key)
}

/// Read and decrypt a vault file, returning the plaintext and the header (needed for re-saves).
pub fn read_vault(path: &Path, password: &str) -> Result<(VaultHeader, VaultPlaintext), CliError> {
    let bytes = std::fs::read(path).map_err(|e| {
        let code = match e.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::VaultFileMissing,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::IoError,
        };
        CliError::new(code, format!("cannot read vault file: {e}"))
    })?;
    decrypt_vault(&bytes, password)
}

/// Parse and decrypt raw bytes into a vault header + plaintext.
pub fn decrypt_vault(
    bytes: &[u8],
    password: &str,
) -> Result<(VaultHeader, VaultPlaintext), CliError> {
    if bytes.len() < 43 {
        return Err(CliError::new(
            ErrorCode::VaultCorrupt,
            "vault file too short",
        ));
    }
    if &bytes[0..8] != MAGIC {
        return Err(CliError::new(
            ErrorCode::VaultCorrupt,
            "invalid vault magic bytes",
        ));
    }
    let version = bytes[8];
    if version != VERSION {
        return Err(CliError::new(
            ErrorCode::VaultCorrupt,
            format!("unsupported vault version: {version}"),
        ));
    }
    let kdf_id = bytes[9];
    if kdf_id != KDF_ARGON2ID {
        return Err(CliError::new(
            ErrorCode::VaultCorrupt,
            format!("unsupported KDF id: {kdf_id}"),
        ));
    }
    let salt: [u8; 16] = bytes[10..26].try_into().unwrap();
    let t_cost = bytes[26];
    let m_cost = u32::from_le_bytes(bytes[27..31].try_into().unwrap());
    let nonce: [u8; 12] = bytes[31..43].try_into().unwrap();
    let ciphertext = &bytes[43..];

    let key = derive_key(password.as_bytes(), &salt, t_cost as u32, m_cost)?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES key setup: {e}")))?;
    let plaintext_bytes = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext)
        .map_err(|_| {
            CliError::new(
                ErrorCode::DecryptFailed,
                "vault decryption failed — wrong password or corrupted vault",
            )
            .with_hint("confirm UBERTOOL_VAULT_PASSWORD or re-run interactively")
        })?;

    let plaintext: VaultPlaintext = serde_json::from_slice(&plaintext_bytes).map_err(|e| {
        CliError::new(
            ErrorCode::VaultCorrupt,
            format!("vault JSON malformed: {e}"),
        )
    })?;

    Ok((
        VaultHeader {
            version,
            kdf_id,
            salt,
            t_cost,
            m_cost,
            nonce,
        },
        plaintext,
    ))
}

/// Encrypt and write the vault to disk atomically.
///
/// Generates a fresh nonce on every save. Salt is preserved from `header`.
pub fn write_vault(
    path: &Path,
    header: &VaultHeader,
    plaintext: &VaultPlaintext,
    password: &str,
) -> Result<(), CliError> {
    let json_bytes = serde_json::to_vec(plaintext)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("serialization failed: {e}")))?;

    let key = derive_key(
        password.as_bytes(),
        &header.salt,
        header.t_cost as u32,
        header.m_cost,
    )?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES key setup: {e}")))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), json_bytes.as_ref())
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES encrypt failed: {e}")))?;

    let mut file_bytes = Vec::with_capacity(43 + ciphertext.len());
    file_bytes.extend_from_slice(MAGIC);
    file_bytes.push(header.version);
    file_bytes.push(header.kdf_id);
    file_bytes.extend_from_slice(&header.salt);
    file_bytes.push(header.t_cost);
    file_bytes.extend_from_slice(&header.m_cost.to_le_bytes());
    file_bytes.extend_from_slice(&nonce_bytes);
    file_bytes.extend_from_slice(&ciphertext);

    atomic_write(path, &file_bytes)
}

/// Create a new vault file with a fresh salt, write empty vault.
pub fn create_vault(path: &Path, password: &str) -> Result<VaultHeader, CliError> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let header = VaultHeader {
        version: VERSION,
        kdf_id: KDF_ARGON2ID,
        salt,
        t_cost: DEFAULT_T_COST,
        m_cost: DEFAULT_M_COST,
        nonce: [0u8; 12], // will be replaced by write_vault
    };

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            CliError::new(
                ErrorCode::IoError,
                format!("cannot create vault directory: {e}"),
            )
        })?;
    }

    let plaintext = VaultPlaintext::new();
    write_vault(path, &header, &plaintext, password)?;
    Ok(header)
}

/// Write bytes to `<path>.tmp`, fsync, then atomic rename to `path`.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let tmp_path = path.with_extension("enc.tmp");
    {
        let mut f = std::fs::File::create(&tmp_path).map_err(|e| {
            CliError::new(ErrorCode::IoError, format!("cannot create temp vault: {e}"))
        })?;
        f.write_all(bytes)
            .map_err(|e| CliError::new(ErrorCode::IoError, format!("cannot write vault: {e}")))?;
        f.sync_all()
            .map_err(|e| CliError::new(ErrorCode::IoError, format!("cannot fsync vault: {e}")))?;
    }
    std::fs::rename(&tmp_path, path)
        .map_err(|e| CliError::new(ErrorCode::IoError, format!("cannot rename vault: {e}")))?;
    Ok(())
}
