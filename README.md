# ubertool

> Developer-focused data-transform CLI — agent-friendly by construction.

A single-binary Rust port of [it-tools](https://it-tools.tech) for the terminal. **56 nouns, 114 leaf verbs**, structured `--json` output on every command, meaningful exit codes, machine-parseable error envelopes, no hangs on TTY stdin. Designed so an LLM coding agent (Claude Code, Codex, Cursor, OpenCode) can call it through a shell without surprises.

## Install

```bash
# One-line installer (recommended) — downloads the matching pre-built binary
# from GitHub Releases for your OS/arch (linux/macos/windows × x86_64/aarch64).
curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/install.sh | sh

# Pin a version
curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/install.sh | sh -s -- --version v0.1.0

# Install binary + the Claude agent skill
curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/install.sh | sh -s -- --with-claude-skill

# From crates.io (once published)
cargo install ubertool

# From source
git clone https://github.com/mieitza/ubertool && cd ubertool && cargo install --path .
```

The installer drops the binary at `~/.local/bin/ubertool`. Override with `--bin-dir /usr/local/bin` (may require `sudo`). A Homebrew tap and crates.io publish are on the roadmap.

### Use with Claude (and other agents)

ubertool ships an agent skill at [`docs/claude-skill/SKILL.md`](./docs/claude-skill/SKILL.md). With `--with-claude-skill` above the installer drops it at `~/.claude/skills/ubertool/SKILL.md` so Claude Code automatically picks it up and knows how to invoke ubertool (discovery, exit codes, error envelope, common patterns). Manual install:

```bash
mkdir -p ~/.claude/skills/ubertool
curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/docs/claude-skill/SKILL.md \
  -o ~/.claude/skills/ubertool/SKILL.md
```

## Golden examples

```bash
# Hash a string
$ ubertool hash sha256 "hello"
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

# JSON output for an agent (UUID v4 is non-deterministic; format is stable)
$ ubertool --json uuid new
{"uuid":"73cec104-5e9b-4169-8853-f83de78a911e"}

# Convert JSON to YAML
$ ubertool json to-yaml '{"name":"alice","age":30}'
age: 30
name: alice

# Subnet math
$ ubertool ipv4 subnet 10.0.0.0/24 --json
{"cidr":"10.0.0.0/24","network":"10.0.0.0","broadcast":"10.0.0.255","first_host":"10.0.0.1","last_host":"10.0.0.254","host_count":254,"mask":"255.255.255.0","prefix":24}

# Numeronym (i18n-style)
$ ubertool numeronym generate internationalization
i18n

# Case conversion
$ ubertool case convert "hello world" --style snake
hello_world

# Integer base conversion
$ ubertool integer-base convert 255 --from 10 --to 16
ff

# Percentage of
$ ubertool percentage of 20 50
10

# Secrets vault
$ ubertool vault init
$ ubertool vault set MY_KEY
$ ubertool vault get MY_KEY
<value you stored>

# Schema introspection (first 3 keys as example)
$ ubertool schema --json | jq '[.commands | keys[0:3]]'
[["ubertool ascii draw <input> [flags]","ubertool ascii table [flags]","ubertool base64 decode [input] [flags]"]]
```

(All examples in this README run against the binary built from this commit. They're checked by the per-noun help snapshot suite in `tests/help_snapshots.rs`.)

## Output modes

| Mode | Flag | Use |
|------|------|-----|
| Text (default) | (none) | Human-readable; bare value for single-field, `key: value` lines for multi-field. |
| JSON | `--json` | Structured JSON on stdout. Stable schema. Nothing else lands on stdout. |
| Quiet | `--quiet` / `-q` | Bare values for piping. |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General failure |
| 2 | Usage error (bad arguments, missing required flag) |
| 3 | Input validation failed (malformed JSON, invalid IP, bad regex, ...) |
| 4 | I/O error (file missing, permission denied) |
| 5 | Cryptographic / integrity failure (bcrypt mismatch, JWT signature, decrypt MAC fail) |
| 6 | Feature not built in this binary |

## Error envelope (`--json` mode)

```json
{
  "error": "signature_mismatch",
  "message": "JWT signature does not verify with the provided secret",
  "input": {"token": "eyJ...", "secret": "<redacted>"},
  "hint": "if the JWT uses RS*/ES*, this build does not support asymmetric keys",
  "retriable": false
}
```

Field types are stable across all commands. Secrets are always redacted from `input`.

## Discovery

```bash
ubertool --help                  # all 56 nouns
ubertool <noun> --help           # all verbs for a noun
ubertool <noun> <verb> --help    # arguments, flags, examples, exit codes
```

For an LLM-friendly single-file reference, see `docs/llms.txt` (generated from `ubertool.ocs.yaml` via `ocli gen docs`).

## Schema introspection

Agents and scripts can introspect the entire command surface in one call — no need to parse multiple `--help` pages:

```bash
ubertool schema --json | jq '.commands | keys'   # list all command signatures
ubertool schema --noun hash                        # narrow to a single noun
```

This is the preferred agent-discovery entry point when surveying ubertool's capabilities programmatically.

## Shell completions

```bash
# Install (bash example)
ubertool completions bash > /etc/bash_completion.d/ubertool

# Other shells: zsh, fish, powershell, elvish
ubertool completions zsh > ~/.zsh/completions/_ubertool
ubertool completions fish > ~/.config/fish/completions/ubertool.fish
```

## Secrets vault

ubertool ships an encrypted local secrets store backed by AES-256-GCM and Argon2id. Use it to keep API keys and credentials out of shell history and plaintext files.

```bash
ubertool vault init                  # create the vault (once)
ubertool vault set MY_API_KEY        # store a secret (value is prompted)
ubertool vault get MY_API_KEY        # retrieve it

# Feed into another command
ubertool hmac sha256 "payload" --key "$(ubertool vault get MY_API_KEY)"
```

Password is resolved from: OS keyring → session cache → `UBERTOOL_VAULT_PASSWORD` env → interactive prompt. For headless/CI flows, set `UBERTOOL_VAULT_PASSWORD`. For interactive sessions, run `ubertool vault unlock --ttl 30` once. See [`docs/claude-skill/SKILL.md`](./docs/claude-skill/SKILL.md) for the full agent-friendly usage guide.

## Design

- Design spec: [`docs/superpowers/specs/2026-05-15-ubertool-cli-design.md`](./docs/superpowers/specs/2026-05-15-ubertool-cli-design.md)
- OpenCLI source-of-truth: [`ubertool.ocs.yaml`](./ubertool.ocs.yaml)
- Implementation plans: [`docs/superpowers/plans/`](./docs/superpowers/plans/)
- Generated CLI reference: [`docs/cli/docs.gen.md`](./docs/cli/docs.gen.md)

Built around the eight rules of [agent-cli-design](https://github.com/anthropics/agent-cli-design):

1. Structured output is not optional (`--json` everywhere).
2. Exit codes are control flow (not just 0/1).
3. Idempotent operations.
4. Self-documenting `--help`.
5. Composable (`--quiet`, stdin, pipes).
6. Dry-run + confirmation bypass on destructive commands.
7. Actionable typed errors.
8. Noun-verb hierarchy.

## Test coverage

Run `make coverage-summary` for a quick stdout report (line / function / region percentages). Run `make coverage` for a browsable HTML report at `target/llvm-cov/html/`. CI gates non-regression at `COVERAGE_MIN`% (currently 75%) — `make coverage-gate` exits non-zero if any of line / function / region coverage drops below that threshold.

Current baseline: **83.08% lines, 77.49% functions, 79.53% regions**. The design spec target is 80% across all dimensions; function and region coverage are the gap to close.

| Target | What it does |
|--------|-------------|
| `make coverage` | HTML report at `target/llvm-cov/html/` |
| `make coverage-summary` | Prints line/function/region % to stdout |
| `make coverage-gate` | Exits 1 if any coverage dimension < `COVERAGE_MIN` (default 75) |

Override the gate locally: `make coverage-gate COVERAGE_MIN=77`.

## License

GPL-3.0 — same as the source project [it-tools](https://github.com/CorentinTh/it-tools).
