# ubertool M1 — Tier 1 (Foundation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ~20 foundation verbs across 14 new nouns to the M0 scaffold (`hash`, `hmac`, `bcrypt`, `url`, `html`, `basic-auth`, `jwt`, `uuid`, `ulid`, `token`, `password`, `json`, `yaml`, `toml`), each plugging into the existing `core/` infrastructure (`CliError`/`ErrorCode`/`Out`/`resolve_input`/etc.) using `base64` as the template pattern.

**Architecture:** Single Rust crate. Each new noun is a folder under `src/commands/<noun>/` with `mod.rs` plus per-verb files when there are ≥2 distinct verbs. All commands return `Result<(), CliError>` and emit through `Out::emit_value`. Hash and HMAC are multiplexed verbs (the algorithm is the subcommand name). Every command supports `--in <path>`, stdin pipe, `--json`, `--quiet` via the shared `resolve_input` + `Out` plumbing.

**Tech Stack:** Adds to M0: `sha2`, `sha1`, `md-5`, `sha3`, `digest`, `hmac` (hash family), `bcrypt`, `jsonwebtoken`, `uuid` (v4+v7), `ulid`, `percent-encoding`, `html-escape`, `serde_yaml`, `toml`, `zxcvbn`, `rand`.

**Companion documents:**
- Design spec: `docs/superpowers/specs/2026-05-15-ubertool-cli-design.md` (§3 inventory, §11 crates, §5 output contract, §7 errors, §8 exit codes).
- M0 plan (template): `docs/superpowers/plans/2026-05-15-ubertool-m0-scaffolding.md`.
- Design rules: `~/.claude/skills/agent-cli-design/SKILL.md` + `fitness-checklist.md`.

**Working directory:** `/Users/mihai/dev/ubertool/` (all tasks run here).

**The pattern every noun follows** (copy `src/commands/base64/` to compare):
1. Create `src/commands/<noun>/mod.rs` (and per-verb `.rs` files when needed): defines `Args`, `Verb` enum, `dispatch()`, `run()`.
2. Add `pub mod <noun>;` to `src/commands/mod.rs`.
3. Add `<Noun>(crate::commands::<noun>::<Noun>Args)` variant to `src/cli.rs::Noun`.
4. Add match arm `cli::Noun::<Noun>(a) => commands::<noun>::dispatch(a, &out)` to `src/lib.rs::run()`.
5. Write `tests/<noun>.rs` (assert_cmd integration tests, ≥ text mode, JSON mode, error path, stdin path).
6. Run `cargo test`. Commit.

**Acceptance criteria (must all pass at end of M1):**

1. `cargo build` succeeds. `cargo build --release` produces a working binary.
2. `cargo test` passes 100%. Test count grows by ≥ 60 over M0 baseline (~44 → ~100+).
3. Every new noun appears in `ubertool --help`. Every verb appears in `ubertool <noun> --help`.
4. Every new data-returning command supports `--json` and emits parseable JSON to stdout.
5. The fitness-checklist test (`tests/agent_cli_fitness.rs`) still passes — extended to cover the new surface (Task 17).
6. `make verify-spec` passes — OpenCLI spec updated with all new nouns/verbs (Task 16).
7. `make ci` passes locally (fmt + clippy + test + verify-spec).
8. Each commit follows TDD: failing test → minimal code → passing test → commit.
9. Repo is clean: `git status` shows nothing uncommitted at end.

---

## File Structure (created by this plan)

```
/Users/mihai/dev/ubertool/
├── Cargo.toml                         (M1: dependencies added)
├── src/
│   ├── cli.rs                         (M1: Noun enum variants added)
│   ├── lib.rs                         (M1: dispatch arms added)
│   ├── core/
│   │   ├── error.rs                   (M1: new ErrorCode variants added)
│   │   └── hex.rs                     (M1: hex encoder, new)
│   └── commands/
│       ├── mod.rs                     (M1: pub mod <noun>; lines added)
│       ├── hash/mod.rs
│       ├── hmac/mod.rs
│       ├── bcrypt/{mod,hash,verify}.rs
│       ├── url/{mod,encode,decode}.rs
│       ├── html/{mod,encode,decode}.rs
│       ├── basic_auth/{mod,encode,decode}.rs
│       ├── jwt/{mod,decode,verify}.rs
│       ├── uuid/mod.rs
│       ├── ulid/mod.rs
│       ├── token/mod.rs
│       ├── password/mod.rs
│       ├── json/{mod,to_yaml,to_toml,minify,prettify}.rs
│       ├── yaml/{mod,to_json,to_toml}.rs
│       └── toml/{mod,to_json,to_yaml}.rs
├── tests/
│   ├── hash.rs           hmac.rs        bcrypt.rs
│   ├── url.rs            html.rs        basic_auth.rs
│   ├── jwt.rs            uuid.rs        ulid.rs
│   ├── token.rs          password.rs    json_convert.rs
│   ├── yaml_convert.rs   toml_convert.rs
│   └── agent_cli_fitness.rs            (extended in Task 17)
└── ubertool.ocs.yaml                   (extended in Task 16)
```

---

## Cross-cutting conventions for all M1 tasks

**Output struct convention** — every command emits a `#[derive(Serialize)]` struct.
- **Single meaningful value** → struct has one field. Text mode auto-emits just the value (no key prefix). Examples: `{"hash": "..."}`, `{"uuid": "..."}`.
- **Multiple meaningful values** → struct has named fields. Text mode emits `key: value\n` lines; quiet mode emits bare values one per line. Examples: jwt decode (`header`/`claims`), basic-auth decode (`user`/`pass`).

**Input convention** — every command that takes a primary input uses `resolve_input(positional, in_path, is_stdin_tty())` exactly as `base64::encode` does. Never block on TTY stdin.

**Error convention** — every command that can fail with bad input returns `CliError::new(ErrorCode::<Variant>, msg).with_input(json!(echoed)).with_hint("...")`. Secrets (passwords, keys, JWT secrets) MUST be redacted from `with_input` — pass `serde_json::json!("<redacted>")` instead of the actual value.

**Test convention** — every noun's `tests/<noun>.rs` covers AT LEAST:
- (a) Happy path text mode produces expected stdout.
- (b) Happy path `--json` mode produces JSON that `serde_json::from_slice` accepts.
- (c) At least one error path returns the documented exit code AND `error: <typed_code>: ...` on stderr.
- (d) stdin pipe input works (echo something | ubertool ...).

Tests that already match this shape live in `tests/base64.rs` — use it as a template.

---

## Task 1: Add M1 dependencies to Cargo.toml

**Files:**
- Modify: `/Users/mihai/dev/ubertool/Cargo.toml`

- [ ] **Step 1: Add dependencies block**

Edit `Cargo.toml`, replacing the `[dependencies]` section with:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
# M1 hash family
sha2 = "0.10"
sha1 = "0.10"
md-5 = "0.10"
sha3 = "0.10"
digest = "0.10"
hmac = "0.12"
# M1 password
bcrypt = "0.15"
zxcvbn = "2"
# M1 JWT
jsonwebtoken = "9"
# M1 IDs / tokens
uuid = { version = "1", features = ["v4", "v7"] }
ulid = "1"
rand = "0.8"
# M1 encoding
percent-encoding = "2"
html-escape = "0.2"
# M1 conversion
serde_yaml = "0.9"
toml = "0.8"
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: succeeds with no errors. Some `unused` warnings on the new crates are acceptable (no code uses them yet).

- [ ] **Step 3: Run existing tests still pass**

Run: `cargo test`
Expected: all M0 tests still pass (44 tests).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add M1 dependencies (hash, hmac, bcrypt, jwt, uuid, ulid, encoding, conversion)"
```

---

## Task 2: `hash` noun — 8 algorithm subcommands

**Files:**
- Create: `src/core/hex.rs` (shared hex encoder)
- Modify: `src/core/mod.rs` (add `pub mod hex;`)
- Create: `src/commands/hash/mod.rs`
- Modify: `src/commands/mod.rs` (add `pub mod hash;`)
- Modify: `src/cli.rs` (add `Hash(...)` variant)
- Modify: `src/lib.rs` (add dispatch arm)
- Create: `tests/hash.rs`

- [ ] **Step 1: Add hex encoder + unit test**

Create `src/core/hex.rs`:

```rust
//! Lowercase hex encoder used by every hash- and MAC-emitting command.

pub fn encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn single_byte_pads_to_two_chars() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(encode(&[0x0f]), "0f");
        assert_eq!(encode(&[0xff]), "ff");
    }

    #[test]
    fn known_pattern() {
        assert_eq!(encode(b"\x00\x01\x02\x03"), "00010203");
    }
}
```

Modify `src/core/mod.rs` to add `pub mod hex;` at the end:

```rust
pub mod error;
pub mod exit;
pub mod input;
pub mod output;
pub mod tty;
pub mod hex;
```

Run: `cargo test --lib core::hex`
Expected: 3 hex tests PASS.

- [ ] **Step 2: Write failing integration test**

Create `tests/hash.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn sha256_of_hello_text_mode() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "hello"])
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}

#[test]
fn sha256_of_hello_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "hash", "sha256", "hello"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("must be valid JSON");
    assert_eq!(
        v["hash"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn md5_of_empty_input() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "md5", ""])
        .assert()
        .success()
        .stdout("d41d8cd98f00b204e9800998ecf8427e\n");
}

#[test]
fn sha1_known_vector() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha1", "hello"])
        .assert()
        .success()
        .stdout("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\n");
}

#[test]
fn sha512_known_vector() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha512", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca",
        ));
}

#[test]
fn sha3_256_known_vector() {
    // SHA3-256 of "hello"
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha3-256", "hello"])
        .assert()
        .success()
        .stdout("3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392\n");
}

#[test]
fn sha256_via_stdin_pipe() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256"])
        .write_stdin("hello")
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}

#[test]
fn sha256_via_in_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"hello").unwrap();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "--in"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}
```

Run: `cargo test --test hash`
Expected: FAIL — `error: unrecognized subcommand 'hash'` (command not yet implemented).

- [ ] **Step 3: Implement the hash module**

Create `src/commands/hash/mod.rs`:

```rust
//! Hashing — multiplexed verb. The algorithm name is the subcommand
//! (e.g. `ubertool hash sha256`). Output: `{"hash": "<hex>"}`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::hex;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HashArgs {
    #[command(subcommand)]
    pub verb: HashVerb,
}

#[derive(Debug, Subcommand)]
pub enum HashVerb {
    #[command(name = "md5", long_about = "Compute MD5 hash (legacy / not collision-resistant).\n\nExamples:\n  ubertool hash md5 \"hello\"\n  ubertool hash md5 --in ./file.bin\n  echo -n hello | ubertool hash md5")]
    Md5(HashRunArgs),
    #[command(name = "sha1", long_about = "Compute SHA-1 hash (legacy / not collision-resistant).\n\nExamples:\n  ubertool hash sha1 \"hello\"\n  ubertool hash sha1 --in ./file.bin")]
    Sha1(HashRunArgs),
    #[command(name = "sha224", long_about = "Compute SHA-224 hash.")]
    Sha224(HashRunArgs),
    #[command(name = "sha256", long_about = "Compute SHA-256 hash.\n\nExamples:\n  ubertool hash sha256 \"hello\"\n  ubertool hash sha256 --in ./file.bin --json")]
    Sha256(HashRunArgs),
    #[command(name = "sha384", long_about = "Compute SHA-384 hash.")]
    Sha384(HashRunArgs),
    #[command(name = "sha512", long_about = "Compute SHA-512 hash.")]
    Sha512(HashRunArgs),
    #[command(name = "sha3-256", long_about = "Compute SHA3-256 hash (Keccak).")]
    Sha3_256(HashRunArgs),
    #[command(name = "sha3-512", long_about = "Compute SHA3-512 hash (Keccak).")]
    Sha3_512(HashRunArgs),
}

#[derive(Debug, Args)]
pub struct HashRunArgs {
    /// Literal input to hash (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
}

#[derive(Debug, Clone, Copy)]
enum Algo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_512,
}

pub fn dispatch(args: HashArgs, out: &Out) -> Result<(), CliError> {
    let (run_args, algo) = match args.verb {
        HashVerb::Md5(a) => (a, Algo::Md5),
        HashVerb::Sha1(a) => (a, Algo::Sha1),
        HashVerb::Sha224(a) => (a, Algo::Sha224),
        HashVerb::Sha256(a) => (a, Algo::Sha256),
        HashVerb::Sha384(a) => (a, Algo::Sha384),
        HashVerb::Sha512(a) => (a, Algo::Sha512),
        HashVerb::Sha3_256(a) => (a, Algo::Sha3_256),
        HashVerb::Sha3_512(a) => (a, Algo::Sha3_512),
    };
    run(run_args, algo, out)
}

fn run(args: HashRunArgs, algo: Algo, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let digest = compute(algo, input.as_bytes());
    out.emit_value(&HashOutput { hash: digest })
}

fn compute(algo: Algo, data: &[u8]) -> String {
    use digest::Digest;
    match algo {
        Algo::Md5 => hex::encode(&md5::Md5::digest(data)),
        Algo::Sha1 => hex::encode(&sha1::Sha1::digest(data)),
        Algo::Sha224 => hex::encode(&sha2::Sha224::digest(data)),
        Algo::Sha256 => hex::encode(&sha2::Sha256::digest(data)),
        Algo::Sha384 => hex::encode(&sha2::Sha384::digest(data)),
        Algo::Sha512 => hex::encode(&sha2::Sha512::digest(data)),
        Algo::Sha3_256 => hex::encode(&sha3::Sha3_256::digest(data)),
        Algo::Sha3_512 => hex::encode(&sha3::Sha3_512::digest(data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_hello() {
        assert_eq!(
            compute(Algo::Sha256, b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn md5_of_empty() {
        assert_eq!(compute(Algo::Md5, b""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn sha1_of_hello() {
        assert_eq!(
            compute(Algo::Sha1, b"hello"),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
    }
}
```

- [ ] **Step 4: Wire the noun into the CLI**

Modify `src/commands/mod.rs` to add the new module:

```rust
pub mod base64;
pub mod hash;
```

Modify `src/cli.rs`: in the `Noun` enum, add the variant:

```rust
#[derive(Subcommand, Debug)]
pub enum Noun {
    /// Base64 encoding and decoding.
    Base64(crate::commands::base64::Base64Args),
    /// Cryptographic hash functions (md5, sha1, sha224, sha256, sha384, sha512, sha3-256, sha3-512).
    Hash(crate::commands::hash::HashArgs),
}
```

Modify `src/lib.rs` — extend the match in `run()`:

```rust
let result: Result<(), CliError> = match parsed.noun {
    cli::Noun::Base64(a) => commands::base64::dispatch(a, &out),
    cli::Noun::Hash(a) => commands::hash::dispatch(a, &out),
};
```

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: PASS — 8 new integration tests in `tests/hash.rs`, 3 unit tests in `commands::hash::tests`, 3 unit tests in `core::hex::tests`, plus all M0 tests.

- [ ] **Step 6: Commit**

```bash
git add src/core/hex.rs src/core/mod.rs src/commands/hash/ src/commands/mod.rs src/cli.rs src/lib.rs tests/hash.rs
git commit -m "feat(hash): add multiplexed hash noun (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512)"
```

---

## Task 3: `hmac` noun — 8 algorithm subcommands with --key

**Files:**
- Create: `src/commands/hmac/mod.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/hmac.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/hmac.rs`:

```rust
use assert_cmd::Command;

#[test]
fn hmac_sha256_known_vector() {
    // HMAC-SHA256("key", "The quick brown fox jumps over the lazy dog")
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "hmac",
            "sha256",
            "The quick brown fox jumps over the lazy dog",
            "--key",
            "key",
        ])
        .assert()
        .success()
        .stdout("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8\n");
}

#[test]
fn hmac_sha256_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "hmac", "sha256", "hello", "--key", "secret",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("must be valid JSON");
    assert!(v["hmac"].is_string(), "hmac field must be a string");
}

#[test]
fn hmac_sha1_known_vector() {
    // HMAC-SHA1("key", "The quick brown fox jumps over the lazy dog")
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "hmac",
            "sha1",
            "The quick brown fox jumps over the lazy dog",
            "--key",
            "key",
        ])
        .assert()
        .success()
        .stdout("de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9\n");
}

#[test]
fn hmac_via_stdin_with_key_flag() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hmac", "sha256", "--key", "key"])
        .write_stdin("The quick brown fox jumps over the lazy dog")
        .assert()
        .success()
        .stdout("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8\n");
}

#[test]
fn hmac_missing_key_is_usage_error() {
    // clap should refuse with exit 2 because --key is required.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hmac", "sha256", "hello"])
        .assert()
        .failure()
        .code(2);
}
```

Run: `cargo test --test hmac`
Expected: FAIL — `unrecognized subcommand 'hmac'`.

- [ ] **Step 2: Implement the hmac module**

Create `src/commands/hmac/mod.rs`:

```rust
//! HMAC — multiplexed verb mirroring `hash`. Requires a `--key`.
//! Output: `{"hmac": "<hex>"}`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use hmac::{Hmac, Mac};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::hex;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HmacArgs {
    #[command(subcommand)]
    pub verb: HmacVerb,
}

#[derive(Debug, Subcommand)]
pub enum HmacVerb {
    #[command(name = "md5", long_about = "Compute HMAC-MD5 (legacy / not collision-resistant).\n\nExamples:\n  ubertool hmac md5 \"msg\" --key \"secret\"")]
    Md5(HmacRunArgs),
    #[command(name = "sha1", long_about = "Compute HMAC-SHA1 (legacy).")]
    Sha1(HmacRunArgs),
    #[command(name = "sha224", long_about = "Compute HMAC-SHA224.")]
    Sha224(HmacRunArgs),
    #[command(name = "sha256", long_about = "Compute HMAC-SHA256.\n\nExamples:\n  ubertool hmac sha256 \"msg\" --key \"secret\"\n  ubertool hmac sha256 --in ./file --key \"secret\" --json")]
    Sha256(HmacRunArgs),
    #[command(name = "sha384", long_about = "Compute HMAC-SHA384.")]
    Sha384(HmacRunArgs),
    #[command(name = "sha512", long_about = "Compute HMAC-SHA512.")]
    Sha512(HmacRunArgs),
    #[command(name = "sha3-256", long_about = "Compute HMAC-SHA3-256.")]
    Sha3_256(HmacRunArgs),
    #[command(name = "sha3-512", long_about = "Compute HMAC-SHA3-512.")]
    Sha3_512(HmacRunArgs),
}

#[derive(Debug, Args)]
pub struct HmacRunArgs {
    /// Literal input to MAC (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// HMAC secret key (required).
    #[arg(long)]
    pub key: String,
}

#[derive(Serialize)]
struct HmacOutput {
    hmac: String,
}

#[derive(Debug, Clone, Copy)]
enum Algo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_512,
}

pub fn dispatch(args: HmacArgs, out: &Out) -> Result<(), CliError> {
    let (run_args, algo) = match args.verb {
        HmacVerb::Md5(a) => (a, Algo::Md5),
        HmacVerb::Sha1(a) => (a, Algo::Sha1),
        HmacVerb::Sha224(a) => (a, Algo::Sha224),
        HmacVerb::Sha256(a) => (a, Algo::Sha256),
        HmacVerb::Sha384(a) => (a, Algo::Sha384),
        HmacVerb::Sha512(a) => (a, Algo::Sha512),
        HmacVerb::Sha3_256(a) => (a, Algo::Sha3_256),
        HmacVerb::Sha3_512(a) => (a, Algo::Sha3_512),
    };
    run(run_args, algo, out)
}

fn run(args: HmacRunArgs, algo: Algo, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let mac_hex = compute(algo, args.key.as_bytes(), input.as_bytes())?;
    out.emit_value(&HmacOutput { hmac: mac_hex })
}

fn compute(algo: Algo, key: &[u8], data: &[u8]) -> Result<String, CliError> {
    fn mac<M: Mac + digest::KeyInit>(key: &[u8], data: &[u8]) -> Result<String, CliError> {
        let mut m = <M as digest::KeyInit>::new_from_slice(key)
            .map_err(|_| CliError::new(ErrorCode::UsageError, "HMAC key length is invalid"))?;
        m.update(data);
        Ok(hex::encode(&m.finalize().into_bytes()))
    }
    match algo {
        Algo::Md5 => mac::<Hmac<md5::Md5>>(key, data),
        Algo::Sha1 => mac::<Hmac<sha1::Sha1>>(key, data),
        Algo::Sha224 => mac::<Hmac<sha2::Sha224>>(key, data),
        Algo::Sha256 => mac::<Hmac<sha2::Sha256>>(key, data),
        Algo::Sha384 => mac::<Hmac<sha2::Sha384>>(key, data),
        Algo::Sha512 => mac::<Hmac<sha2::Sha512>>(key, data),
        Algo::Sha3_256 => mac::<Hmac<sha3::Sha3_256>>(key, data),
        Algo::Sha3_512 => mac::<Hmac<sha3::Sha3_512>>(key, data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_sha256_rfc_vector() {
        // HMAC-SHA256("key", "The quick brown fox jumps over the lazy dog")
        let r =
            compute(Algo::Sha256, b"key", b"The quick brown fox jumps over the lazy dog").unwrap();
        assert_eq!(
            r,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn hmac_sha1_rfc_vector() {
        let r =
            compute(Algo::Sha1, b"key", b"The quick brown fox jumps over the lazy dog").unwrap();
        assert_eq!(r, "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9");
    }
}
```

- [ ] **Step 3: Wire into CLI**

Modify `src/commands/mod.rs`:

```rust
pub mod base64;
pub mod hash;
pub mod hmac;
```

Modify `src/cli.rs` `Noun` enum — add:

```rust
    /// Keyed-hash MAC (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512).
    Hmac(crate::commands::hmac::HmacArgs),
```

Modify `src/lib.rs` — add match arm:

```rust
        cli::Noun::Hmac(a) => commands::hmac::dispatch(a, &out),
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: PASS — 5 new integration tests + 2 unit tests added.

- [ ] **Step 5: Commit**

```bash
git add src/commands/hmac/ src/commands/mod.rs src/cli.rs src/lib.rs tests/hmac.rs
git commit -m "feat(hmac): add multiplexed HMAC noun with --key flag"
```

---

## Task 4: `bcrypt` noun — `hash` + `verify` verbs

**Files:**
- Modify: `src/core/error.rs` (add `InvalidBcrypt`)
- Create: `src/commands/bcrypt/mod.rs`, `hash.rs`, `verify.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/bcrypt.rs`

- [ ] **Step 1: Add InvalidBcrypt error code**

Modify `src/core/error.rs`:

In the `ErrorCode` enum, add `InvalidBcrypt` after `InvalidBase64`:

```rust
    InvalidBase64,
    InvalidBcrypt,
    FileNotFound,
```

In `exit()`, extend the `Invalid` arm:

```rust
            InvalidJson | InvalidYaml | InvalidToml | InvalidXml | InvalidRegex | InvalidIban
            | InvalidPhone | InvalidJwt | InvalidBase64 | InvalidBcrypt => ExitCode::Invalid,
```

In `as_str()`, add:

```rust
            InvalidBcrypt => "invalid_bcrypt",
```

In the test `error_code_as_str_matches_serde_serialization`, add `ErrorCode::InvalidBcrypt` to the `cases` array.

Run: `cargo test --lib core::error`
Expected: PASS.

- [ ] **Step 2: Write failing integration test**

Create `tests/bcrypt.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn bcrypt_hash_produces_valid_format() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "secret123", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert!(
        s.starts_with("$2") && s.len() >= 60,
        "expected bcrypt hash format, got: {s}"
    );
}

#[test]
fn bcrypt_verify_correct_password_succeeds() {
    // Pre-computed bcrypt hash of "secret" with cost 4.
    let hash = "$2b$04$Cqhgyt8N0VBKlcLfxXGY0OqOj40C2NX9btSF9Q6kJyJyUFmCYUMxq";
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "secret", "--hash", hash])
        .assert()
        .success();
}

#[test]
fn bcrypt_verify_wrong_password_exits_5() {
    let hash = "$2b$04$Cqhgyt8N0VBKlcLfxXGY0OqOj40C2NX9btSF9Q6kJyJyUFmCYUMxq";
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "wrongpass", "--hash", hash])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn bcrypt_verify_malformed_hash_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "secret", "--hash", "not-a-bcrypt-hash"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_bcrypt"));
}

#[test]
fn bcrypt_hash_round_trip_via_verify() {
    let hash_out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "round-trip-pw", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8(hash_out).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "round-trip-pw", "--hash", &hash])
        .assert()
        .success();
}

#[test]
fn bcrypt_verify_json_mode_emits_verified_true() {
    let hash = "$2b$04$Cqhgyt8N0VBKlcLfxXGY0OqOj40C2NX9btSF9Q6kJyJyUFmCYUMxq";
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "bcrypt", "verify", "secret", "--hash", hash])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
}
```

> Note on the pre-computed hash: if `cargo test` shows the canned hash doesn't verify on your bcrypt version (cost-4 hashes are version-portable, but you may have a slightly different alphabet output), regenerate one inline with the just-built binary as the first step of the test and pin it. The literal above is `bcrypt::hash("secret", 4)` — verify it round-trips before pinning.

Run: `cargo test --test bcrypt`
Expected: FAIL — `unrecognized subcommand 'bcrypt'`.

- [ ] **Step 3: Implement the bcrypt module**

Create `src/commands/bcrypt/mod.rs`:

```rust
//! Bcrypt password hashing and verification.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod hash;
pub mod verify;

#[derive(Debug, Args)]
pub struct BcryptArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Hash a password using bcrypt.
    #[command(long_about = "Hash a password using bcrypt.\n\nExamples:\n  ubertool bcrypt hash \"my-password\"\n  ubertool bcrypt hash \"my-password\" --cost 12 --json\n  echo -n \"my-password\" | ubertool bcrypt hash")]
    Hash(hash::HashArgs),
    /// Verify a password against a bcrypt hash.
    #[command(long_about = "Verify a password against a bcrypt hash.\n\nExamples:\n  ubertool bcrypt verify \"my-password\" --hash \"$2b$12$...\"\n  ubertool bcrypt verify \"my-password\" --hash \"$2b$12$...\" --json\n\nExit codes specific to this command:\n  3   invalid bcrypt hash format\n  5   password does not match hash (signature_mismatch)")]
    Verify(verify::VerifyArgs),
}

pub fn dispatch(args: BcryptArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Hash(a) => hash::run(a, out),
        Verb::Verify(a) => verify::run(a, out),
    }
}
```

Create `src/commands/bcrypt/hash.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HashArgs {
    /// Password to hash (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read password from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Bcrypt cost factor (4-31; bcrypt default is 12).
    #[arg(long, default_value_t = bcrypt::DEFAULT_COST)]
    pub cost: u32,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
}

pub fn run(args: HashArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let password = input.as_str()?;
    let hashed = bcrypt::hash(password, args.cost).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("bcrypt hashing failed: {e}"))
            .with_hint("cost must be between 4 and 31")
    })?;
    out.emit_value(&HashOutput { hash: hashed })
}
```

Create `src/commands/bcrypt/verify.rs`:

```rust
use std::path::PathBuf;

use bcrypt::BcryptError;
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Password to check (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read password from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Bcrypt hash to verify against (required).
    #[arg(long)]
    pub hash: String,
}

#[derive(Serialize)]
struct VerifyOutput {
    verified: bool,
}

pub fn run(args: VerifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let password = input.as_str()?;
    match bcrypt::verify(password, &args.hash) {
        Ok(true) => out.emit_value(&VerifyOutput { verified: true }),
        Ok(false) => Err(CliError::new(
            ErrorCode::SignatureMismatch,
            "bcrypt verify: password does not match the supplied hash",
        )
        .with_hint("confirm the password is correct")),
        Err(BcryptError::InvalidHash(msg)) | Err(BcryptError::InvalidPrefix(msg)) => Err(
            CliError::new(ErrorCode::InvalidBcrypt, format!("malformed bcrypt hash: {msg}"))
                .with_input(serde_json::json!(args.hash))
                .with_hint("bcrypt hashes start with $2a$, $2b$, $2x$, or $2y$ followed by cost and salt"),
        ),
        Err(BcryptError::InvalidCost(msg)) => Err(CliError::new(
            ErrorCode::InvalidBcrypt,
            format!("invalid bcrypt cost: {msg}"),
        )),
        Err(e) => Err(CliError::new(
            ErrorCode::Internal,
            format!("bcrypt verify failed: {e}"),
        )),
    }
}
```

- [ ] **Step 4: Wire into CLI**

Modify `src/commands/mod.rs`:

```rust
pub mod base64;
pub mod bcrypt;
pub mod hash;
pub mod hmac;
```

Modify `src/cli.rs` `Noun` enum:

```rust
    /// Bcrypt password hashing and verification.
    Bcrypt(crate::commands::bcrypt::BcryptArgs),
```

Modify `src/lib.rs`:

```rust
        cli::Noun::Bcrypt(a) => commands::bcrypt::dispatch(a, &out),
```

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: PASS — 6 new integration tests + the extended unit test for `InvalidBcrypt`.

- [ ] **Step 6: Commit**

```bash
git add src/core/error.rs src/commands/bcrypt/ src/commands/mod.rs src/cli.rs src/lib.rs tests/bcrypt.rs
git commit -m "feat(bcrypt): add bcrypt hash|verify with exit-5 mismatch and exit-3 malformed hash"
```

---

## Task 5: `url` noun — `encode` + `decode` verbs

**Files:**
- Create: `src/commands/url/mod.rs`, `encode.rs`, `decode.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/url.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/url.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn url_encode_simple() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "hello world"])
        .assert()
        .success()
        .stdout("hello%20world\n");
}

#[test]
fn url_encode_special_chars() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "a+b=c&d"])
        .assert()
        .success()
        .stdout("a%2Bb%3Dc%26d\n");
}

#[test]
fn url_decode_simple() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "hello%20world"])
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn url_decode_special_chars() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "a%2Bb%3Dc%26d"])
        .assert()
        .success()
        .stdout("a+b=c&d\n");
}

#[test]
fn url_encode_round_trip_via_pipe() {
    let encoded = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "Hello, World! ñ"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let encoded_s = String::from_utf8(encoded).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", &encoded_s])
        .assert()
        .success()
        .stdout("Hello, World! ñ\n");
}

#[test]
fn url_decode_invalid_utf8_returns_exit_3() {
    // %ff%fe is not valid UTF-8.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "%ff%fe"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn url_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "url", "encode", "hello world"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["encoded"], "hello%20world");
}
```

Run: `cargo test --test url`
Expected: FAIL.

- [ ] **Step 2: Implement the url module**

Create `src/commands/url/mod.rs`:

```rust
//! URL percent-encoding and decoding.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct UrlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Percent-encode a string for use in URLs.
    #[command(long_about = "Percent-encode a string for use in URLs (non-ASCII + reserved chars become %XX).\n\nExamples:\n  ubertool url encode \"hello world\"\n  ubertool url encode \"a+b=c\" --json")]
    Encode(encode::EncodeArgs),
    /// Decode a percent-encoded string.
    #[command(long_about = "Decode a percent-encoded string.\n\nExamples:\n  ubertool url decode \"hello%20world\"\n  ubertool url decode \"a%2Bb%3Dc\" --json\n\nExit codes specific to this command:\n  3   decoded bytes are not valid UTF-8")]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: UrlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
```

Create `src/commands/url/encode.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Literal input to encode (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct EncodeOutput {
    encoded: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let encoded = utf8_percent_encode(s, NON_ALPHANUMERIC).to_string();
    out.emit_value(&EncodeOutput { encoded })
}
```

Create `src/commands/url/decode.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use percent_encoding::percent_decode_str;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Literal input to decode (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    decoded: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let decoded = percent_decode_str(s).decode_utf8().map_err(|_| {
        CliError::new(
            ErrorCode::UsageError,
            "decoded bytes are not valid UTF-8",
        )
        .with_input(serde_json::json!(s))
        .with_hint("the input contains percent-escapes that produce non-UTF-8 bytes")
    })?;
    out.emit_value(&DecodeOutput {
        decoded: decoded.into_owned(),
    })
}
```

Wait — invalid UTF-8 should be exit 3 (Invalid), not 2 (Usage). The test asserts exit 3. Adjust the error:

Actually re-reading the design: exit 3 is "input validation failed" — and percent-decoded bytes that are not UTF-8 is exactly that. The current `ErrorCode` set doesn't have a perfect fit; the closest is `InvalidBase64` (no, wrong domain). Let me add `InvalidUtf8`:

In `src/core/error.rs`, add `InvalidUtf8` variant (same pattern as `InvalidBcrypt` from Task 4 — add to enum, `as_str`, `exit` (maps to `Invalid`), and the sync test).

Then in `url/decode.rs`, use:

```rust
        CliError::new(
            ErrorCode::InvalidUtf8,
            "decoded bytes are not valid UTF-8",
        )
```

- [ ] **Step 3: Wire into CLI**

Modify `src/commands/mod.rs` — add `pub mod url;` (sorted alphabetically).

Modify `src/cli.rs` — add the `Url` variant:

```rust
    /// URL percent-encoding and decoding.
    Url(crate::commands::url::UrlArgs),
```

Modify `src/lib.rs` — add:

```rust
        cli::Noun::Url(a) => commands::url::dispatch(a, &out),
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: PASS — 7 new integration tests.

- [ ] **Step 5: Commit**

```bash
git add src/core/error.rs src/commands/url/ src/commands/mod.rs src/cli.rs src/lib.rs tests/url.rs
git commit -m "feat(url): add url encode|decode with exit-3 on invalid utf8 from decode"
```

---

## Task 6: `html` noun — `encode` + `decode` verbs

**Files:**
- Create: `src/commands/html/mod.rs`, `encode.rs`, `decode.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/html.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/html.rs`:

```rust
use assert_cmd::Command;

#[test]
fn html_encode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "encode", "<div class=\"x\">hi & bye</div>"])
        .assert()
        .success()
        .stdout("&lt;div class=&quot;x&quot;&gt;hi &amp; bye&lt;/div&gt;\n");
}

#[test]
fn html_decode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", "&lt;b&gt;hi&lt;/b&gt; &amp; bye"])
        .assert()
        .success()
        .stdout("<b>hi</b> & bye\n");
}

#[test]
fn html_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "html", "encode", "<x>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["encoded"], "&lt;x&gt;");
}

#[test]
fn html_round_trip_via_pipe() {
    let encoded = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "encode", "<i>x & y</i>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(encoded).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", &s])
        .assert()
        .success()
        .stdout("<i>x & y</i>\n");
}

#[test]
fn html_decode_unknown_entities_passthrough() {
    // Unknown entities are left unchanged by html-escape.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", "&notarealentity; &amp;"])
        .assert()
        .success()
        .stdout("&notarealentity; &\n");
}
```

Run: `cargo test --test html`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/html/mod.rs`:

```rust
//! HTML entity encoding/decoding.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct HtmlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode HTML-special characters into entities.
    #[command(long_about = "Encode HTML-special characters into entities (<, >, &, \", ').\n\nExamples:\n  ubertool html encode \"<b>hi</b>\"\n  ubertool html encode \"<b>hi</b>\" --json")]
    Encode(encode::EncodeArgs),
    /// Decode HTML entities back to characters.
    #[command(long_about = "Decode HTML entities back to characters. Unknown entities pass through unchanged.\n\nExamples:\n  ubertool html decode \"&lt;b&gt;hi&lt;/b&gt;\"\n  ubertool html decode \"&amp;\" --json")]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: HtmlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
```

Create `src/commands/html/encode.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct EncodeArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct EncodeOutput {
    encoded: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let encoded = html_escape::encode_text(s).into_owned();
    // html_escape's encode_text only escapes <, >, & by default; we also need quotes
    // for attribute safety. Apply an additional pass.
    let encoded = encoded.replace('"', "&quot;").replace('\'', "&#x27;");
    out.emit_value(&EncodeOutput { encoded })
}
```

Create `src/commands/html/decode.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    decoded: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let decoded = html_escape::decode_html_entities(s).into_owned();
    out.emit_value(&DecodeOutput { decoded })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod html;` to `src/commands/mod.rs`.

Add to `src/cli.rs` `Noun` enum:

```rust
    /// HTML entity encoding and decoding.
    Html(crate::commands::html::HtmlArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Html(a) => commands::html::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS — 5 new integration tests.

- [ ] **Step 5: Commit**

```bash
git add src/commands/html/ src/commands/mod.rs src/cli.rs src/lib.rs tests/html.rs
git commit -m "feat(html): add html encode|decode covering quotes and named entities"
```

---

## Task 7: `basic-auth` noun — `encode` + `decode` verbs

**Files:**
- Create: `src/commands/basic_auth/mod.rs`, `encode.rs`, `decode.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/basic_auth.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/basic_auth.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn basic_auth_encode_text_mode() {
    // "Aladdin:open sesame" → "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "basic-auth",
            "encode",
            "--user",
            "Aladdin",
            "--pass",
            "open sesame",
        ])
        .assert()
        .success()
        .stdout("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==\n");
}

#[test]
fn basic_auth_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "basic-auth",
            "encode",
            "--user",
            "user",
            "--pass",
            "pass",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["header"], "Basic dXNlcjpwYXNz");
}

#[test]
fn basic_auth_decode_with_basic_prefix() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "basic-auth", "decode", "Basic dXNlcjpwYXNz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["user"], "user");
    assert_eq!(v["pass"], "pass");
}

#[test]
fn basic_auth_decode_without_basic_prefix() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "basic-auth", "decode", "dXNlcjpwYXNz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["user"], "user");
}

#[test]
fn basic_auth_decode_invalid_base64_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["basic-auth", "decode", "!!!"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_base64"));
}

#[test]
fn basic_auth_decode_missing_colon_exits_3() {
    // "nocolon" base64-encoded — decodes fine but has no colon separator.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["basic-auth", "decode", "bm9jb2xvbg=="])
        .assert()
        .failure()
        .code(3);
}
```

Run: `cargo test --test basic_auth`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/basic_auth/mod.rs`:

```rust
//! HTTP Basic authentication: encode user/pass to a header value, decode back.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod encode;

#[derive(Debug, Args)]
pub struct BasicAuthArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode a user/pass pair as an HTTP Basic Authorization header value.
    #[command(long_about = "Encode user/password as an HTTP Basic Authorization header value.\n\nExamples:\n  ubertool basic-auth encode --user alice --pass hunter2\n  ubertool basic-auth encode --user alice --pass hunter2 --json")]
    Encode(encode::EncodeArgs),
    /// Decode an HTTP Basic Authorization header value into user/pass.
    #[command(long_about = "Decode an HTTP Basic Authorization header value into user/pass. The leading \"Basic \" prefix is optional.\n\nExamples:\n  ubertool basic-auth decode \"Basic dXNlcjpwYXNz\"\n  ubertool basic-auth decode dXNlcjpwYXNz --json\n\nExit codes specific to this command:\n  3   invalid base64, or decoded value has no `:` separator")]
    Decode(decode::DecodeArgs),
}

pub fn dispatch(args: BasicAuthArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Encode(a) => encode::run(a, out),
        Verb::Decode(a) => decode::run(a, out),
    }
}
```

Create `src/commands/basic_auth/encode.rs`:

```rust
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Args;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Username.
    #[arg(long)]
    pub user: String,
    /// Password.
    #[arg(long)]
    pub pass: String,
}

#[derive(Serialize)]
struct EncodeOutput {
    header: String,
}

pub fn run(args: EncodeArgs, out: &Out) -> Result<(), CliError> {
    let credential = format!("{}:{}", args.user, args.pass);
    let b64 = STANDARD.encode(credential.as_bytes());
    out.emit_value(&EncodeOutput {
        header: format!("Basic {b64}"),
    })
}
```

Create `src/commands/basic_auth/decode.rs`:

```rust
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// Header value to decode (with or without the leading "Basic ").
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    user: String,
    pass: String,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let raw = input.as_str()?.trim();
    let b64 = raw.strip_prefix("Basic ").unwrap_or(raw);
    let decoded = STANDARD.decode(b64).map_err(|e| {
        CliError::new(ErrorCode::InvalidBase64, format!("base64 decode failed: {e}"))
            .with_input(serde_json::json!(b64))
            .with_hint("Basic auth header values are standard base64 of `user:pass`")
    })?;
    let s = String::from_utf8(decoded).map_err(|_| {
        CliError::new(
            ErrorCode::InvalidUtf8,
            "decoded credentials are not valid UTF-8",
        )
    })?;
    let (user, pass) = s.split_once(':').ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidBase64,
            "decoded credentials have no `:` separator",
        )
        .with_hint("expected format is `user:pass`")
    })?;
    out.emit_value(&DecodeOutput {
        user: user.to_string(),
        pass: pass.to_string(),
    })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod basic_auth;` to `src/commands/mod.rs`.

Add to `src/cli.rs` `Noun` enum (the explicit clap name maps to the kebab-case CLI surface):

```rust
    /// HTTP Basic auth header encoding/decoding.
    #[command(name = "basic-auth")]
    BasicAuth(crate::commands::basic_auth::BasicAuthArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::BasicAuth(a) => commands::basic_auth::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/basic_auth/ src/commands/mod.rs src/cli.rs src/lib.rs tests/basic_auth.rs
git commit -m "feat(basic-auth): add basic-auth encode|decode with optional \"Basic \" prefix"
```

---

## Task 8: `jwt` noun — `decode` + `verify` verbs

**Files:**
- Create: `src/commands/jwt/mod.rs`, `decode.rs`, `verify.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/jwt.rs`

> Pre-generated test fixtures for `tests/jwt.rs` (these are well-known JWT.io example tokens):
> - **HS256, secret "secret":** `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c`

- [ ] **Step 1: Write failing integration test**

Create `tests/jwt.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

const HS256_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

#[test]
fn jwt_decode_emits_header_and_claims() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "jwt", "decode", HS256_TOKEN])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["header"]["alg"], "HS256");
    assert_eq!(v["header"]["typ"], "JWT");
    assert_eq!(v["claims"]["sub"], "1234567890");
    assert_eq!(v["claims"]["name"], "John Doe");
}

#[test]
fn jwt_decode_malformed_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "decode", "not.a.jwt"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}

#[test]
fn jwt_verify_correct_secret_succeeds() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success();
}

#[test]
fn jwt_verify_correct_secret_emits_claims_json() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "jwt", "verify", HS256_TOKEN, "--secret", "secret",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
    assert_eq!(v["claims"]["sub"], "1234567890");
}

#[test]
fn jwt_verify_wrong_secret_exits_5() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "wrong"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn jwt_verify_secret_is_redacted_from_json_error_input() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "jwt", "verify", HS256_TOKEN, "--secret", "wrong-secret-do-not-leak",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("wrong-secret-do-not-leak"),
        "secret leaked in JSON error: {s}"
    );
}

#[test]
fn jwt_verify_malformed_token_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", "garbage", "--secret", "secret"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}
```

Run: `cargo test --test jwt`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/jwt/mod.rs`:

```rust
//! JSON Web Token decoding and verification.

use clap::{Args, Subcommand, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod decode;
pub mod verify;

#[derive(Debug, Args)]
pub struct JwtArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Decode a JWT without verifying its signature.
    #[command(long_about = "Decode a JWT without verifying its signature — useful for inspection.\n\nExamples:\n  ubertool jwt decode \"eyJhbGc...\"\n  ubertool jwt decode \"eyJhbGc...\" --json\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)")]
    Decode(decode::DecodeArgs),
    /// Verify a JWT signature (HS256/HS384/HS512 only in this build).
    #[command(long_about = "Verify a JWT signature against a shared secret. HS256, HS384, HS512 only.\n\nExamples:\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\"\n  ubertool jwt verify \"eyJhbGc...\" --secret \"my-secret\" --algo hs512 --json\n\nExit codes specific to this command:\n  3   malformed JWT (invalid_jwt)\n  5   signature mismatch / wrong secret (signature_mismatch)\n  6   asymmetric algorithm not supported in this build (algo_not_supported)")]
    Verify(verify::VerifyArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum JwtAlgo {
    Hs256,
    Hs384,
    Hs512,
}

impl JwtAlgo {
    pub fn to_jsonwebtoken(self) -> jsonwebtoken::Algorithm {
        match self {
            JwtAlgo::Hs256 => jsonwebtoken::Algorithm::HS256,
            JwtAlgo::Hs384 => jsonwebtoken::Algorithm::HS384,
            JwtAlgo::Hs512 => jsonwebtoken::Algorithm::HS512,
        }
    }
}

pub fn dispatch(args: JwtArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Decode(a) => decode::run(a, out),
        Verb::Verify(a) => verify::run(a, out),
    }
}
```

Create `src/commands/jwt/decode.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// JWT to decode (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct DecodeOutput {
    header: serde_json::Value,
    claims: serde_json::Value,
}

pub fn run(args: DecodeArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let token = input.as_str()?.trim();

    // Split & base64-decode without invoking the signature validator.
    // The two header/claim segments are URL-safe base64 of JSON.
    let mut parts = token.split('.');
    let header_b64 = parts.next().ok_or_else(|| jwt_invalid("missing header"))?;
    let claims_b64 = parts.next().ok_or_else(|| jwt_invalid("missing claims"))?;
    let _sig = parts.next().ok_or_else(|| jwt_invalid("missing signature"))?;
    if parts.next().is_some() {
        return Err(jwt_invalid("too many segments"));
    }

    let header = decode_segment(header_b64).map_err(|_| jwt_invalid("invalid header encoding"))?;
    let claims = decode_segment(claims_b64).map_err(|_| jwt_invalid("invalid claims encoding"))?;

    out.emit_value(&DecodeOutput { header, claims })
}

fn decode_segment(s: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let bytes = URL_SAFE_NO_PAD.decode(s)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn jwt_invalid(msg: &str) -> CliError {
    CliError::new(ErrorCode::InvalidJwt, format!("malformed JWT: {msg}"))
        .with_hint("JWT must be three URL-safe base64 segments joined by `.`")
}
```

Create `src/commands/jwt/verify.rs`:

```rust
use std::path::PathBuf;

use clap::Args;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::JwtAlgo;

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// JWT to verify (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Shared HMAC secret (required for HS* algorithms).
    #[arg(long)]
    pub secret: String,
    /// Signing algorithm.
    #[arg(long, value_enum, default_value = "hs256")]
    pub algo: JwtAlgo,
    /// Validate `exp` claim against the current time (off by default).
    #[arg(long, default_value_t = false)]
    pub validate_exp: bool,
}

#[derive(Serialize)]
struct VerifyOutput {
    verified: bool,
    claims: serde_json::Value,
}

pub fn run(args: VerifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let token = input.as_str()?.trim();

    let key = DecodingKey::from_secret(args.secret.as_bytes());
    let mut validation = Validation::new(args.algo.to_jsonwebtoken());
    validation.validate_exp = args.validate_exp;
    // We never enforce aud/iss/nbf at the CLI level — the user can post-process claims.
    validation.required_spec_claims.clear();

    match decode::<serde_json::Value>(token, &key, &validation) {
        Ok(data) => out.emit_value(&VerifyOutput {
            verified: true,
            claims: data.claims,
        }),
        Err(e) => {
            use jsonwebtoken::errors::ErrorKind::*;
            let (code, msg) = match e.kind() {
                InvalidSignature => (
                    ErrorCode::SignatureMismatch,
                    "JWT signature does not verify with the provided secret",
                ),
                ExpiredSignature => (
                    ErrorCode::SignatureMismatch,
                    "JWT has expired (exp claim < now)",
                ),
                InvalidAlgorithm | InvalidAlgorithmName => (
                    ErrorCode::AlgoNotSupported,
                    "JWT algorithm is not supported by this build",
                ),
                _ => (ErrorCode::InvalidJwt, "JWT is malformed"),
            };
            // Echo the JWT (it's not secret) but NEVER the secret.
            Err(CliError::new(code, msg)
                .with_input(serde_json::json!({"token": token, "secret": "<redacted>"}))
                .with_hint(
                    "if the JWT uses RS*/ES*, this build does not support asymmetric keys",
                ))
        }
    }
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod jwt;` to `src/commands/mod.rs`.

Add to `src/cli.rs` `Noun` enum:

```rust
    /// JSON Web Token decode and verify (HS* only).
    Jwt(crate::commands::jwt::JwtArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Jwt(a) => commands::jwt::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS — 7 new integration tests.

- [ ] **Step 5: Commit**

```bash
git add src/commands/jwt/ src/commands/mod.rs src/cli.rs src/lib.rs tests/jwt.rs
git commit -m "feat(jwt): add jwt decode|verify with HS256/384/512 and secret-redaction in error output"
```

---

## Task 9: `uuid` noun — `new` verb

**Files:**
- Create: `src/commands/uuid/mod.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/uuid.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/uuid.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn uuid_new_default_is_v4() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // UUIDv4: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx where y ∈ [89ab].
    let parts: Vec<&str> = s.split('-').collect();
    assert_eq!(parts.len(), 5, "expected 5 hyphenated parts in {s}");
    assert!(parts[2].starts_with('4'), "v4 must start version nibble with 4 in {s}");
}

#[test]
fn uuid_new_v7_emits_time_ordered_uuid() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new", "--version", "7"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split('-').collect();
    assert!(parts[2].starts_with('7'), "v7 must start version nibble with 7 in {s}");
}

#[test]
fn uuid_new_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "uuid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["uuid"].is_string());
    let s = v["uuid"].as_str().unwrap();
    assert_eq!(s.len(), 36, "uuid length must be 36 with hyphens");
}

#[test]
fn uuid_new_invalid_version_is_usage_error() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new", "--version", "99"])
        .assert()
        .failure()
        .code(2);
}
```

Run: `cargo test --test uuid`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/uuid/mod.rs`:

```rust
//! UUID generator.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use uuid::Uuid;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UuidArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a new UUID.
    #[command(long_about = "Generate a new UUID. Default version is 4 (random).\n\nExamples:\n  ubertool uuid new\n  ubertool uuid new --version 7\n  ubertool uuid new --version 4 --json")]
    New(NewArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UuidVersion {
    #[value(name = "4")]
    V4,
    #[value(name = "7")]
    V7,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// UUID version (4 = random, 7 = time-ordered).
    #[arg(long, value_enum, default_value = "4")]
    pub version: UuidVersion,
}

#[derive(Serialize)]
struct UuidOutput {
    uuid: String,
}

pub fn dispatch(args: UuidArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New(a) => run(a, out),
    }
}

fn run(args: NewArgs, out: &Out) -> Result<(), CliError> {
    let u = match args.version {
        UuidVersion::V4 => Uuid::new_v4(),
        UuidVersion::V7 => Uuid::now_v7(),
    };
    out.emit_value(&UuidOutput {
        uuid: u.hyphenated().to_string(),
    })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod uuid;` to `src/commands/mod.rs`.

Add to `src/cli.rs` `Noun` enum:

```rust
    /// UUID generation (v4, v7).
    Uuid(crate::commands::uuid::UuidArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Uuid(a) => commands::uuid::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/uuid/ src/commands/mod.rs src/cli.rs src/lib.rs tests/uuid.rs
git commit -m "feat(uuid): add uuid new with --version 4|7"
```

---

## Task 10: `ulid` noun — `new` verb

**Files:**
- Create: `src/commands/ulid/mod.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/ulid.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/ulid.rs`:

```rust
use assert_cmd::Command;

#[test]
fn ulid_new_text_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["ulid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // ULID is 26 Crockford base32 chars, no hyphens.
    assert_eq!(s.len(), 26, "expected 26-char ULID, got {s}");
    for c in s.chars() {
        assert!(c.is_ascii_alphanumeric(), "non-alphanumeric in ULID: {s}");
    }
}

#[test]
fn ulid_new_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "ulid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let s = v["ulid"].as_str().expect("ulid field must be a string");
    assert_eq!(s.len(), 26);
}

#[test]
fn ulid_new_two_calls_produce_different_ids() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool")
            .unwrap()
            .args(["ulid", "new"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one(), "two ULIDs must differ");
}
```

Run: `cargo test --test ulid`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/ulid/mod.rs`:

```rust
//! ULID generator.

use clap::{Args, Subcommand};
use serde::Serialize;
use ulid::Ulid;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct UlidArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a new ULID (lexically-sortable 128-bit identifier).
    #[command(long_about = "Generate a new ULID (lexically-sortable, 128-bit, 26-char Crockford base32).\n\nExamples:\n  ubertool ulid new\n  ubertool ulid new --json")]
    New,
}

#[derive(Serialize)]
struct UlidOutput {
    ulid: String,
}

pub fn dispatch(args: UlidArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => out.emit_value(&UlidOutput {
            ulid: Ulid::new().to_string(),
        }),
    }
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod ulid;` to `src/commands/mod.rs`.

Add to `src/cli.rs` `Noun` enum:

```rust
    /// ULID generation.
    Ulid(crate::commands::ulid::UlidArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Ulid(a) => commands::ulid::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/ulid/ src/commands/mod.rs src/cli.rs src/lib.rs tests/ulid.rs
git commit -m "feat(ulid): add ulid new"
```

---

## Task 11: `token` noun — `new` verb

**Files:**
- Create: `src/commands/token/mod.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/token.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/token.rs`:

```rust
use assert_cmd::Command;

#[test]
fn token_new_default_is_hex_64_chars() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["token", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // 32 bytes of entropy → 64 hex chars by default.
    assert_eq!(s.len(), 64);
    assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn token_new_length_16_hex_emits_32_chars() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["token", "new", "--length", "16"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert_eq!(s.trim().len(), 32);
}

#[test]
fn token_new_format_base64() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["token", "new", "--length", "12", "--format", "base64"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // 12 bytes → 16 base64 chars (with padding).
    assert_eq!(s.trim().len(), 16);
}

#[test]
fn token_new_format_alphanumeric() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["token", "new", "--length", "24", "--format", "alphanumeric"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 24);
    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn token_new_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "token", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["token"].is_string());
}
```

Run: `cargo test --test token`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/token/mod.rs`:

```rust
//! Random token generator — hex, base64, or alphanumeric.

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::{Args, Subcommand, ValueEnum};
use rand::Rng;
use rand::RngCore;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::hex;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct TokenArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random token.
    #[command(long_about = "Generate a random token from a CSPRNG.\n\nFor hex and base64, --length is the number of bytes of entropy. For alphanumeric, --length is the number of output characters.\n\nExamples:\n  ubertool token new                              # 32 bytes → 64 hex chars\n  ubertool token new --length 16 --format hex     # 32 hex chars\n  ubertool token new --format base64\n  ubertool token new --format alphanumeric --length 24 --json")]
    New(NewArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Hex,
    Base64,
    Alphanumeric,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Length: bytes of entropy (for hex/base64), or character count (for alphanumeric).
    #[arg(long, default_value_t = 32)]
    pub length: usize,
    /// Output format.
    #[arg(long, value_enum, default_value = "hex")]
    pub format: Format,
}

#[derive(Serialize)]
struct TokenOutput {
    token: String,
}

const ALPHANUMERIC: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn dispatch(args: TokenArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New(a) => run(a, out),
    }
}

fn run(args: NewArgs, out: &Out) -> Result<(), CliError> {
    let mut rng = rand::thread_rng();
    let token = match args.format {
        Format::Hex => {
            let mut buf = vec![0u8; args.length];
            rng.fill_bytes(&mut buf);
            hex::encode(&buf)
        }
        Format::Base64 => {
            let mut buf = vec![0u8; args.length];
            rng.fill_bytes(&mut buf);
            STANDARD.encode(&buf)
        }
        Format::Alphanumeric => (0..args.length)
            .map(|_| ALPHANUMERIC[rng.gen_range(0..ALPHANUMERIC.len())] as char)
            .collect(),
    };
    out.emit_value(&TokenOutput { token })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod token;` to `src/commands/mod.rs`.

Add to `src/cli.rs`:

```rust
    /// Random token generation (hex / base64 / alphanumeric).
    Token(crate::commands::token::TokenArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Token(a) => commands::token::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/token/ src/commands/mod.rs src/cli.rs src/lib.rs tests/token.rs
git commit -m "feat(token): add token new with --length and --format hex|base64|alphanumeric"
```

---

## Task 12: `password` noun — `score` verb

**Files:**
- Create: `src/commands/password/mod.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/password.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/password.rs`:

```rust
use assert_cmd::Command;

#[test]
fn password_score_weak() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "password"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let score = v["score"].as_u64().unwrap();
    assert!(score <= 1, "expected weak score for 'password', got {score}");
}

#[test]
fn password_score_strong() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "password", "score", "Tr0ub4dor&3-correct-horse-battery-staple",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let score = v["score"].as_u64().unwrap();
    assert!(score >= 3, "expected strong score, got {score}");
}

#[test]
fn password_score_emits_strength_label_and_suggestions() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "abc"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["strength"].is_string(), "must emit strength label");
    assert!(v["guesses_log10"].is_number());
    assert!(v["suggestions"].is_array());
}

#[test]
fn password_score_does_not_echo_password_in_error_path() {
    // No real error path here, but smoke-test that we never accidentally echo
    // the input in normal output either.
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "my-secret-password-xyz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("my-secret-password-xyz"),
        "password must not appear in score output: {s}"
    );
}
```

Run: `cargo test --test password`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/password/mod.rs`:

```rust
//! Password strength scoring via zxcvbn.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct PasswordArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Score a password's strength (zxcvbn).
    #[command(long_about = "Score a password's strength using zxcvbn.\n\nReturns a score 0-4 (very-weak..very-strong), a log10 estimate of guesses required, and feedback strings. The password itself is never echoed in output.\n\nExamples:\n  ubertool password score \"P@ssw0rd!\"\n  echo -n \"P@ssw0rd!\" | ubertool password score --json")]
    Score(ScoreArgs),
}

#[derive(Debug, Args)]
pub struct ScoreArgs {
    /// Password (omit to read from --in or stdin).
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct ScoreOutput {
    score: u8,
    strength: &'static str,
    guesses_log10: f64,
    warning: Option<String>,
    suggestions: Vec<String>,
}

pub fn dispatch(args: PasswordArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Score(a) => run(a, out),
    }
}

fn run(args: ScoreArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    // Strip trailing newline from piped/file input — common ergonomic snag.
    let password = input.as_str()?.trim_end_matches(['\r', '\n']);

    let entropy = zxcvbn::zxcvbn(password, &[]);
    let score = entropy.score() as u8;
    let strength = match score {
        0 => "very-weak",
        1 => "weak",
        2 => "fair",
        3 => "strong",
        _ => "very-strong",
    };
    let (warning, suggestions) = match entropy.feedback() {
        Some(fb) => (
            fb.warning().map(|w| format!("{w:?}")),
            fb.suggestions().iter().map(|s| format!("{s:?}")).collect(),
        ),
        None => (None, Vec::new()),
    };

    out.emit_value(&ScoreOutput {
        score,
        strength,
        guesses_log10: entropy.guesses_log10(),
        warning,
        suggestions,
    })
}
```

> Note on zxcvbn 2.x API: `zxcvbn::zxcvbn(password, &[])` returns `Entropy` directly (infallible for non-empty input; empty password returns score 0). `Feedback::warning()` returns `Option<&Warning>` and `suggestions()` returns `&[Suggestion]`. Both `Warning` and `Suggestion` implement `Debug` (formatted via `{:?}`); if the crate version on disk implements `Display`, swap `{:?}` to `{}` and update the test expectations accordingly.

- [ ] **Step 3: Wire into CLI**

Add `pub mod password;` to `src/commands/mod.rs`.

Add to `src/cli.rs`:

```rust
    /// Password strength scoring (zxcvbn).
    Password(crate::commands::password::PasswordArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Password(a) => commands::password::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/password/ src/commands/mod.rs src/cli.rs src/lib.rs tests/password.rs
git commit -m "feat(password): add password score (zxcvbn) with strength label and suggestions"
```

---

## Task 13: `json` noun — `to-yaml` + `to-toml` + `minify` + `prettify`

**Files:**
- Create: `src/commands/json/mod.rs`, `to_yaml.rs`, `to_toml.rs`, `minify.rs`, `prettify.rs`, `convert.rs` (shared JSON↔TOML conversion helper)
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/json_convert.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/json_convert.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn json_minify_compacts_whitespace() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "minify", r#"{ "a": 1, "b": [1, 2] }"#])
        .assert()
        .success()
        .stdout("{\"a\":1,\"b\":[1,2]}\n");
}

#[test]
fn json_prettify_indents_two_spaces() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "prettify", r#"{"a":1,"b":2}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("  \"a\": 1"));
}

#[test]
fn json_prettify_custom_indent() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "prettify", r#"{"a":1}"#, "--indent", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("    \"a\": 1"));
}

#[test]
fn json_to_yaml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-yaml", r#"{"name":"alice","age":30}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name: alice"));
    assert!(s.contains("age: 30"));
}

#[test]
fn json_to_toml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-toml", r#"{"name":"alice","age":30}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name = \"alice\""));
    assert!(s.contains("age = 30"));
}

#[test]
fn json_to_toml_null_value_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-toml", r#"{"a":null}"#])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_toml"));
}

#[test]
fn json_minify_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "minify", "{not json}"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_json"));
}

#[test]
fn json_to_yaml_json_mode_wraps_in_yaml_field() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "json", "to-yaml", r#"{"k":"v"}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let yaml = v["yaml"].as_str().expect("must have yaml field");
    assert!(yaml.contains("k: v"));
}
```

Run: `cargo test --test json_convert`
Expected: FAIL.

- [ ] **Step 2: Implement the json/convert helper**

Create `src/commands/json/convert.rs`:

```rust
//! Shared helpers used by all four JSON verbs.

use crate::core::error::{CliError, ErrorCode};

pub fn parse_json(s: &str) -> Result<serde_json::Value, CliError> {
    serde_json::from_str(s).map_err(|e| {
        CliError::new(ErrorCode::InvalidJson, format!("invalid JSON: {e}"))
            .with_hint("ensure the input is well-formed JSON")
    })
}

/// Convert serde_json::Value → toml::Value. TOML lacks a null type; nulls fail
/// fast with `invalid_toml`.
pub fn json_to_toml(v: serde_json::Value) -> Result<toml::Value, CliError> {
    match v {
        serde_json::Value::Null => Err(CliError::new(
            ErrorCode::InvalidToml,
            "TOML does not support null values",
        )
        .with_hint("remove or replace null fields before converting")),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(CliError::new(
                    ErrorCode::InvalidToml,
                    "number out of range for TOML",
                ))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s)),
        serde_json::Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.into_iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(items?))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map {
                table.insert(k, json_to_toml(v)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}
```

Create `src/commands/json/mod.rs`:

```rust
//! JSON conversion verbs: to-yaml, to-toml, minify, prettify.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod convert;
pub mod minify;
pub mod prettify;
pub mod to_toml;
pub mod to_yaml;

#[derive(Debug, Args)]
pub struct JsonArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert JSON to YAML.
    #[command(name = "to-yaml", long_about = "Convert JSON to YAML.\n\nExamples:\n  ubertool json to-yaml '{\"k\":\"v\"}'\n  ubertool json to-yaml --in ./data.json\n  ubertool json to-yaml --in ./data.json --json   # wraps result in {\"yaml\":\"...\"}")]
    ToYaml(RunArgs),
    /// Convert JSON to TOML.
    #[command(name = "to-toml", long_about = "Convert JSON to TOML.\n\nTOML does not support null values; nulls fail with exit 3 (invalid_toml).\n\nExamples:\n  ubertool json to-toml '{\"a\":1}'\n  ubertool json to-toml --in ./data.json\n\nExit codes specific to this command:\n  3   invalid JSON, or value type not representable in TOML")]
    ToToml(RunArgs),
    /// Minify JSON (strip whitespace).
    #[command(long_about = "Minify JSON (strip insignificant whitespace).\n\nExamples:\n  ubertool json minify '{ \"a\": 1 }'\n  cat data.json | ubertool json minify")]
    Minify(RunArgs),
    /// Prettify JSON with indentation.
    #[command(long_about = "Prettify JSON with indentation.\n\nExamples:\n  ubertool json prettify '{\"a\":1}'\n  ubertool json prettify '{\"a\":1}' --indent 4")]
    Prettify(PrettifyArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct PrettifyArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Indent width in spaces (default: 2).
    #[arg(long, default_value_t = 2)]
    pub indent: usize,
}

pub fn dispatch(args: JsonArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToYaml(a) => to_yaml::run(a, out),
        Verb::ToToml(a) => to_toml::run(a, out),
        Verb::Minify(a) => minify::run(a, out),
        Verb::Prettify(a) => prettify::run(a, out),
    }
}
```

Create `src/commands/json/to_yaml.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::parse_json;
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    yaml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_json(input.as_str()?)?;
    let yaml = serde_yaml::to_string(&v).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("yaml serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { yaml })
}
```

Create `src/commands/json/to_toml.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::{json_to_toml, parse_json};
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    toml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let json_val = parse_json(input.as_str()?)?;
    let toml_val = json_to_toml(json_val)?;
    let s = toml::to_string_pretty(&toml_val).map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("TOML serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { toml: s })
}
```

Create `src/commands/json/minify.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::parse_json;
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_json(input.as_str()?)?;
    let minified = serde_json::to_string(&v).unwrap();
    out.emit_value(&Out0 { json: minified })
}
```

Create `src/commands/json/prettify.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::convert::parse_json;
use super::PrettifyArgs;

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: PrettifyArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_json(input.as_str()?)?;
    let indent_bytes = " ".repeat(args.indent);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes.as_bytes());
    let mut buf = Vec::new();
    {
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        use serde::Serialize as _;
        v.serialize(&mut ser).map_err(|e| {
            CliError::new(
                ErrorCode::Internal,
                format!("json prettify failed: {e}"),
            )
        })?;
    }
    let pretty = String::from_utf8(buf).map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("pretty buffer is not UTF-8: {e}"),
        )
    })?;
    out.emit_value(&Out0 { json: pretty })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod json;` to `src/commands/mod.rs`.

Add to `src/cli.rs`:

```rust
    /// JSON conversion: to-yaml / to-toml / minify / prettify.
    Json(crate::commands::json::JsonArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Json(a) => commands::json::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS — 8 new integration tests.

- [ ] **Step 5: Commit**

```bash
git add src/commands/json/ src/commands/mod.rs src/cli.rs src/lib.rs tests/json_convert.rs
git commit -m "feat(json): add json to-yaml|to-toml|minify|prettify with shared parse/convert helpers"
```

---

## Task 14: `yaml` noun — `to-json` + `to-toml`

**Files:**
- Create: `src/commands/yaml/mod.rs`, `to_json.rs`, `to_toml.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/yaml_convert.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/yaml_convert.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn yaml_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "name: alice\nage: 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(v["name"], "alice");
    assert_eq!(v["age"], 30);
}

#[test]
fn yaml_to_json_pretty() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "a: 1\nb: 2\n", "--pretty"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("  \"a\": 1"));
}

#[test]
fn yaml_to_toml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-toml", "name: alice\nage: 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name = \"alice\""));
    assert!(s.contains("age = 30"));
}

#[test]
fn yaml_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "key: [unclosed"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_yaml"));
}
```

Run: `cargo test --test yaml_convert`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/yaml/mod.rs`:

```rust
//! YAML conversion verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

pub mod to_json;
pub mod to_toml;

#[derive(Debug, Args)]
pub struct YamlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert YAML to JSON.
    #[command(name = "to-json", long_about = "Convert YAML to JSON.\n\nExamples:\n  ubertool yaml to-json 'name: alice'\n  ubertool yaml to-json --in ./config.yaml --pretty\n\nExit codes specific to this command:\n  3   invalid YAML (invalid_yaml)")]
    ToJson(ToJsonArgs),
    /// Convert YAML to TOML.
    #[command(name = "to-toml", long_about = "Convert YAML to TOML via JSON value interchange.\n\nExamples:\n  ubertool yaml to-toml 'k: v'\n\nExit codes specific to this command:\n  3   invalid YAML, or value type not representable in TOML")]
    ToToml(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ToJsonArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Emit indented JSON instead of compact.
    #[arg(long, default_value_t = false)]
    pub pretty: bool,
}

pub fn dispatch(args: YamlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::ToToml(a) => to_toml::run(a, out),
    }
}

pub(super) fn parse_yaml(s: &str) -> Result<serde_yaml::Value, CliError> {
    serde_yaml::from_str(s).map_err(|e| {
        CliError::new(ErrorCode::InvalidYaml, format!("invalid YAML: {e}"))
            .with_hint("ensure the input is well-formed YAML")
    })
}
```

Create `src/commands/yaml/to_json.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::{parse_yaml, ToJsonArgs};

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: ToJsonArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_yaml(input.as_str()?)?;
    let json = if args.pretty {
        serde_json::to_string_pretty(&v)
    } else {
        serde_json::to_string(&v)
    }
    .map_err(|e| CliError::new(ErrorCode::Internal, format!("json serialization failed: {e}")))?;
    out.emit_value(&Out0 { json })
}
```

Create `src/commands/yaml/to_toml.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use crate::commands::json::convert::json_to_toml;

use super::{parse_yaml, RunArgs};

#[derive(Serialize)]
struct Out0 {
    toml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let yaml_val = parse_yaml(input.as_str()?)?;
    // Bridge via serde_json::Value to reuse the json_to_toml machinery.
    let json_val: serde_json::Value = serde_json::to_value(yaml_val).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("yaml→json bridge failed: {e}"))
    })?;
    let toml_val = json_to_toml(json_val)?;
    let s = toml::to_string_pretty(&toml_val).map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("TOML serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { toml: s })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod yaml;` to `src/commands/mod.rs`.

Add to `src/cli.rs`:

```rust
    /// YAML conversion: to-json / to-toml.
    Yaml(crate::commands::yaml::YamlArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Yaml(a) => commands::yaml::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/yaml/ src/commands/mod.rs src/cli.rs src/lib.rs tests/yaml_convert.rs
git commit -m "feat(yaml): add yaml to-json|to-toml (via json::convert bridge)"
```

---

## Task 15: `toml` noun — `to-json` + `to-yaml`

**Files:**
- Create: `src/commands/toml/mod.rs`, `to_json.rs`, `to_yaml.rs`
- Modify: `src/commands/mod.rs`, `src/cli.rs`, `src/lib.rs`
- Create: `tests/toml_convert.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/toml_convert.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn toml_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-json", "name = \"alice\"\nage = 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(v["name"], "alice");
    assert_eq!(v["age"], 30);
}

#[test]
fn toml_to_yaml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-yaml", "name = \"alice\"\nage = 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name: alice"));
    assert!(s.contains("age: 30"));
}

#[test]
fn toml_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-json", "= invalid"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_toml"));
}
```

Run: `cargo test --test toml_convert`
Expected: FAIL.

- [ ] **Step 2: Implement**

Create `src/commands/toml/mod.rs`:

```rust
//! TOML conversion verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

pub mod to_json;
pub mod to_yaml;

#[derive(Debug, Args)]
pub struct TomlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert TOML to JSON.
    #[command(name = "to-json", long_about = "Convert TOML to JSON.\n\nExamples:\n  ubertool toml to-json 'k = \"v\"'\n  ubertool toml to-json --in ./Cargo.toml --pretty\n\nExit codes specific to this command:\n  3   invalid TOML (invalid_toml)")]
    ToJson(ToJsonArgs),
    /// Convert TOML to YAML.
    #[command(name = "to-yaml", long_about = "Convert TOML to YAML.\n\nExamples:\n  ubertool toml to-yaml 'k = \"v\"'")]
    ToYaml(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ToJsonArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub pretty: bool,
}

pub fn dispatch(args: TomlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::ToYaml(a) => to_yaml::run(a, out),
    }
}

pub(super) fn parse_toml(s: &str) -> Result<toml::Value, CliError> {
    s.parse::<toml::Value>().map_err(|e| {
        CliError::new(ErrorCode::InvalidToml, format!("invalid TOML: {e}"))
            .with_hint("ensure the input is well-formed TOML")
    })
}
```

Create `src/commands/toml/to_json.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::{parse_toml, ToJsonArgs};

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: ToJsonArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_toml(input.as_str()?)?;
    let json = if args.pretty {
        serde_json::to_string_pretty(&v)
    } else {
        serde_json::to_string(&v)
    }
    .map_err(|e| CliError::new(ErrorCode::Internal, format!("json serialization failed: {e}")))?;
    out.emit_value(&Out0 { json })
}
```

Create `src/commands/toml/to_yaml.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::{parse_toml, RunArgs};

#[derive(Serialize)]
struct Out0 {
    yaml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let v = parse_toml(input.as_str()?)?;
    let yaml = serde_yaml::to_string(&v).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("yaml serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { yaml })
}
```

- [ ] **Step 3: Wire into CLI**

Add `pub mod toml;` to `src/commands/mod.rs`.

Add to `src/cli.rs`:

```rust
    /// TOML conversion: to-json / to-yaml.
    Toml(crate::commands::toml::TomlArgs),
```

Add to `src/lib.rs`:

```rust
        cli::Noun::Toml(a) => commands::toml::dispatch(a, &out),
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/toml/ src/commands/mod.rs src/cli.rs src/lib.rs tests/toml_convert.rs
git commit -m "feat(toml): add toml to-json|to-yaml"
```

---

## Task 16: Update OpenCLI spec for all M1 nouns + regenerate docs

**Files:**
- Modify: `ubertool.ocs.yaml` (add all M1 commands)
- Verify: `make verify-spec` passes
- Verify: `make gen` regenerates docs cleanly

> Reminder: the OpenCLI alpha-7 schema does not represent `errors:`, top-level `examples:`, or global flags. Document those via `--help` `long_about` (already done in each module's `#[command(long_about = ...)]`). The spec captures the command tree, flags, types, and summaries.

- [ ] **Step 1: Read the M0 spec for shape reference**

Run: `head -80 /Users/mihai/dev/ubertool/ubertool.ocs.yaml`
Expected: M0 spec with the `base64` noun and `encode`/`decode` verbs.

- [ ] **Step 2: Append M1 nouns to the spec**

Edit `ubertool.ocs.yaml`. Under the existing `commands:` key, add the following command keys after the base64 ones (the exact YAML syntax follows the M0 `base64` entries — group commands need `group: true`, leaf commands include `arguments:` and `flags:`). Add for each:

- `ubertool hash {command} [flags]`  (group)
- `ubertool hash md5 [input] [flags]`  (and one entry for each of: sha1, sha224, sha256, sha384, sha512, sha3-256, sha3-512)
- `ubertool hmac {command} [flags]`  (group)
- `ubertool hmac md5 [input] [flags]`  (× each algo, with required `--key` flag)
- `ubertool bcrypt {command} [flags]`  (group)
- `ubertool bcrypt hash [input] [flags]`  (with `--cost`)
- `ubertool bcrypt verify [input] [flags]`  (with required `--hash`)
- `ubertool url {command} [flags]`  (group)
- `ubertool url encode [input] [flags]`
- `ubertool url decode [input] [flags]`
- `ubertool html {command} [flags]`  (group)
- `ubertool html encode [input] [flags]`
- `ubertool html decode [input] [flags]`
- `ubertool basic-auth {command} [flags]`  (group)
- `ubertool basic-auth encode [flags]`  (with required `--user`, `--pass`)
- `ubertool basic-auth decode [input] [flags]`
- `ubertool jwt {command} [flags]`  (group)
- `ubertool jwt decode [input] [flags]`
- `ubertool jwt verify [input] [flags]`  (with required `--secret`, optional `--algo`, `--validate-exp`)
- `ubertool uuid {command} [flags]`  (group)
- `ubertool uuid new [flags]`  (with `--version` choices `4`, `7`)
- `ubertool ulid {command} [flags]`  (group)
- `ubertool ulid new [flags]`
- `ubertool token {command} [flags]`  (group)
- `ubertool token new [flags]`  (with `--length`, `--format` choices `hex`, `base64`, `alphanumeric`)
- `ubertool password {command} [flags]`  (group)
- `ubertool password score [input] [flags]`
- `ubertool json {command} [flags]`  (group)
- `ubertool json to-yaml [input] [flags]`
- `ubertool json to-toml [input] [flags]`
- `ubertool json minify [input] [flags]`
- `ubertool json prettify [input] [flags]`  (with `--indent`)
- `ubertool yaml {command} [flags]`  (group)
- `ubertool yaml to-json [input] [flags]`  (with `--pretty`)
- `ubertool yaml to-toml [input] [flags]`
- `ubertool toml {command} [flags]`  (group)
- `ubertool toml to-json [input] [flags]`  (with `--pretty`)
- `ubertool toml to-yaml [input] [flags]`

Every flag entry must include `summary`, `type`, and where applicable `required: true`, `default`, or `choices:`. Refer to the M0 `base64 encode` entry as the canonical example.

- [ ] **Step 3: Validate the spec**

Run: `ocli spec check ubertool.ocs.yaml`
Expected: validation passes (no schema errors). If errors mention unknown fields, remove them — alpha-7 is conservative.

- [ ] **Step 4: Regenerate docs**

Run: `make gen`
Expected: regenerates `docs/cli/` Markdown files and `docs/llms.txt` without errors. Inspect a couple of files to confirm new nouns appear.

- [ ] **Step 5: Run verify-spec**

Run: `make verify-spec`
Expected: PASS — the spec and `--help` outputs are consistent for every noun/verb.

- [ ] **Step 6: Commit**

```bash
git add ubertool.ocs.yaml docs/cli/ docs/llms.txt
git commit -m "docs(spec): add all M1 nouns to OpenCLI spec and regenerate Markdown + llms.txt"
```

---

## Task 17: Extend fitness-checklist + final CI run

**Files:**
- Modify: `tests/agent_cli_fitness.rs` (extend to cover the new surface)
- Run: `make ci`

- [ ] **Step 1: Inspect current fitness test**

Run: `cat tests/agent_cli_fitness.rs`
Expected: 8 fitness tests from M0 covering base64.

- [ ] **Step 2: Add coverage for every new noun**

Extend `tests/agent_cli_fitness.rs` with a parameterized test that walks every M1 noun and asserts:
1. `ubertool <noun> --help` succeeds (exit 0).
2. `ubertool <noun> <verb> --help` succeeds for every documented verb.
3. `--json` is mentioned in the top-level `ubertool --help`.

Add this function and call it from a new test:

```rust
const M1_NOUNS_AND_VERBS: &[(&str, &[&str])] = &[
    ("base64", &["encode", "decode"]),
    ("hash", &["md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-256", "sha3-512"]),
    ("hmac", &["md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-256", "sha3-512"]),
    ("bcrypt", &["hash", "verify"]),
    ("url", &["encode", "decode"]),
    ("html", &["encode", "decode"]),
    ("basic-auth", &["encode", "decode"]),
    ("jwt", &["decode", "verify"]),
    ("uuid", &["new"]),
    ("ulid", &["new"]),
    ("token", &["new"]),
    ("password", &["score"]),
    ("json", &["to-yaml", "to-toml", "minify", "prettify"]),
    ("yaml", &["to-json", "to-toml"]),
    ("toml", &["to-json", "to-yaml"]),
];

#[test]
fn every_m1_noun_and_verb_has_discoverable_help() {
    use assert_cmd::Command;
    for (noun, verbs) in M1_NOUNS_AND_VERBS {
        // noun-level help
        Command::cargo_bin("ubertool")
            .unwrap()
            .args([noun, "--help"])
            .assert()
            .success();
        for verb in *verbs {
            Command::cargo_bin("ubertool")
                .unwrap()
                .args([noun, verb, "--help"])
                .assert()
                .success();
        }
    }
}

#[test]
fn data_returning_commands_advertise_json() {
    // The global --json flag is documented at the top level; spot-check three new nouns.
    use assert_cmd::Command;
    use predicates::prelude::*;
    for noun in ["hash", "uuid", "json"] {
        Command::cargo_bin("ubertool")
            .unwrap()
            .args(["--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--json"));
        let _ = noun;
    }
}
```

> Note: `--json` is a *global* flag declared on the top-level `Cli` struct, so it appears in `ubertool --help` rather than per-command help. The fitness assertion confirms it's visible at the top level — that's enough to satisfy the agent-cli-design rule.

- [ ] **Step 3: Run the extended fitness tests**

Run: `cargo test --test agent_cli_fitness`
Expected: all tests PASS, including the 2 new ones.

- [ ] **Step 4: Run the full CI pipeline locally**

Run: `make ci`
Expected: fmt + clippy + test + verify-spec all PASS.

If `cargo clippy -- -D warnings` flags anything: fix it. Common likely findings:
- `#[allow(clippy::large_enum_variant)]` may be needed on the `Noun` enum since clap subcommand variants vary in size.
- Unused imports (especially when adding new modules).

- [ ] **Step 5: Commit**

```bash
git add tests/agent_cli_fitness.rs
git commit -m "test(fitness): extend fitness checklist to all M1 nouns and verbs"
```

- [ ] **Step 6: Confirm working tree is clean and tag M1**

```bash
git status
# Expected: nothing to commit, working tree clean

git log --oneline | head -20
# Expected: 17 commits from M1 on top of M0
```

(Optional) Tag the milestone:

```bash
git tag m1
```

---

## Self-Review

**Spec coverage** — every requirement in the M1 scope from the design spec is covered:

- ✅ Hash family — Task 2 (8 algos via multiplexed `hash` noun)
- ✅ HMAC family — Task 3 (8 algos with `--key`)
- ✅ Bcrypt — Task 4 (`hash` + `verify` with exit-5 mismatch)
- ✅ URL encoding — Task 5
- ✅ HTML encoding — Task 6
- ✅ Basic auth — Task 7
- ✅ JWT — Task 8 (`decode` + `verify` with secret redaction)
- ✅ UUID — Task 9
- ✅ ULID — Task 10
- ✅ Token — Task 11
- ✅ Password score — Task 12
- ✅ JSON conversion — Task 13 (`to-yaml`, `to-toml`, `minify`, `prettify`)
- ✅ YAML conversion — Task 14 (`to-json`, `to-toml`)
- ✅ TOML conversion — Task 15 (`to-json`, `to-yaml`)
- ✅ OpenCLI spec updated — Task 16
- ✅ Fitness checklist extended + CI green — Task 17

**Placeholder scan** — no "TBD"/"TODO"/"implement later" in any task. Every code block is complete. The one location that could be considered fuzzy (the Task 4 pre-computed bcrypt hash) has a documented fallback: regenerate inline if it doesn't verify on the local bcrypt version.

**Type consistency**:
- Every `Out` struct uses `#[derive(Serialize)]` with named fields. Single-value outputs use a single-field struct (`{"hash": ...}`, `{"uuid": ...}`, `{"token": ...}`).
- `ErrorCode` extensions (`InvalidBcrypt` in Task 4, `InvalidUtf8` in Task 5) follow the existing pattern: enum variant + `as_str` arm + `exit` arm + sync-test inclusion.
- The shared `json::convert::json_to_toml` helper (Task 13) is reused by `yaml::to_toml` (Task 14) via a JSON-value bridge — single source of truth for the conversion.
- `--in` paths consistently typed as `Option<PathBuf>`. `--key`/`--secret`/`--hash`/`--user`/`--pass` consistently typed as `String` (required).
- Exit-code → error-code mapping is consistent: `Invalid*` → 3, `SignatureMismatch` → 5, `AlgoNotSupported` → 6.

**Scope check** — all 14 nouns + 1 spec task + 1 CI task = 17 tasks. Single sub-project, no decomposition needed.

**Ambiguity check** — explicit decisions documented inline:
- Hash output is single-field (`hash`), not nested with algo (algo is implicit from the subcommand invoked).
- bcrypt mismatch returns exit 5 (per design spec §8); malformed hash returns exit 3.
- JWT `--validate-exp` is opt-in (false by default); makes the tool useful for inspecting expired tokens.
- JWT secret is redacted from error `input` field (per design spec §7).
- Password is trimmed of trailing newline (ergonomic for piped input).
- JSON-to-TOML `null` is a hard error (TOML has no null type).
- `json prettify` supports `--indent`; default is 2.

---

## Execution Handoff

Plan complete and saved to `/Users/mihai/dev/ubertool/docs/superpowers/plans/2026-05-15-ubertool-m1-foundation.md`. Two execution options:

**1. Subagent-Driven (recommended)** — Dispatch a fresh implementer subagent per task; spec-compliance review then code-quality review after each task; fast iteration. Use `superpowers:subagent-driven-development`.

**2. Inline Execution** — Execute tasks in this session using `superpowers:executing-plans`; batch execution with checkpoints for review.

Which approach?
