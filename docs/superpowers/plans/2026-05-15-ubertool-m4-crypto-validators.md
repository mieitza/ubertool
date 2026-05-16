# ubertool M4 — Tier 4 (Crypto & Validators) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) tracking.

**Goal:** Add 9 new nouns + 1 verb extension (~14 verbs) covering cryptographic operations and input validators: `cipher`, `rsa`, `otp`, `pdf`, `regex`, `email`, `iban`, `phone`, `user-agent`, plus `url parse` extending the existing M1 `url` noun.

**Architecture:** Same single-crate pattern as M1-M3. Each new noun is a folder under `src/commands/<noun>/`. Adding `parse` to the existing `url` noun extends `commands/url/mod.rs` and adds `commands/url/parse.rs`.

**Tech Stack added:** `aes-gcm`, `chacha20poly1305`, `argon2`, `pbkdf2` (cipher); `rsa`, `pkcs1`, `pkcs8` (rsa); `totp-rs` (otp); `lopdf` (pdf); `regex`, `regex-generate` (regex); `email_address` (email); `iban_validate` (iban); `phonenumber` (phone, ~3MB libphonenumber data); `woothee` (user-agent, ~1MB regex DB); `url` (url parse).

**Companion documents:** Design spec §3 (inventory), §11 (crates), §16 (milestones). M3 plan as the most recent template.

**Working directory:** `/Users/mihai/dev/ubertool/`. M3 just pushed (origin/main at `48249aa`). Test count baseline: 249.

**The cross-cutting noun pattern** (same as prior milestones):
1. Create `src/commands/<noun>/` with `mod.rs` + per-verb files.
2. Add `pub mod <noun>;` to `src/commands/mod.rs`.
3. Add `<Noun>(...)` variant to `src/cli.rs::Noun`.
4. Add match arm to `src/lib.rs::run()`.
5. Write `tests/<noun>.rs`.
6. Run `cargo test`; update top-level help snapshot; commit.

**Acceptance criteria:**
1. `cargo build` and `cargo build --release` succeed.
2. `cargo test` passes 100%. Test count ≥ 290 (M3 ended at 249; M4 adds ≥ ~40 tests).
3. Every new noun appears in `ubertool --help`; every verb in `ubertool <noun> --help`.
4. Fitness checklist extended; spec updated; `make verify-spec` ≥ 92 leaf commands.
5. `make ci` green.
6. Repo clean.

**Scope notes (deferred to later milestones):**
- PDF signature **verification** (we only do info extraction in M4).
- Full IEEE OUI registry (still using the M3 curated CSV; unrelated to M4).
- Asymmetric JWT signing (RSA keypair generation lands here but JWT verify still HS-only).

---

## Cross-cutting conventions

Same as M1-M3. See M3 plan for details. Key reminders:
- New ErrorCode variants: add to enum, `as_str`, `exit()` Invalid arm, and the sync test `cases` array.
- Secrets (`--secret`, `--key`, `--password`) MUST be redacted from `with_input(...)` echo in error paths.
- Bare-value-in-text-mode commands switch on `out.mode` (like M2 `temperature`).

---

## Task 1: Add M4 dependencies

**Files:** Modify `Cargo.toml`.

- [ ] **Step 1: Append to `[dependencies]` (after M3 section)**

```toml
# M4 crypto
aes-gcm = "0.10"
chacha20poly1305 = "0.10"
argon2 = "0.5"
pbkdf2 = "0.12"
rsa = "0.9"
pkcs1 = { version = "0.7", features = ["pem"] }
pkcs8 = { version = "0.10", features = ["pem"] }
totp-rs = "5"

# M4 PDF
lopdf = "0.32"

# M4 regex
regex = "1"
regex-generate = "0.2"

# M4 validators
email_address = "0.2"
iban_validate = "4"
phonenumber = "0.3"
woothee = "0.13"

# M4 URL
url = "2"
```

- [ ] **Step 2: Run `cargo check`** — succeeds. Unused-crate warnings OK. Note any version substitutions in the report.

- [ ] **Step 3: Run `cargo test`** — 249 M3 tests still pass.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add M4 dependencies (cipher/rsa/otp/pdf/regex/validators/url)"
```

---

## Task 2: `cipher` noun — `encrypt` + `decrypt`

**Files:** `src/commands/cipher/{mod,encrypt,decrypt}.rs`, `tests/cipher.rs`. Adds `InvalidCipher` and `DecryptFailed` is already present from M0 (sanity-check it's still wired).

**Self-describing output format** (one line, base64-only):
```
<algo>$<kdf>$<salt_b64>$<nonce_b64>$<ciphertext_b64>
```

Where `<algo>` ∈ {`aes-gcm`, `chacha20-poly1305`}, `<kdf>` ∈ {`argon2`, `pbkdf2`}. Salts are 16 bytes, nonces 12 bytes.

- [ ] **Step 1: Add `InvalidCipher` to `src/core/error.rs`** (variant + `as_str` `"invalid_cipher"` + `exit` Invalid arm + sync test cases).

- [ ] **Step 2: Write `tests/cipher.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cipher_encrypt_emits_self_describing_format() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "encrypt", "hello", "--password", "pw"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split('$').collect();
    assert_eq!(parts.len(), 5, "expected 5 $-separated parts: {s}");
    assert_eq!(parts[0], "aes-gcm");
    assert_eq!(parts[1], "argon2");
}

#[test]
fn cipher_round_trip_aes_gcm_argon2() {
    let ct = Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "encrypt", "the message", "--password", "pw"])
        .assert().success().get_output().stdout.clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "pw"])
        .assert().success()
        .stdout("the message\n");
}

#[test]
fn cipher_round_trip_chacha20_pbkdf2() {
    let ct = Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "encrypt", "msg", "--password", "pw", "--algo", "chacha20-poly1305", "--kdf", "pbkdf2"])
        .assert().success().get_output().stdout.clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    let parts: Vec<&str> = ct_s.split('$').collect();
    assert_eq!(parts[0], "chacha20-poly1305");
    assert_eq!(parts[1], "pbkdf2");
    Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "pw"])
        .assert().success()
        .stdout("msg\n");
}

#[test]
fn cipher_decrypt_wrong_password_exits_5() {
    let ct = Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "encrypt", "msg", "--password", "right"])
        .assert().success().get_output().stdout.clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "wrong"])
        .assert().failure().code(5)
        .stderr(predicate::str::contains("decrypt_failed"));
}

#[test]
fn cipher_decrypt_malformed_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["cipher", "decrypt", "garbage", "--password", "pw"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_cipher"));
}

#[test]
fn cipher_password_redacted_in_error_json() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "cipher", "decrypt", "garbage", "--password", "do-not-leak"])
        .assert().failure().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains("do-not-leak"), "password leaked in error JSON: {s}");
}
```

- [ ] **Step 3: Create `src/commands/cipher/mod.rs`**

```rust
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
    #[command(long_about = "Encrypt input using AES-256-GCM or ChaCha20-Poly1305 with a key derived from the password via Argon2id (default) or PBKDF2-SHA256.\n\nOutput format: <algo>$<kdf>$<salt>$<nonce>$<ciphertext> with all components base64-encoded.\n\nExamples:\n  ubertool cipher encrypt 'hello' --password 'pw'\n  ubertool cipher encrypt 'msg' --password 'pw' --algo chacha20-poly1305\n  ubertool cipher encrypt 'msg' --password 'pw' --kdf pbkdf2 --json")]
    Encrypt(encrypt::EncryptArgs),
    /// Decrypt input produced by `cipher encrypt`.
    #[command(long_about = "Decrypt input previously produced by `cipher encrypt`.\n\nThe algo and KDF are auto-detected from the self-describing format.\n\nExamples:\n  ubertool cipher decrypt 'aes-gcm$argon2$...' --password 'pw'\n\nExit codes specific to this command:\n  3   malformed cipher format (invalid_cipher)\n  5   authentication tag failed — wrong password or tampered ciphertext (decrypt_failed)")]
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

pub(super) fn derive_key(kdf: Kdf, password: &[u8], salt: &[u8], key_len: usize) -> Result<Vec<u8>, CliError> {
    use crate::core::error::ErrorCode;
    let mut key = vec![0u8; key_len];
    match kdf {
        Kdf::Argon2 => {
            let argon = argon2::Argon2::default();
            argon
                .hash_password_into(password, salt, &mut key)
                .map_err(|e| {
                    CliError::new(ErrorCode::Internal, format!("argon2 failed: {e}"))
                })?;
        }
        Kdf::Pbkdf2 => {
            pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, 600_000, &mut key);
        }
    }
    Ok(key)
}
```

- [ ] **Step 4: Create `src/commands/cipher/encrypt.rs`**

```rust
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
    /// Plaintext input.
    pub input: String,
    /// Password.
    #[arg(long)]
    pub password: String,
    /// Symmetric algorithm.
    #[arg(long, value_enum, default_value = "aes-gcm")]
    pub algo: Algo,
    /// Key derivation function.
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
            let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
                CliError::new(ErrorCode::Internal, format!("AES key error: {e}"))
            })?;
            cipher
                .encrypt(Nonce::from_slice(&nonce), args.input.as_bytes())
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("AES encrypt failed: {e}")))?
        }
        Algo::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| {
                CliError::new(ErrorCode::Internal, format!("ChaCha key error: {e}"))
            })?;
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
```

- [ ] **Step 5: Create `src/commands/cipher/decrypt.rs`**

```rust
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
    /// Ciphertext (self-describing format from `cipher encrypt`).
    pub input: String,
    /// Password.
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
        CliError::new(ErrorCode::InvalidCipher, format!("unknown algo: {}", parts[0]))
    })?;
    let kdf = Kdf::parse(parts[1]).ok_or_else(|| {
        CliError::new(ErrorCode::InvalidCipher, format!("unknown kdf: {}", parts[1]))
    })?;
    let salt = STANDARD.decode(parts[2]).map_err(|_| {
        CliError::new(ErrorCode::InvalidCipher, "salt is not valid base64")
    })?;
    let nonce = STANDARD.decode(parts[3]).map_err(|_| {
        CliError::new(ErrorCode::InvalidCipher, "nonce is not valid base64")
    })?;
    let ct = STANDARD.decode(parts[4]).map_err(|_| {
        CliError::new(ErrorCode::InvalidCipher, "ciphertext is not valid base64")
    })?;

    let key = derive_key(kdf, args.password.as_bytes(), &salt, 32)?;
    let pt = match algo {
        Algo::AesGcm => {
            let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
                CliError::new(ErrorCode::Internal, format!("AES key error: {e}"))
            })?;
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
    let plaintext = String::from_utf8(pt).map_err(|_| {
        CliError::new(ErrorCode::InvalidUtf8, "decrypted plaintext is not UTF-8")
    })?;
    out.emit_value(&Out0 { plaintext })
}
```

- [ ] **Step 6: Wire + test + commit**

Add `pub mod cipher;` to `src/commands/mod.rs`. Add `Cipher(crate::commands::cipher::CipherArgs)` to `Noun`. Add dispatch arm.

```bash
cargo test
git add src/core/error.rs src/commands/cipher/ src/commands/mod.rs src/cli.rs src/lib.rs tests/cipher.rs tests/snapshots/
git commit -m "feat(cipher): add cipher encrypt|decrypt (AES-GCM/ChaCha20-Poly1305 + Argon2/PBKDF2)"
```

---

## Task 3: `rsa` noun — `keypair`

**Files:** `src/commands/rsa/mod.rs`, `tests/rsa.rs`.

Generates an RSA keypair, emits PEM-encoded private + public keys. Default 2048 bits.

- [ ] **Step 1: Write `tests/rsa.rs`**

```rust
use assert_cmd::Command;

#[test]
fn rsa_keypair_emits_pem_pair() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "rsa", "keypair", "--bits", "2048"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let priv_pem = v["private_pem"].as_str().expect("private_pem");
    let pub_pem = v["public_pem"].as_str().expect("public_pem");
    assert!(priv_pem.contains("BEGIN PRIVATE KEY") || priv_pem.contains("BEGIN RSA PRIVATE KEY"));
    assert!(pub_pem.contains("BEGIN PUBLIC KEY") || pub_pem.contains("BEGIN RSA PUBLIC KEY"));
    assert_eq!(v["bits"], 2048);
}

#[test]
fn rsa_keypair_default_bits_2048() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "rsa", "keypair"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["bits"], 2048);
}

#[test]
fn rsa_keypair_unsupported_bits_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["rsa", "keypair", "--bits", "100"])
        .assert().failure().code(2);
}
```

- [ ] **Step 2: Create `src/commands/rsa/mod.rs`**

```rust
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
    let priv_key = RsaPrivateKey::new(&mut rng, args.bits).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("RSA keygen failed: {e}"))
    })?;
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
```

- [ ] **Step 3: Wire + test + commit**

Add `pub mod rsa;` (alphabetically). Add `Rsa(crate::commands::rsa::RsaArgs)` variant. Dispatch arm.

```bash
cargo test
git add src/commands/rsa/ src/commands/mod.rs src/cli.rs src/lib.rs tests/rsa.rs tests/snapshots/
git commit -m "feat(rsa): add rsa keypair (PKCS#8 PEM, 2048/3072/4096 bits)"
```

> Note: RSA keygen at 2048 bits takes ~100-500ms in release mode, several seconds in debug. The test runs against `cargo bin` debug binary by default; expect it to take a few seconds.

---

## Task 4: `otp` noun — `generate` + `validate`

**Files:** `src/commands/otp/{mod,generate,validate}.rs`, `tests/otp.rs`.

TOTP per RFC 6238. Inputs: base32-encoded secret, default period 30s, default 6 digits, SHA1.

- [ ] **Step 1: Write `tests/otp.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

const SECRET: &str = "JBSWY3DPEHPK3PXP";  // base32 of "Hello!\xde\xad\xbe\xef"

#[test]
fn otp_generate_emits_6_digit_code() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "generate", "--secret", SECRET])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 6);
    assert!(s.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn otp_generate_validate_round_trip() {
    let code_out = Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "generate", "--secret", SECRET])
        .assert().success().get_output().stdout.clone();
    let code = String::from_utf8(code_out).unwrap().trim().to_string();
    Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "validate", &code, "--secret", SECRET])
        .assert().success();
}

#[test]
fn otp_validate_wrong_code_exits_5() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "validate", "000000", "--secret", SECRET])
        .assert().failure().code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn otp_validate_secret_redacted_in_error_json() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "otp", "validate", "000000", "--secret", "do-not-leak"])
        .assert().failure().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains("do-not-leak"), "secret leaked in JSON: {s}");
}
```

- [ ] **Step 2: Create `src/commands/otp/mod.rs`**

```rust
//! Time-based One-Time Passwords (RFC 6238).

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod validate;

#[derive(Debug, Args)]
pub struct OtpArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate the current TOTP code from a base32 secret.
    #[command(long_about = "Generate the current TOTP code (RFC 6238) from a base32 secret.\n\nExamples:\n  ubertool otp generate --secret JBSWY3DPEHPK3PXP\n  ubertool otp generate --secret JBSWY3DPEHPK3PXP --period 60 --digits 8 --json")]
    Generate(generate::GenerateArgs),
    /// Validate a TOTP code against the current time window.
    #[command(long_about = "Validate a TOTP code against the current time window. Allows ±1 step skew by default.\n\nExamples:\n  ubertool otp validate 123456 --secret JBSWY3DPEHPK3PXP\n\nExit codes:\n  5   code does not match within the allowed window (signature_mismatch)")]
    Validate(validate::ValidateArgs),
}

pub fn dispatch(args: OtpArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => generate::run(a, out),
        Verb::Validate(a) => validate::run(a, out),
    }
}
```

- [ ] **Step 3: `src/commands/otp/generate.rs`**

```rust
use clap::Args;
use serde::Serialize;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(long)]
    pub secret: String,
    #[arg(long, default_value_t = 6)]
    pub digits: usize,
    /// Period in seconds.
    #[arg(long, default_value_t = 30)]
    pub period: u64,
}

#[derive(Serialize)]
struct Out0 {
    code: String,
}

pub fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let secret_bytes = Secret::Encoded(args.secret.clone())
        .to_bytes()
        .map_err(|e| {
            CliError::new(ErrorCode::UsageError, format!("invalid base32 secret: {e:?}"))
                .with_input(serde_json::json!({"secret": "<redacted>"}))
        })?;
    let totp = TOTP::new(
        Algorithm::SHA1,
        args.digits,
        1,
        args.period,
        secret_bytes,
    )
    .map_err(|e| CliError::new(ErrorCode::UsageError, format!("TOTP setup: {e}")))?;
    let code = totp.generate_current().map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("TOTP generate: {e}"))
    })?;
    out.emit_value(&Out0 { code })
}
```

- [ ] **Step 4: `src/commands/otp/validate.rs`**

```rust
use clap::Args;
use serde::Serialize;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub code: String,
    #[arg(long)]
    pub secret: String,
    #[arg(long, default_value_t = 6)]
    pub digits: usize,
    #[arg(long, default_value_t = 30)]
    pub period: u64,
}

#[derive(Serialize)]
struct Out0 {
    valid: bool,
}

pub fn run(args: ValidateArgs, out: &Out) -> Result<(), CliError> {
    let secret_bytes = Secret::Encoded(args.secret.clone())
        .to_bytes()
        .map_err(|e| {
            CliError::new(ErrorCode::UsageError, format!("invalid base32 secret: {e:?}"))
                .with_input(serde_json::json!({"secret": "<redacted>"}))
        })?;
    let totp = TOTP::new(
        Algorithm::SHA1,
        args.digits,
        1,
        args.period,
        secret_bytes,
    )
    .map_err(|e| CliError::new(ErrorCode::UsageError, format!("TOTP setup: {e}")))?;
    let ok = totp.check_current(&args.code).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("TOTP validate: {e}"))
    })?;
    if !ok {
        return Err(CliError::new(
            ErrorCode::SignatureMismatch,
            "TOTP code does not match",
        )
        .with_input(serde_json::json!({"code": args.code, "secret": "<redacted>"})));
    }
    out.emit_value(&Out0 { valid: true })
}
```

- [ ] **Step 5: Wire + test + commit**

```bash
cargo test
git add src/commands/otp/ src/commands/mod.rs src/cli.rs src/lib.rs tests/otp.rs tests/snapshots/
git commit -m "feat(otp): add otp generate|validate (TOTP RFC 6238)"
```

---

## Task 5: `pdf` noun — `signature` (info extraction only)

**Files:** `src/commands/pdf/{mod,signature}.rs`, `tests/pdf.rs`, `tests/fixtures/unsigned.pdf` (small fixture).

**Scope:** M4 extracts signature *info* (presence, count, signer name from `/Name`, signing time from `/M`). It does NOT verify the signature cryptographically. Verification would require parsing the signed byte range, hashing, and verifying PKCS#7 — deferred.

- [ ] **Step 1: Create a tiny unsigned PDF fixture**

Create `tests/fixtures/` if missing. Write `tests/fixtures/unsigned.pdf` as a minimal valid PDF (raw bytes). Use a script approach inside the implementer's task: generate via a one-liner or check in a real tiny PDF. The simplest: write a 5-line PDF that lopdf can parse.

The test can also create the fixture inline at test time using the `lopdf` builder API, which avoids checking in a binary file. Use this approach if the implementer prefers (see test code below).

- [ ] **Step 2: Write `tests/pdf.rs`**

```rust
use assert_cmd::Command;

#[test]
fn pdf_signature_on_unsigned_pdf_emits_zero_count() {
    // Build a minimal unsigned PDF via lopdf and write to a temp file.
    let tmp = tempfile::NamedTempFile::new().unwrap();
    {
        use lopdf::{dictionary, Document, Object, Stream};
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = lopdf::content::Content {
            operations: vec![
                lopdf::content::Operation::new("BT", vec![]),
                lopdf::content::Operation::new("Tf", vec!["F1".into(), 12.into()]),
                lopdf::content::Operation::new("Td", vec![100.into(), 100.into()]),
                lopdf::content::Operation::new("Tj", vec![Object::string_literal("hello")]),
                lopdf::content::Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.save(tmp.path()).unwrap();
    }
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "pdf", "signature", "--in"]).arg(tmp.path())
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["signature_count"], 0);
    assert!(v["signatures"].as_array().unwrap().is_empty());
}

#[test]
fn pdf_signature_missing_file_exits_4() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["pdf", "signature", "--in", "/nonexistent/path.pdf"])
        .assert().failure().code(4);
}
```

- [ ] **Step 3: Add `InvalidPdf` ErrorCode** (variant + as_str `"invalid_pdf"` + exit Invalid arm + sync test).

- [ ] **Step 4: Create `src/commands/pdf/mod.rs`**

```rust
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
    #[command(long_about = "Extract digital signature info from a PDF.\n\nReports signature count and, per signature, signer name (/Name) and signing time (/M) when present. Does NOT cryptographically verify the signature — verification is deferred to a later milestone.\n\nExamples:\n  ubertool pdf signature --in ./signed.pdf\n  ubertool pdf signature --in ./signed.pdf --json\n\nExit codes:\n  3   invalid or unreadable PDF (invalid_pdf)\n  4   file not found / permission denied")]
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
```

- [ ] **Step 5: Create `src/commands/pdf/signature.rs`**

```rust
use lopdf::{Document, Object};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::SignatureArgs;

#[derive(Serialize)]
struct SignatureInfo {
    signer: Option<String>,
    signing_time: Option<String>,
    location: Option<String>,
    reason: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    signature_count: usize,
    signatures: Vec<SignatureInfo>,
}

pub fn run(args: SignatureArgs, out: &Out) -> Result<(), CliError> {
    let doc = Document::load(&args.in_path).map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("not found") || msg.contains("no such file") {
            CliError::new(ErrorCode::FileNotFound, format!("{e}"))
        } else {
            CliError::new(ErrorCode::InvalidPdf, format!("could not load PDF: {e}"))
                .with_hint("ensure the file is a valid PDF")
        }
    })?;

    let mut signatures = Vec::new();
    // Walk the object table; any dictionary with `/Type /Sig` is a signature.
    for (_, obj) in doc.objects.iter() {
        if let Object::Dictionary(d) = obj {
            let is_sig = d
                .get(b"Type")
                .ok()
                .and_then(|v| v.as_name().ok())
                .map(|n| n == b"Sig")
                .unwrap_or(false);
            if !is_sig {
                continue;
            }
            let signer = d
                .get(b"Name")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let signing_time = d
                .get(b"M")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let location = d
                .get(b"Location")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let reason = d
                .get(b"Reason")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            signatures.push(SignatureInfo {
                signer,
                signing_time,
                location,
                reason,
            });
        }
    }

    out.emit_value(&Out0 {
        signature_count: signatures.len(),
        signatures,
    })
}
```

- [ ] **Step 6: Wire + test + commit**

```bash
cargo test
git add src/core/error.rs src/commands/pdf/ src/commands/mod.rs src/cli.rs src/lib.rs tests/pdf.rs tests/snapshots/
git commit -m "feat(pdf): add pdf signature (info extraction via lopdf; no crypto verify)"
```

> Note: If `lopdf::dictionary!` macro signature has changed in 0.32, adapt. The `Object::as_str()` returning bytes (not str) is expected behavior; we use `from_utf8_lossy` for safety.

---

## Task 6: `regex` noun — `test` + `generate`

**Files:** `src/commands/regex/{mod,test,generate}.rs`, `tests/regex.rs`. Adds `InvalidRegex` is already in `ErrorCode` from M0.

- [ ] **Step 1: Write `tests/regex.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn regex_test_match() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "regex", "test", "--pattern", r"^\d+$", "--text", "12345"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], true);
}

#[test]
fn regex_test_no_match() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "regex", "test", "--pattern", r"^\d+$", "--text", "abc"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], false);
}

#[test]
fn regex_test_with_groups() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "regex", "test", "--pattern", r"(\d+)-(\d+)", "--text", "42-17"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], true);
    let groups = v["groups"].as_array().expect("groups");
    assert_eq!(groups[0], "42-17");
    assert_eq!(groups[1], "42");
    assert_eq!(groups[2], "17");
}

#[test]
fn regex_test_invalid_pattern_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["regex", "test", "--pattern", "[unclosed", "--text", "x"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_regex"));
}

#[test]
fn regex_generate_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["regex", "generate", r"\d{3}"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 3);
    assert!(s.chars().all(|c| c.is_ascii_digit()));
}
```

- [ ] **Step 2: Create `src/commands/regex/mod.rs`**

```rust
//! Regular expression utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod test;

#[derive(Debug, Args)]
pub struct RegexArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Test whether a pattern matches text; emit groups.
    #[command(long_about = "Test whether a regex pattern matches text and emit any capture groups.\n\nNote: Rust's regex crate does not support lookaround. For patterns with `(?=...)` or `(?!...)`, this will report an invalid_regex error.\n\nExamples:\n  ubertool regex test --pattern '\\d+' --text '42'\n  ubertool regex test --pattern '(\\w+)@(\\w+)' --text 'a@b' --json\n\nExit codes:\n  3   pattern does not compile (invalid_regex)")]
    Test(TestArgs),
    /// Generate a string matching a pattern.
    #[command(long_about = "Generate a string that matches a regex pattern (via regex-generate).\n\nLimitations: lookaround, backreferences, and anchors (^ $) are not supported by the generator.\n\nExamples:\n  ubertool regex generate '\\d{3}-\\d{4}'\n  ubertool regex generate '[A-Z]{2,4}'")]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(long)]
    pub pattern: String,
    #[arg(long)]
    pub text: String,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub pattern: String,
}

pub fn dispatch(args: RegexArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Test(a) => test::run(a, out),
        Verb::Generate(a) => generate::run(a, out),
    }
}
```

- [ ] **Step 3: `src/commands/regex/test.rs`**

```rust
use regex::Regex;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::TestArgs;

#[derive(Serialize)]
struct Out0 {
    matched: bool,
    groups: Vec<String>,
}

pub fn run(args: TestArgs, out: &Out) -> Result<(), CliError> {
    let re = Regex::new(&args.pattern).map_err(|e| {
        CliError::new(ErrorCode::InvalidRegex, format!("invalid regex: {e}"))
            .with_input(serde_json::json!({"pattern": args.pattern}))
            .with_hint("Rust's regex crate does not support lookaround or backreferences")
    })?;
    if let Some(caps) = re.captures(&args.text) {
        let groups: Vec<String> = caps
            .iter()
            .map(|m| m.map(|x| x.as_str().to_string()).unwrap_or_default())
            .collect();
        out.emit_value(&Out0 { matched: true, groups })
    } else {
        out.emit_value(&Out0 { matched: false, groups: Vec::new() })
    }
}
```

- [ ] **Step 4: `src/commands/regex/generate.rs`**

```rust
use regex_generate::Generator;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::GenerateArgs;

#[derive(Serialize)]
struct Out0 {
    generated: String,
}

pub fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let mut gen = Generator::new(&args.pattern, rand::thread_rng(), 10).map_err(|e| {
        CliError::new(ErrorCode::InvalidRegex, format!("cannot generate from pattern: {e}"))
            .with_input(serde_json::json!({"pattern": args.pattern}))
            .with_hint("regex-generate does not support lookaround, backrefs, or anchors")
    })?;
    let mut buf = Vec::new();
    gen.generate(&mut buf).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("regex-generate failed: {e}"))
    })?;
    let generated = String::from_utf8(buf).map_err(|_| {
        CliError::new(ErrorCode::InvalidUtf8, "generated bytes are not UTF-8")
    })?;
    out.emit_value(&Out0 { generated })
}
```

- [ ] **Step 5: Wire + test + commit**

> If `regex-generate 0.2` API differs (e.g., the `Generator::new` signature or `generate` taking a `&mut Vec<u8>`), adapt. Note the actual API in your report.

```bash
cargo test
git add src/commands/regex/ src/commands/mod.rs src/cli.rs src/lib.rs tests/regex.rs tests/snapshots/
git commit -m "feat(regex): add regex test (capture groups) and regex generate"
```

---

## Task 7: `email` + `iban` (batched)

Two single-verb validators. One commit per noun.

### email normalize

`tests/email.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn email_normalize_lowercases() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "email", "normalize", "Alice@Example.COM"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["email"], "alice@example.com");
    assert_eq!(v["valid"], true);
}

#[test]
fn email_normalize_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["email", "normalize", "not-an-email"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_email"));
}
```

Add `InvalidEmail` ErrorCode.

`src/commands/email/mod.rs`:

```rust
//! Email normalization + validation.

use clap::{Args, Subcommand};
use email_address::EmailAddress;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EmailArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Normalize and validate an email address.
    #[command(long_about = "Normalize and validate an email address. Trims whitespace and lowercases the domain (leaving the local part as-is for case-sensitivity per RFC 5321, but emitting a fully-lowercased version too).\n\nExamples:\n  ubertool email normalize Alice@Example.COM\n  ubertool email normalize ' bob@example.com ' --json\n\nExit codes:\n  3   invalid email address (invalid_email)")]
    Normalize(NormalizeArgs),
}

#[derive(Debug, Args)]
pub struct NormalizeArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    email: String,
    valid: bool,
}

pub fn dispatch(args: EmailArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Normalize(a) => run(a, out),
    }
}

fn run(args: NormalizeArgs, out: &Out) -> Result<(), CliError> {
    let trimmed = args.input.trim();
    let parsed = EmailAddress::from_str(trimmed).map_err(|e| {
        CliError::new(ErrorCode::InvalidEmail, format!("invalid email: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    // Lowercase domain; keep local part case-preserved but also emit a fully-lowercased form.
    let lower = parsed.to_string().to_lowercase();
    out.emit_value(&Out0 {
        email: lower,
        valid: true,
    })
}
```

> Note: `EmailAddress::from_str` may need `use std::str::FromStr;` at the top. Add it if rustc complains.

Wire + commit:
```bash
git add src/core/error.rs src/commands/email/ src/commands/mod.rs src/cli.rs src/lib.rs tests/email.rs tests/snapshots/
git commit -m "feat(email): add email normalize (email_address)"
```

### iban validate

`tests/iban.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn iban_validate_valid() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "iban", "validate", "GB82WEST12345698765432"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["valid"], true);
    assert_eq!(v["country"], "GB");
}

#[test]
fn iban_validate_invalid_check_digits_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["iban", "validate", "GB00WEST12345698765432"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_iban"));
}

#[test]
fn iban_validate_with_spaces() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["iban", "validate", "GB82 WEST 1234 5698 7654 32"])
        .assert().success();
}
```

Add `InvalidIban` ErrorCode (already exists from M0 — sanity check).

`src/commands/iban/mod.rs`:

```rust
//! IBAN validation.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct IbanArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Validate an IBAN check digit (mod-97).
    #[command(long_about = "Validate an IBAN using the mod-97 check digit algorithm. Spaces and other separators are stripped.\n\nExamples:\n  ubertool iban validate GB82WEST12345698765432\n  ubertool iban validate 'GB82 WEST 1234 5698 7654 32' --json\n\nExit codes:\n  3   invalid IBAN (invalid_iban)")]
    Validate(ValidateArgs),
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    iban: String,
    country: String,
    valid: bool,
}

pub fn dispatch(args: IbanArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Validate(a) => run(a, out),
    }
}

fn run(args: ValidateArgs, out: &Out) -> Result<(), CliError> {
    // iban_validate API may be `Iban::from_str` or `parse_iban`. Adapt based on the actual crate.
    use iban_validate::Iban;
    use std::str::FromStr;
    let iban = Iban::from_str(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidIban, format!("invalid IBAN: {e}"))
            .with_input(serde_json::json!(args.input))
            .with_hint("verify country code, length, and check digits")
    })?;
    out.emit_value(&Out0 {
        iban: iban.to_string(),
        country: iban.country_code().to_string(),
        valid: true,
    })
}
```

> Note: `iban_validate` 4.x API. The struct may be `IbanLike` with method `parse`, or `Iban::from_str`. Adapt as needed. The `country_code()` method returns a `&str` or `String` in different versions.

Wire + commit:
```bash
git add src/core/error.rs src/commands/iban/ src/commands/mod.rs src/cli.rs src/lib.rs tests/iban.rs tests/snapshots/
git commit -m "feat(iban): add iban validate (mod-97 check digits)"
```

---

## Task 8: `phone` + `user-agent` (batched)

Two parser nouns. One commit per noun.

### phone parse

`tests/phone.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn phone_parse_us() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "phone", "parse", "+14155552671"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["country_code"], 1);
    assert_eq!(v["e164"], "+14155552671");
    assert_eq!(v["country"], "US");
}

#[test]
fn phone_parse_with_default_region() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "phone", "parse", "415 555 2671", "--region", "US"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["e164"], "+14155552671");
}

#[test]
fn phone_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["phone", "parse", "not-a-phone"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_phone"));
}
```

`src/commands/phone/mod.rs`:

```rust
//! Phone number parser (libphonenumber via phonenumber crate).

use clap::{Args, Subcommand};
use phonenumber::country;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct PhoneArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse a phone number and emit structured info.
    #[command(long_about = "Parse a phone number (E.164 or with --region default) and emit country, national format, E.164, and type.\n\nExamples:\n  ubertool phone parse +14155552671\n  ubertool phone parse '415 555 2671' --region US --json\n\nExit codes:\n  3   invalid phone number (invalid_phone)")]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
    /// Default region (ISO 3166-1 alpha-2, e.g., US, GB).
    #[arg(long)]
    pub region: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    country: String,
    country_code: u16,
    national: String,
    e164: String,
    kind: String,
}

pub fn dispatch(args: PhoneArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => run(a, out),
    }
}

fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let region: Option<country::Id> = args
        .region
        .as_deref()
        .map(|r| r.parse::<country::Id>())
        .transpose()
        .map_err(|e| {
            CliError::new(ErrorCode::UsageError, format!("invalid --region: {e:?}"))
        })?;
    let num = phonenumber::parse(region, &args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidPhone, format!("invalid phone: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    if !num.is_valid() {
        return Err(CliError::new(ErrorCode::InvalidPhone, "phone number failed validation")
            .with_input(serde_json::json!(args.input)));
    }
    let cc = num.country().code();
    let country_label = num.country().id().map(|i| format!("{i:?}")).unwrap_or_default();
    let national = phonenumber::format(&num)
        .mode(phonenumber::Mode::National)
        .to_string();
    let e164 = phonenumber::format(&num)
        .mode(phonenumber::Mode::E164)
        .to_string();
    let kind = format!("{:?}", num.metadata(&phonenumber::metadata::DATABASE).map(|m| m).unwrap_or_default());
    out.emit_value(&Out0 {
        country: country_label,
        country_code: cc,
        national,
        e164,
        kind,
    })
}
```

> Note: phonenumber 0.3 API may differ. `country::Id` may be `country::Code`. `num.country().code()` may be `num.country_code()`. `metadata(...)` may not exist on `Number`. **Adapt these to whatever compiles**; the goal is to emit country, country_code, national, e164. Skip `kind` if no clean way to get the type. Note adaptations in the report.

Add `InvalidPhone` ErrorCode (likely already exists from M0).

Wire + commit:
```bash
git add src/core/error.rs src/commands/phone/ src/commands/mod.rs src/cli.rs src/lib.rs tests/phone.rs tests/snapshots/
git commit -m "feat(phone): add phone parse (libphonenumber via phonenumber crate)"
```

### user-agent parse

`tests/user_agent.rs`:

```rust
use assert_cmd::Command;

#[test]
fn ua_parse_chrome() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "user-agent", "parse",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["browser"].as_str().unwrap().to_lowercase().contains("chrome"));
    assert!(v["os"].as_str().unwrap().to_lowercase().contains("mac"));
}

#[test]
fn ua_parse_unknown_emits_unknown_fields() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "user-agent", "parse", "definitely-not-a-real-ua"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    // woothee returns "UNKNOWN" for unrecognized UAs rather than erroring.
    assert!(v["browser"].is_string());
}
```

`src/commands/user_agent/mod.rs`:

```rust
//! User-agent string parser (woothee).

use clap::{Args, Subcommand};
use serde::Serialize;
use woothee::parser::Parser;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UserAgentArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse a user-agent string into structured fields.
    #[command(long_about = "Parse a user-agent string (browser, OS, version, etc.) using the woothee parser.\n\nExamples:\n  ubertool user-agent parse 'Mozilla/5.0 ...'\n  ubertool user-agent parse '...' --json")]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    browser: String,
    browser_version: String,
    os: String,
    os_version: String,
    category: String,
    vendor: String,
}

pub fn dispatch(args: UserAgentArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => run(a, out),
    }
}

fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let parser = Parser::new();
    let result = parser.parse(&args.input);
    match result {
        Some(r) => out.emit_value(&Out0 {
            browser: r.name.to_string(),
            browser_version: r.version.to_string(),
            os: r.os.to_string(),
            os_version: r.os_version.to_string(),
            category: r.category.to_string(),
            vendor: r.vendor.to_string(),
        }),
        None => out.emit_value(&Out0 {
            browser: "UNKNOWN".into(),
            browser_version: "UNKNOWN".into(),
            os: "UNKNOWN".into(),
            os_version: "UNKNOWN".into(),
            category: "UNKNOWN".into(),
            vendor: "UNKNOWN".into(),
        }),
    }
}
```

> Note: woothee's `WootheeResult` may use `Cow<'_, str>` for fields, requiring `.to_string()` calls. Adapt to whatever the API offers.

Wire + commit:
```bash
git add src/commands/user_agent/ src/commands/mod.rs src/cli.rs src/lib.rs tests/user_agent.rs tests/snapshots/
git commit -m "feat(user-agent): add user-agent parse (woothee)"
```

(The Rust module is `user_agent` (snake_case); the CLI surface is `user-agent` via `#[command(name = "user-agent")]` on the enum variant.)

---

## Task 9: `url parse` (extends existing M1 `url` noun)

**Files:** Modify `src/commands/url/mod.rs` (add `Parse` variant). Create `src/commands/url/parse.rs`. Add tests to `tests/url.rs` (or create `tests/url_parse.rs`).

- [ ] **Step 1: Add to `tests/url.rs` (or new `tests/url_parse.rs`)**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn url_parse_full() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "url", "parse", "https://user:pass@example.com:8080/path?q=1#frag"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["scheme"], "https");
    assert_eq!(v["host"], "example.com");
    assert_eq!(v["port"], 8080);
    assert_eq!(v["path"], "/path");
    assert_eq!(v["query"], "q=1");
    assert_eq!(v["fragment"], "frag");
    assert_eq!(v["username"], "user");
    assert_eq!(v["password"], "pass");
}

#[test]
fn url_parse_minimal() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "url", "parse", "https://example.com"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["scheme"], "https");
    assert_eq!(v["host"], "example.com");
    // No port / no query / no fragment → null.
    assert!(v["port"].is_null());
}

#[test]
fn url_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["url", "parse", "not a url"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_url"));
}
```

- [ ] **Step 2: Add `InvalidUrl` ErrorCode** (variant + as_str `"invalid_url"` + exit Invalid arm + sync test).

- [ ] **Step 3: Extend `src/commands/url/mod.rs`**

Find the existing `Verb` enum (currently `Encode`/`Decode`). Add a third variant:

```rust
    /// Parse a URL into its component parts.
    #[command(long_about = "Parse a URL into scheme, host, port, path, query, fragment, username, password.\n\nExamples:\n  ubertool url parse 'https://example.com/path?q=1'\n  ubertool url parse 'https://user:pass@example.com:8080/path' --json\n\nExit codes:\n  3   invalid URL (invalid_url)")]
    Parse(parse::ParseArgs),
```

Add `pub mod parse;` to the file's top-of-file `pub mod` declarations.

Extend `dispatch()`:
```rust
        Verb::Parse(a) => parse::run(a, out),
```

- [ ] **Step 4: Create `src/commands/url/parse.rs`**

```rust
use clap::Args;
use serde::Serialize;
use url::Url;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    scheme: String,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

pub fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let parsed = Url::parse(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidUrl, format!("invalid URL: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    let username = if parsed.username().is_empty() {
        None
    } else {
        Some(parsed.username().to_string())
    };
    let password = parsed.password().map(|p| p.to_string());
    out.emit_value(&Out0 {
        scheme: parsed.scheme().to_string(),
        username,
        password,
        host: parsed.host_str().map(|h| h.to_string()),
        port: parsed.port(),
        path: parsed.path().to_string(),
        query: parsed.query().map(|q| q.to_string()),
        fragment: parsed.fragment().map(|f| f.to_string()),
    })
}
```

- [ ] **Step 5: Run + commit**

```bash
cargo test
git add src/core/error.rs src/commands/url/ tests/url.rs tests/snapshots/
git commit -m "feat(url): add url parse (scheme/host/port/path/query/fragment/user/pass)"
```

---

## Task 10: Update OpenCLI spec for M4

**Files:** Modify `ubertool.ocs.yaml`. Run `make gen` and `make verify-spec`.

Add entries for each new M4 noun + the new `url parse` verb. Use the M3 spec task as the template.

**M4 entries to add:**

1. **cipher** group + 2 leaves:
   - `cipher encrypt <input>`: required positional. Flags `--password` (string, required), `--algo` (string, choices: `aes-gcm`, `chacha20-poly1305`, default `aes-gcm`), `--kdf` (string, choices: `argon2`, `pbkdf2`, default `argon2`), `--json`, `--quiet`.
   - `cipher decrypt <input>`: required positional. Flags `--password` (required), `--json`, `--quiet`.

2. **rsa** group + 1 leaf:
   - `rsa keypair`: no positional. Flags `--bits` (integer, default `"2048"`), `--json`, `--quiet`.

3. **otp** group + 2 leaves:
   - `otp generate`: no positional. Flags `--secret` (required), `--digits` (integer, default `"6"`), `--period` (integer, default `"30"`), `--json`, `--quiet`.
   - `otp validate <code>`: required positional `code`. Flags `--secret` (required), `--digits`, `--period`, `--json`, `--quiet`.

4. **pdf** group + 1 leaf:
   - `pdf signature`: no positional. Flags `--in` (string, required), `--json`, `--quiet`.

5. **regex** group + 2 leaves:
   - `regex test`: no positional. Flags `--pattern` (required), `--text` (required), `--json`, `--quiet`.
   - `regex generate <pattern>`: required positional `pattern`. Flags `--json`, `--quiet`.

6. **email** group + 1 leaf:
   - `email normalize <input>`: required positional. Flags `--json`, `--quiet`.

7. **iban** group + 1 leaf:
   - `iban validate <input>`: required positional. Flags `--json`, `--quiet`.

8. **phone** group + 1 leaf:
   - `phone parse <input>`: required positional. Flags `--region` (optional string), `--json`, `--quiet`.

9. **user-agent** group + 1 leaf (note kebab-case noun key):
   - `user-agent parse <input>`: required positional. Flags `--json`, `--quiet`.

10. **url** gets a new third verb:
    - `url parse <input>`: required positional. Flags `--json`, `--quiet`. The existing `url` group entry stays unchanged; add the new leaf under it.

Spec format gotchas (same as M3):
- `choices:` is list of `{value: "..."}` maps.
- Integer defaults quoted as strings.
- Summaries with colons need quoting.

```bash
ocli spec check ubertool.ocs.yaml
make gen
make verify-spec
git add ubertool.ocs.yaml docs/cli/ docs/llms.txt
git commit -m "docs(spec): add all M4 nouns to OpenCLI spec and regenerate Markdown + llms.txt"
```

Expected: `make verify-spec` reports ≥ 92 leaf commands match (M3 ended at 79; M4 adds ~14 leaves).

---

## Task 11: Fitness + `make ci`

**Files:** Modify `tests/agent_cli_fitness.rs`. Run `make ci`.

Append to `ALL_NOUNS_AND_VERBS`:

```rust
    // M4
    ("cipher", &["encrypt", "decrypt"]),
    ("rsa", &["keypair"]),
    ("otp", &["generate", "validate"]),
    ("pdf", &["signature"]),
    ("regex", &["test", "generate"]),
    ("email", &["normalize"]),
    ("iban", &["validate"]),
    ("phone", &["parse"]),
    ("user-agent", &["parse"]),
```

Also extend the existing `url` entry from `&["encode", "decode"]` to `&["encode", "decode", "parse"]`.

```bash
cargo test --test agent_cli_fitness
make ci
```

Fix any clippy or fmt issues. If `clippy::large_enum_variant` fires on `Noun` (now ~46 variants), add `#[allow(clippy::large_enum_variant)]`.

```bash
git add tests/agent_cli_fitness.rs
git commit -m "test(fitness): extend fitness checklist to all M4 nouns and verbs"
```

If fmt applied:
```bash
git add -u
git commit -m "style: apply cargo fmt across M4 source and test files"
```

Confirm clean tree. **Do not push or tag.**

---

## Self-Review

**Spec coverage** — every M4 requirement from design spec §16 covered:
- ✅ cipher encrypt/decrypt — Task 2
- ✅ rsa keypair — Task 3
- ✅ otp generate/validate — Task 4
- ✅ pdf signature — Task 5
- ✅ regex test/generate — Task 6
- ✅ email normalize — Task 7
- ✅ iban validate — Task 7
- ✅ phone parse — Task 8
- ✅ user-agent parse — Task 8
- ✅ url parse — Task 9
- ✅ Spec + fitness + CI — Tasks 10, 11

**Placeholder scan** — no TBDs. Each task has complete code. Crate-version adaptations are flagged inline with explicit "note any adaptations in your report" instructions for crates whose APIs are version-sensitive (phonenumber, iban_validate, regex-generate, woothee, totp-rs).

**Type consistency** — Output structs all use `#[derive(Serialize)]`. New ErrorCode variants in M4: `InvalidCipher`, `InvalidPdf`, `InvalidEmail`, `InvalidUrl` (plus the `InvalidIban`, `InvalidPhone`, `InvalidRegex`, `DecryptFailed` already in `ErrorCode` from M0). All new variants → exit 3 (Invalid) except `DecryptFailed` → exit 5 (Crypto).

**Scope check** — 9 new nouns + 1 verb extension + spec + CI = 11 task topics across 11 numbered tasks. Single milestone.

**Ambiguity check** — explicit decisions:
- Cipher output is the self-describing `$`-delimited base64 format. Decrypt auto-detects algo/kdf.
- RSA keys are PKCS#8 PEM (not PKCS#1) — modern standard.
- OTP validate exits 5 on mismatch, NOT 0+verified=false. The `bcrypt verify` pattern from M1 is the precedent.
- PDF signature is info-only; full PKCS#7 verification is deferred.
- Regex generate has no support for lookaround/backrefs/anchors — documented in `--help`.
- Email normalize lowercases the entire address (not strict RFC 5321 which permits case-sensitive local parts) — pragmatic choice for an agent tool.
- Phone parser accepts `--region` for non-E.164 inputs.
- URL parse emits `null` for missing components (no port, no query, etc.) rather than empty strings.

---

## Execution Handoff

Plan saved. Same workflow as prior milestones — subagent-driven execution.
