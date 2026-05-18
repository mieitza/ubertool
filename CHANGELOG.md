# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/mieitza/ubertool/releases/tag/v0.1.0
