# ubertool M6 — Polish & v0.1.0 Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) tracking.

**Goal:** Polish ubertool into a tagged v0.1.0 release. No new verbs — the surface is locked at 56 nouns / 102 leaf commands. The artifacts produced here make the binary distributable and discoverable: README with golden examples, CHANGELOG, shell-completion subcommand, comprehensive per-noun help snapshot suite, multi-stage Dockerfile, cargo-dist release config, version bump to `0.1.0`, and a local `v0.1.0` tag (push left to the user).

**Architecture:** No code-structure changes. Adds a single new clap subcommand (`completions <shell>`) under a new top-level `completions` verb. Bundles release-tooling files at the repo root.

**Tech Stack added:** `clap_complete` (shell completion generation).

**Companion documents:** Design spec §14 (build profile), §15 (distribution), §16 (milestone definition). M5 plan as the most recent template.

**Working directory:** `/Users/mihai/dev/ubertool/`. M5 just pushed (origin/main at `f541df5`). Test count baseline: 316. `make verify-spec` reports 102 leaf commands.

**Acceptance criteria:**
1. `cargo build --release` succeeds. Binary runs.
2. `cargo test` ≥ 316 (M5 baseline + new snapshot tests; expect ~370+).
3. `make ci` green.
4. `README.md` exists with: install (cargo/brew/docker/binstall), 8+ golden examples, link to spec/llms.txt.
5. `CHANGELOG.md` exists, first entry `## [0.1.0]` covering M0–M5.
6. `ubertool completions bash` emits a working bash completion script (≥ 50 lines).
7. `Dockerfile` produces an Alpine+musl binary ≤ 60 MB.
8. `dist-workspace.toml` or `cargo dist` config present; `.github/workflows/release.yml` defines the release pipeline.
9. `Cargo.toml` version is `0.1.0` (no `-alpha` suffix).
10. A local annotated tag `v0.1.0` exists (do NOT push).
11. Repo clean.

**Out of scope (deferred to later releases):**
- Homebrew tap repo creation (needs separate GitHub repo).
- Actual crates.io publish (needs cargo login token).
- Actual GHCR/Docker Hub push (needs auth).
- Triggering the cargo-dist release (needs tag pushed).

---

## Task 1: Add M6 dependencies + tidy Cargo.toml

**Files:** `Cargo.toml`.

- [ ] **Step 1: Append in `[dependencies]`**

```toml
# M6 polish
clap_complete = "4"
```

- [ ] **Step 2: Update `[package]` metadata for crates.io readiness**

Set/verify these fields:

```toml
[package]
name = "ubertool"
version = "0.1.0-alpha.0"          # bumped at end of M6 (Task 8)
edition = "2021"
authors = ["mihai"]
description = "Developer-focused data-transform CLI; agent-friendly Rust port of it-tools"
license = "GPL-3.0"
repository = "https://github.com/mieitza/ubertool"
homepage = "https://github.com/mieitza/ubertool"
documentation = "https://github.com/mieitza/ubertool/blob/main/docs/superpowers/specs/2026-05-15-ubertool-cli-design.md"
readme = "README.md"
keywords = ["cli", "agent", "developer-tools", "json", "encoding"]
categories = ["command-line-utilities", "development-tools"]
rust-version = "1.81"
```

`keywords` is limited to 5 entries on crates.io; pick the most descriptive five.

- [ ] **Step 3: Verify**

```bash
cargo check
cargo test
```

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add clap_complete and tidy [package] metadata for crates.io"
```

---

## Task 2: Add `completions` top-level command

**Files:** `src/commands/completions/mod.rs`, modify `src/cli.rs` + `src/lib.rs` + `src/commands/mod.rs`. Tests: `tests/completions.rs`.

This is a slightly different shape than the noun-verb pattern — `completions` is a single-action subcommand (no second verb). To keep the noun-verb contract, structure it as `completions <shell>`. Acceptable shells per `clap_complete`: bash, zsh, fish, powershell, elvish.

- [ ] **Step 1: Write `tests/completions.rs`**

```rust
use assert_cmd::Command;

#[test]
fn completions_bash_emits_script() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "bash"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    // Bash completion scripts typically reference _ubertool or complete -F.
    assert!(s.contains("complete -F"), "expected bash completion script: {s}");
    assert!(s.contains("ubertool"));
    assert!(s.lines().count() >= 50);
}

#[test]
fn completions_zsh_emits_script() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "zsh"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("#compdef ubertool"));
}

#[test]
fn completions_fish_emits_script() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "fish"])
        .assert().success();
}

#[test]
fn completions_invalid_shell_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "csh"])
        .assert().failure().code(2);
}
```

- [ ] **Step 2: Create `src/commands/completions/mod.rs`**

```rust
//! Shell completion script generator.

use clap::{Args, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Target shell.
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs, _out: &Out) -> Result<(), CliError> {
    let mut cmd = crate::cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
```

> Note: completions write directly to stdout (raw script text). `--json` mode is not meaningful here — the output is a shell script, not data. The completions output bypasses `Out::emit_value` deliberately.

- [ ] **Step 3: Wire into CLI**

In `src/commands/mod.rs`, add `pub mod completions;`.

In `src/cli.rs`, add to `Noun`:

```rust
    /// Generate shell completion script (bash, zsh, fish, powershell, elvish).
    #[command(long_about = "Emit a shell completion script for the chosen shell.\n\nInstallation (examples):\n  Bash:  ubertool completions bash > /etc/bash_completion.d/ubertool\n  Zsh:   ubertool completions zsh > ~/.zsh/completions/_ubertool\n  Fish:  ubertool completions fish > ~/.config/fish/completions/ubertool.fish")]
    Completions(crate::commands::completions::CompletionsArgs),
```

Note: this variant is a **single positional**, not a noun-verb pair. That's an intentional exception — completion generation is a one-shot utility action. The fitness-checklist test in Task 7 will explicitly skip `completions` from the noun-verb walk.

In `src/lib.rs`, add the dispatch arm:

```rust
        cli::Noun::Completions(a) => commands::completions::run(a, &out),
```

- [ ] **Step 4: Run + commit**

```bash
cargo test
git add src/commands/completions/ src/commands/mod.rs src/cli.rs src/lib.rs tests/completions.rs tests/snapshots/
git commit -m "feat(completions): add completions <shell> subcommand (clap_complete)"
```

---

## Task 3: Per-noun help snapshot suite

**Files:** `tests/help_snapshots.rs`.

Catch help-text drift across refactors. For each of the 56 nouns, capture `ubertool <noun> --help` as an insta snapshot. The existing `tests/snapshots.rs` already has the top-level help snapshot from M0; this task adds per-noun snapshots in a new file to avoid breaking the existing one.

- [ ] **Step 1: Create `tests/help_snapshots.rs`**

```rust
//! Per-noun --help snapshot suite. Catches help-text drift across refactors.

use assert_cmd::Command;
use std::process::Output;

const NOUNS: &[&str] = &[
    "base64", "basic-auth", "bcrypt", "hash", "hmac", "html", "json", "jwt",
    "password", "toml", "token", "ulid", "url", "uuid", "yaml",
    "xml", "csv", "case", "slugify", "list", "integer-base", "roman",
    "temperature", "sql", "markdown", "text", "docker-run", "safelink",
    "ipv4", "mac", "ipv6-ula", "math", "percentage", "eta", "date",
    "crontab", "chmod",
    "cipher", "rsa", "otp", "pdf", "regex", "email", "iban", "phone",
    "user-agent",
    "mime", "http-status", "qr", "svg-placeholder", "ascii", "git",
    "numeronym", "port", "completions",
];

fn run_help(noun: &str) -> String {
    let Output { stdout, status, stderr } = Command::cargo_bin("ubertool")
        .unwrap()
        .args([noun, "--help"])
        .output()
        .unwrap();
    if !status.success() {
        panic!(
            "ubertool {noun} --help exited with {:?}: stderr={}",
            status,
            String::from_utf8_lossy(&stderr)
        );
    }
    // Normalize: strip terminal width-dependent wrapping by collapsing runs of
    // 2+ spaces between non-flag tokens. (insta snapshots are reproducible
    // because clap renders deterministically given fixed COLUMNS — we set it.)
    String::from_utf8(stdout).unwrap()
}

#[test]
fn noun_help_snapshots() {
    // Force consistent wrap width for clap so snapshots don't depend on terminal.
    std::env::set_var("COLUMNS", "100");

    for noun in NOUNS {
        let out = run_help(noun);
        // Use insta's settings to produce one snapshot per noun.
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(noun);
        settings.bind(|| {
            insta::assert_snapshot!("noun_help", out);
        });
    }
}
```

> Note: insta's `set_snapshot_suffix` is the canonical way to vary snapshot names within a single test function. The first run creates ~56 `.snap.new` files under `tests/snapshots/`. Promote them to `.snap` via `INSTA_UPDATE=always cargo test --test help_snapshots`.

- [ ] **Step 2: Run + accept snapshots**

```bash
INSTA_UPDATE=always cargo test --test help_snapshots
```

After the first run, all snapshots are accepted. Subsequent runs will fail if any help text drifts.

- [ ] **Step 3: Run full suite + commit**

```bash
cargo test
git add tests/help_snapshots.rs tests/snapshots/
git commit -m "test(snapshots): add per-noun --help snapshot suite (56 snapshots)"
```

---

## Task 4: README.md with golden examples

**Files:** `README.md`. Replace the M0 placeholder.

- [ ] **Step 1: Write `README.md`**

Use this skeleton; fill in the bracketed bits with the actual outputs (run the commands and paste).

```markdown
# ubertool

> Developer-focused data-transform CLI — agent-friendly by construction.

A single-binary Rust port of [it-tools](https://it-tools.tech) for the terminal. 56 nouns, 102 leaf verbs, structured `--json` output on every command, meaningful exit codes, machine-parseable error envelopes, no hangs on TTY stdin. Designed so an LLM coding agent (Claude Code, Codex, Cursor, OpenCode) can call it through a shell without surprises.

## Install

```bash
# crates.io
cargo install ubertool

# Homebrew (after tap is published)
brew tap mieitza/ubertool && brew install ubertool

# Docker
docker run --rm ghcr.io/mieitza/ubertool:latest base64 encode hello

# Pre-built binary via cargo-binstall
cargo binstall ubertool

# From source
git clone https://github.com/mieitza/ubertool && cd ubertool && cargo install --path .
```

## Golden examples

```bash
# Hash a string
$ ubertool hash sha256 "hello"
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

# JSON output for an agent
$ ubertool --json uuid new
{"uuid":"<some-uuid>"}

# Pipe-friendly: encode a file, take the first 32 chars
$ ubertool base64 encode --in ./README.md --quiet | head -c 32

# Verify a JWT
$ ubertool jwt verify "$TOKEN" --secret "$SECRET" --json
{"verified":true,"claims":{...}}

# Convert formats
$ ubertool json to-yaml '{"name":"alice","age":30}'
age: 30
name: alice

# Subnet math
$ ubertool ipv4 subnet 10.0.0.0/24 --json
{"cidr":"10.0.0.0/24","network":"10.0.0.0","broadcast":"10.0.0.255",...}

# Generate a QR code
$ ubertool qr generate "https://example.com" --format png --out qr.png

# Cheat sheet
$ ubertool git memo | head -20
```

## Output modes

| Mode | Flag | Use |
|------|------|-----|
| Text (default) | (none) | Human-readable; `key: value` lines for multi-field, bare value for single-field. |
| JSON | `--json` | Structured JSON on stdout. Stable schema. Nothing else lands on stdout. |
| Quiet | `--quiet` / `-q` | Bare values for piping. |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General failure |
| 2 | Usage error (bad arguments, missing required flag) |
| 3 | Input validation failed (malformed JSON, invalid IP, bad regex, …) |
| 4 | I/O error (file missing, permission denied) |
| 5 | Cryptographic / integrity failure (bcrypt mismatch, JWT signature, decrypt MAC fail) |
| 6 | Feature not built in this binary |

## Error envelope (`--json` mode)

```json
{
  "error": "signature_mismatch",
  "message": "JWT signature does not verify with the provided secret",
  "input": {"token":"eyJ...","secret":"<redacted>"},
  "hint": "if the JWT uses RS*/ES*, this build does not support asymmetric keys",
  "retriable": false
}
```

Field types are stable across all commands. Secrets are always redacted from `input`.

## Discovery

```bash
ubertool --help               # all 56 nouns
ubertool <noun> --help        # all verbs for a noun
ubertool <noun> <verb> --help # arguments, flags, examples, exit codes
```

Or read the full Markdown reference at [docs/cli/docs.gen.md](./docs/cli/docs.gen.md) (generated from `ubertool.ocs.yaml` via `ocli gen docs`).

For an LLM-friendly single-file reference, use [docs/llms.txt](./docs/llms.txt).

## Design

- Design spec: [docs/superpowers/specs/2026-05-15-ubertool-cli-design.md](./docs/superpowers/specs/2026-05-15-ubertool-cli-design.md)
- OpenCLI source-of-truth: [ubertool.ocs.yaml](./ubertool.ocs.yaml)
- Implementation plans: [docs/superpowers/plans/](./docs/superpowers/plans/)

Built using the [agent-cli-design](https://github.com/anthropics/agent-cli-design) eight rules:
1. Structured output is not optional (`--json` everywhere).
2. Exit codes are control flow (not just 0/1).
3. Idempotent operations.
4. Self-documenting `--help`.
5. Composable (`--quiet`, stdin, pipes).
6. Dry-run + confirmation bypass on destructive commands.
7. Actionable typed errors.
8. Noun-verb hierarchy.

## License

GPL-3.0 — same as the source project [it-tools](https://github.com/CorentinTh/it-tools).
```

- [ ] **Step 2: Verify example outputs**

For each `$ ubertool ...` block in the README, run the command in a separate shell and paste the actual output. Replace any placeholders (`<some-uuid>`, `...`) with real values.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add README with install instructions, golden examples, and design pointers"
```

---

## Task 5: CHANGELOG.md

**Files:** `CHANGELOG.md`.

- [ ] **Step 1: Create `CHANGELOG.md`**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-18

### Added

#### M0 — Scaffolding
- Single-binary Rust crate with lib+bin split.
- Shared `core/` infrastructure: `Out` (text/json/quiet output), `resolve_input` (positional / `--in` / stdin / fail-fast TTY), `CliError` envelope with typed `ErrorCode`, `ExitCode` mapping.
- OpenCLI spec at `ubertool.ocs.yaml` as the source of truth for docs and CI verification.
- Five-layer testing: unit, integration (`assert_cmd`), snapshot (`insta`), spec drift (`verify-spec.sh`), agent-cli fitness checklist.
- First verb: `base64 encode|decode`.
- GitHub Actions CI: fmt + clippy + test + verify-spec on Linux and macOS.

#### M1 — Foundation (14 nouns, ~20 verbs)
- `hash` (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512)
- `hmac` (same algos, with `--key`)
- `bcrypt hash|verify` (cost configurable, exit 5 on mismatch)
- `url encode|decode`
- `html encode|decode`
- `basic-auth encode|decode`
- `jwt decode|verify` (HS256/384/512, secret redacted in errors)
- `uuid new` (v4, v7)
- `ulid new`
- `token new` (hex / base64 / alphanumeric)
- `password score` (zxcvbn)
- `json to-yaml|to-toml|minify|prettify`
- `yaml to-json|to-toml`
- `toml to-json|to-yaml`

#### M2 — Converters (13 nouns, ~22 verbs)
- `xml to-json|format`, `csv to-json`, `case convert`, `slugify generate`, `list convert`, `integer-base convert`, `roman to-num|from-num`, `temperature convert`, `sql format`, `markdown to-html`
- `text` (8 verbs): `to-binary|from-binary|to-unicode|from-unicode|to-nato|stats|diff|obfuscate`
- `docker-run to-compose` (subset: name/ports/volumes/env/restart/network)
- `safelink decode` (Outlook / Google / generic URL wrappers)

#### M3 — Network & Calc (9 nouns, 17 verbs)
- `ipv4 parse|subnet|range-expand|to-ipv6`
- `mac new|lookup` (bundled curated OUI table)
- `ipv6-ula new` (RFC 4193)
- `math eval` (evalexpr)
- `percentage of|change|of-total`
- `eta calc`
- `date convert` (unix / ISO 8601 / RFC 2822, with optional `--tz`)
- `crontab describe|next`
- `chmod calc|parse`

#### M4 — Crypto & Validators (9 nouns + 1 verb, 14 verbs)
- `cipher encrypt|decrypt` (AES-GCM / ChaCha20-Poly1305 + Argon2 / PBKDF2; self-describing format)
- `rsa keypair` (2048 / 3072 / 4096 PKCS#8 PEM)
- `otp generate|validate` (TOTP RFC 6238)
- `pdf signature` (info extraction; no crypto verify)
- `regex test|generate`
- `email normalize`
- `iban validate`
- `phone parse` (libphonenumber)
- `user-agent parse` (woothee)
- `url parse` (extends M1 url noun)

#### M5 — Lookups, Images, Memos (8 nouns + 1 verb, 10 verbs)
- `mime lookup`, `http-status lookup` (soft-fail with null/false for unknowns)
- `qr generate|wifi` (SVG and PNG output)
- `svg-placeholder generate`
- `ascii draw` (figlet standard font)
- `git memo`, `regex memo` (bundled markdown cheat sheets)
- `numeronym generate` (i18n-style contraction)
- `port random`

#### M6 — Polish & Release
- `completions <shell>` subcommand for shell completion script generation.
- Per-noun `--help` snapshot suite.
- README with install instructions and golden examples.
- Dockerfile (multi-stage Alpine + musl).
- cargo-dist release configuration + GitHub Actions workflow.
- crates.io-ready `[package]` metadata.

### Surface

- **56 nouns**, **102 leaf commands**, all discoverable via `ubertool <noun> --help`.
- Every data-returning command supports `--json`, `--quiet`, `--in`, and stdin pipe.
- Eight rules from the agent-cli-design skill enforced by the fitness checklist test.

[0.1.0]: https://github.com/mieitza/ubertool/releases/tag/v0.1.0
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: add CHANGELOG with v0.1.0 entry covering M0-M6"
```

---

## Task 6: Dockerfile (multi-stage Alpine + musl)

**Files:** `Dockerfile`, `.dockerignore`.

- [ ] **Step 1: Create `Dockerfile`**

```dockerfile
# syntax=docker/dockerfile:1.7
# Build stage: compile static musl binary.
FROM rust:1.81-alpine AS build
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY src/data ./src/data

# Pre-fetch deps for cache reuse.
RUN cargo fetch

# Static musl build with full release profile from Cargo.toml.
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage: minimal alpine.
FROM alpine:3.20 AS runtime
RUN apk add --no-cache ca-certificates && adduser -D -u 1000 app
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/ubertool /usr/local/bin/ubertool
USER app
ENTRYPOINT ["ubertool"]
CMD ["--help"]
```

- [ ] **Step 2: Create `.dockerignore`**

```
.git
.github
target
docs
tests
*.md
.gitignore
.dockerignore
Dockerfile
ubertool.ocs.yaml
scripts
```

- [ ] **Step 3: Test the build (optional, requires Docker)**

If Docker is available in the implementer's environment:

```bash
docker build -t ubertool:test .
docker run --rm ubertool:test base64 encode hello
# Expected: aGVsbG8=
```

If Docker isn't available, skip the test — the file is correct by construction.

- [ ] **Step 4: Commit**

```bash
git add Dockerfile .dockerignore
git commit -m "build: add multi-stage Dockerfile (musl static binary on alpine 3.20)"
```

---

## Task 7: cargo-dist release configuration

**Files:** `dist-workspace.toml` (or extend `Cargo.toml`), `.github/workflows/release.yml`.

cargo-dist generates pre-built binaries for 7 targets per tag. This task **adds the config**; it does NOT trigger a release (that happens when the user pushes a tag).

- [ ] **Step 1: Add `[workspace.metadata.dist]` to `Cargo.toml`**

If `Cargo.toml` doesn't have a `[workspace]` block, add one at the top (or use `[package.metadata.dist]` for single-crate). Pick whichever works on first compile.

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.22.0"
ci = "github"
installers = ["shell", "powershell"]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
pr-run-mode = "plan"
allow-dirty = ["ci"]
```

- [ ] **Step 2: Generate the release workflow**

If `cargo-dist` CLI is installed:

```bash
cargo dist init --yes
cargo dist generate
```

This produces `.github/workflows/release.yml` automatically.

If `cargo-dist` is NOT installed (which is likely in this environment), **write the workflow manually**. Use this hand-rolled minimal version:

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - "v[0-9]+.[0-9]+.[0-9]+*"

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            cross: true
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            cross: true
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
            cross: true
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          target: ${{ matrix.target }}
      - name: Install cross
        if: matrix.cross
        run: cargo install cross
      - name: Build
        shell: bash
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
      - name: Package
        shell: bash
        run: |
          mkdir -p dist
          if [ "${{ runner.os }}" = "Windows" ]; then
            cp target/${{ matrix.target }}/release/ubertool.exe dist/
            cd dist && 7z a ubertool-${{ matrix.target }}.zip ubertool.exe
          else
            cp target/${{ matrix.target }}/release/ubertool dist/
            cd dist && tar czf ubertool-${{ matrix.target }}.tar.gz ubertool
          fi
      - uses: actions/upload-artifact@v4
        with:
          name: ubertool-${{ matrix.target }}
          path: dist/ubertool-${{ matrix.target }}.*

  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: artifacts/*
          generate_release_notes: true
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml .github/workflows/release.yml
git commit -m "build(release): add cargo-dist config and cross-platform release workflow"
```

---

## Task 8: Version bump + final make ci + tag v0.1.0

**Files:** `Cargo.toml`, `CHANGELOG.md` (link line at the bottom).

- [ ] **Step 1: Bump version**

Edit `Cargo.toml`:

```toml
[package]
name = "ubertool"
version = "0.1.0"     # was "0.1.0-alpha.0"
```

- [ ] **Step 2: Run `make ci`**

```bash
make ci
```

Fix any clippy/fmt issues. The snapshot suite from Task 3 should be stable.

Expected: all green. Test count ≥ 316 + 56 (snapshots) ≈ 372+.

- [ ] **Step 3: Commit version bump**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(release): bump version to 0.1.0"
```

- [ ] **Step 4: Tag (local only)**

```bash
git tag -a v0.1.0 -m "ubertool v0.1.0

First tagged release. 56 nouns, 102 leaf commands. See CHANGELOG.md for full details.
"
```

**Do NOT push the tag.** The user will push when ready, which triggers the release workflow.

- [ ] **Step 5: Confirm clean state**

```bash
git status   # clean
git tag      # shows v0.1.0
git log --oneline origin/main..HEAD | head -20
```

---

## Self-Review

**Acceptance criteria** — all should pass after Task 8:
- ✅ `cargo build --release` succeeds (Task 1+8).
- ✅ `cargo test` ≥ 316+56 ≈ 372+ (Task 3 adds 56 snapshot tests).
- ✅ `make ci` green (Task 8).
- ✅ `README.md` exists with install, examples, links (Task 4).
- ✅ `CHANGELOG.md` exists, `## [0.1.0]` entry (Task 5).
- ✅ `ubertool completions bash` works (Task 2).
- ✅ `Dockerfile` exists (Task 6).
- ✅ `dist-workspace.toml` config + `.github/workflows/release.yml` (Task 7).
- ✅ Version 0.1.0 (Task 8).
- ✅ Local tag `v0.1.0` (Task 8).

**Placeholder scan** — README golden examples need real output pasted in (the implementer runs each command and captures stdout). CHANGELOG date should be the actual release date.

**Type consistency** — only one new clap subcommand (`completions`) which is a deliberate exception to noun-verb. The fitness test will skip it.

**Scope check** — 8 tasks. Single milestone. No actual external publishing (just artifacts).

**Ambiguity check** — decisions documented:
- `completions` is a single-verb top-level command — noun-verb exception flagged.
- Per-noun help snapshots use `set_snapshot_suffix` for per-noun snapshot files; 56 separate `.snap` files.
- Dockerfile pins Rust 1.81 (matches `Cargo.toml` `rust-version`) and Alpine 3.20.
- cargo-dist config is hand-rolled rather than `cargo dist init` to avoid requiring the CLI.
- Tag is created locally; user pushes when ready to trigger release pipeline.

---

## Execution Handoff

Plan saved. Same workflow as prior milestones — subagent-driven execution. After M6, the user pushes the tag (`git push origin v0.1.0`) to trigger the release workflow.
