# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-05-25

Patch release — vault unlock chain order corrected to match the locked design.

### Fixed

- **`vault`**: password resolution now checks the OS keyring **before** the
  session cache (was: session cache → keyring). The keyring is the persistent
  "I trust this machine" assertion made at `vault init`; the session cache is
  a short-lived fallback for headless / no-keyring environments. Order
  matters because in interactive use the keyring is the user's intended
  primary unlock path; checking the session cache first masked it whenever
  `vault unlock` had been run more recently than the daemon could supply via
  keyring. Functional behavior is unchanged for any environment that uses
  only one of the two (CI with no keyring, or a workstation that never runs
  `vault unlock`).

### Documentation

- `vault --help`, README, and `docs/claude-skill/SKILL.md` updated to match
  the corrected chain order.
- Skill now includes a `jq` snippet for filtering `ubertool schema` output to
  leaf commands only (the raw `.commands | keys` list also contains group
  placeholders like `ubertool hash {command} [flags]`; the leaf-only count
  is 114).

[0.4.1]: https://github.com/mieitza/ubertool/releases/tag/v0.4.1

## [0.4.0] - 2026-05-25

Agent-experience + hardening release.

### Added

- **`vault`** — encrypted local secrets store. 9 verbs (`init`, `set`, `get`,
  `list`, `delete`, `export`, `import`, `unlock`, `lock`). AES-256-GCM payload
  with Argon2id key derivation; vault file defaults to
  `~/.config/ubertool/vault.enc`. Master password unlock chain: OS keyring
  (macOS Keychain / Linux Secret Service / Windows Credential Manager) →
  session cache → `UBERTOOL_VAULT_PASSWORD` env var → interactive TTY prompt.
  Standalone tool — use `--secret "$(ubertool vault get my-key)"` to feed
  secrets into other commands.
- **`schema [--noun <name>]`** — emits the full ubertool command surface as
  JSON in a single call, so an agent can introspect all 56 nouns / 104 verbs
  without walking dozens of `--help` pages.
- **`--batch` JSONL mode** on `hash`, `hmac`, `base64 encode`/`decode`,
  `url encode`/`decode`, `html encode`/`decode`. Reads newline-delimited input
  from stdin, emits one JSON object per line. Closes the "Batch operations
  exist for bulk work" gap on the agent-cli-design fitness checklist.

### Fixed

- **xml**: `parse_xml_to_value` no longer panics on a stray closing tag
  (e.g. raw `</root>` input). Surfaced by cargo-fuzz; now returns
  `invalid_xml` cleanly.

### Test infrastructure

- **proptest round-trip property tests** — 9 invariants (base64 / url / html
  encode-decode, text to/from binary, text to/from unicode, json minify
  idempotence, integer-base round-trip, roman round-trip). 64 generated cases
  per property.
- **cargo-fuzz** — `fuzz/` workspace with 5 parser targets (json, yaml, toml,
  xml, csv). `make fuzz-smoke` runs each for 30 s.
- **Coverage gate** — `cargo-llvm-cov` Makefile targets and a CI job that
  fails the build if line / function / region coverage drops below
  `COVERAGE_MIN` (default 75; current baseline 83% lines, 77% functions, 80%
  regions).

[0.4.0]: https://github.com/mieitza/ubertool/releases/tag/v0.4.0

## [0.3.0] - 2026-05-20

Feature-gap release — fills four scope-downs from earlier milestones.

### Added

- **`jwt verify` asymmetric algorithms** — RS256/384/512 and ES256/384 via
  `--key-file <pem>` (a PEM public key). HS* still uses `--secret`. Wrong
  algo/key combination is a usage error.
- **`regex test --fancy`** — opt into the `fancy-regex` engine for lookaround
  (`(?=...)`, `(?<=...)`) and backreferences (`\1`), which Rust's default
  `regex` crate does not support.

### Changed

- **`mac lookup`** now ships the full IEEE OUI MA-L registry (~39,400 vendor
  entries, gzip-bundled and decompressed lazily) instead of the 53-entry
  curated stub.
- **`pdf signature --verify`** — cryptographically verifies RSA PKCS#7/CMS
  detached signatures: confirms the ByteRange digest matches the signed
  `messageDigest` attribute and that the RSA signature over the signed
  attributes is valid against the embedded signer certificate. Reports
  `verified: true|false|null` (null = non-RSA, not yet supported). This is an
  integrity check — it does NOT validate the certificate trust chain. Without
  `--verify`, the command's M4 info-extraction behavior is unchanged.

## [0.2.0] - 2026-05-20

Self-improvement release. **56 nouns, 104 leaf commands.**

### Added

- `self version` — print the running binary's version.
- `self update` — update the binary in place from the latest GitHub release
  (`self_update` crate). `--check` reports current vs latest without installing.
- `install.sh --with-completions` — auto-detects the shell from `$SHELL` and
  installs the completion script to the conventional location (bash / zsh /
  fish); `--completions-dir` overrides the target directory.
- One-line installer (`install.sh`) and Claude agent skill
  (`docs/claude-skill/SKILL.md`), with `install.sh --with-claude-skill` to
  install both. End-to-end skill test guide at `docs/claude-skill/TESTING.md`.

## [0.1.0] - 2026-05-18

First tagged release. **55 nouns, 102 leaf commands.**

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
- `phone parse` (libphonenumber via phonenumber crate)
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
- Per-noun `--help` snapshot suite (55 snapshots).
- README with install instructions and golden examples.
- Dockerfile (multi-stage Alpine + musl).
- cargo-dist release configuration + GitHub Actions release workflow.
- crates.io-ready `[package]` metadata.

### Surface

- **55 nouns**, **102 leaf commands**, all discoverable via `ubertool <noun> --help`.
- Every data-returning command supports `--json`, `--quiet`, `--in`, and stdin pipe.
- Eight rules from the agent-cli-design skill enforced by the fitness checklist test.

[0.4.0]: https://github.com/mieitza/ubertool/releases/tag/v0.4.0
[0.3.0]: https://github.com/mieitza/ubertool/releases/tag/v0.3.0
[0.2.0]: https://github.com/mieitza/ubertool/releases/tag/v0.2.0
[0.1.0]: https://github.com/mieitza/ubertool/releases/tag/v0.1.0
