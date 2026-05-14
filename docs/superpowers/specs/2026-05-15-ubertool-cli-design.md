# ubertool — Single-Binary Rust CLI Port of it-tools

**Status:** Design, awaiting implementation
**Date:** 2026-05-15
**Owner:** mihai
**Companion skills:** `agent-cli-design` (rules), `opencli-spec-author` (spec format), `superpowers:brainstorming` (this process)

---

## Goal

Port the data-transformation tools from the `it-tools` Vue web app into a single Rust binary called `ubertool` that an LLM coding agent (Claude Code, Codex, Cursor, OpenCode) can call reliably through a shell. Humans use it too, but every design choice is made for the agent first.

## Non-goals

- The browser app is not being retired. This is a sibling CLI, not a replacement.
- Interactive tools that are inherently UI-bound do not get ported (camera-recorder, html-wysiwyg-editor, benchmark-builder, chronometer, keycode-info, device-information, emoji-picker UI, color-picker UI, ASCII-drawer UI). A few of them have a non-interactive subset; that subset *is* ported (e.g., color conversion math, ASCII figlet output).
- Backwards compatibility with the web app's URL routes, query parameters, or i18n strings.
- A plugin runtime, dynamic tool loading, or workspace of crates. Single crate, single binary, closed set.

## Scope (~65 tools / ~75 leaf verbs)

The "clean CLI fit + awkward-but-doable" subset. The exact inventory is in §3 below.

---

## 1. Design rules in force

All eight rules from the `agent-cli-design` skill are binding:

1. Structured output (`--json`) on every data-returning command. JSON exclusively to stdout in `--json` mode; everything else to stderr. Flat over nested. Consistent types and field names across commands.
2. Meaningful exit codes (not just 0/1), documented in `--help`.
3. Idempotent operations. The vast majority of `ubertool` commands are pure functions, so this is automatic.
4. Self-documenting `--help`: summary, usage signature, flags with types, realistic examples, mention of `--json`, discoverable subcommand tree.
5. Composability: `--quiet` for bare pipe-friendly output, explicit stdin support, batch where natural.
6. `--dry-run` on destructive commands; `--yes` to bypass any confirmation; never hang on a TTY.
7. Errors include a typed code, echo the failing input, suggest next steps where possible, and indicate retriability.
8. Noun-verb hierarchy: `ubertool <noun> <verb>`, two levels, no exceptions.

These eight rules, encoded into the OpenCLI spec, are *also* enforced by integration tests (see §10).

---

## 2. CLI-vs-MCP justification

The agent-cli-design skill's decision table flags ">15 commands → reconsider MCP." `ubertool` has ~65. Every other factor still points to CLI:

| Factor | Verdict |
|---|---|
| State between calls | Stateless data transforms — CLI |
| Agent has shell access | Yes — CLI |
| Token budget matters | 65 MCP tool defs would cost tens of thousands of tokens *per turn* — CLI |
| Reliability | No server lifecycle — CLI |
| "Would a human reach for a CLI here?" | Yes — CLI |

The discoverability cost of a large command surface is mitigated by the noun-verb structure: the agent enumerates `ubertool --help` → ~30 nouns → 1–3 verbs each. It never needs all 65 in context at once; it pulls `--help` on demand.

---

## 3. Tool inventory

~75 verbs across ~30 nouns. Grouped by family:

### Encoding & text
```
ubertool base64 encode|decode          ubertool url encode|decode
ubertool html encode|decode            ubertool text to-binary|from-binary
ubertool text to-unicode|from-unicode  ubertool text to-nato
ubertool text obfuscate                ubertool text stats
ubertool text diff                     ubertool case <style>
ubertool slugify                       ubertool list convert
ubertool markdown to-html              ubertool ascii draw
```

### Hashing & MAC
```
ubertool hash <algo>          # md5, sha1, sha224, sha256, sha384, sha512, sha3-256, sha3-512
ubertool hmac <algo>          # same algos
ubertool bcrypt hash|verify
```

### Tokens & auth
```
ubertool jwt decode|verify             ubertool basic-auth encode|decode
ubertool otp generate|validate         ubertool token new
```

### IDs & generators
```
ubertool uuid new                      ubertool ulid new
ubertool bip39 new|to-entropy|from-entropy   ubertool password score
ubertool mac new                       ubertool port random
ubertool ipv6-ula new                  ubertool numeronym
```

### Conversion
```
ubertool json to-yaml|to-toml|to-xml|to-csv|minify|prettify|diff
ubertool yaml to-json|to-toml|view
ubertool toml to-json|to-yaml
ubertool xml to-json|format
ubertool csv to-json
ubertool integer-base                  ubertool roman to-num|from-num
ubertool temperature                   ubertool sql format
ubertool docker-run to-compose         ubertool safelink decode
```

### Network
```
ubertool ipv4 parse|subnet|range-expand|to-ipv6
ubertool mac lookup
```

### Crypto
```
ubertool encrypt                       ubertool decrypt
ubertool rsa keypair                   ubertool pdf signature
```

### Validators / parsers
```
ubertool regex test|generate           ubertool email normalize
ubertool iban validate                 ubertool phone parse
ubertool user-agent parse              ubertool url parse
```

### Lookups
```
ubertool mime lookup                   ubertool http-status
ubertool chmod calc|parse
```

### Calculations
```
ubertool math eval                     ubertool percentage of|change|of-total
ubertool eta calc                      ubertool date convert
ubertool crontab describe|next
```

### Images & SVG
```
ubertool qr generate                   ubertool qr wifi
ubertool svg-placeholder
```

### Memos (cheat sheets)
```
ubertool git memo                      ubertool regex memo
```

### Out of scope (not portable)
camera-recorder, html-wysiwyg-editor, benchmark-builder, chronometer, keycode-info, device-information, emoji-picker (interactive picker), color-converter (UI is the value; conversion math is trivial enough to omit), json-viewer/yaml-viewer (the in-app tree explorer is the value; `json prettify` covers the static case).

---

## 4. Command surface conventions

**Binary:** `ubertool`.

**Shape:** `ubertool <noun> <verb> [args] [flags]`. Two depths. No exceptions.

**Canonical flag names** (one name, used identically everywhere they apply):

| Flag | Meaning |
|---|---|
| `--in <path>` | Read input from file |
| `--out <path>` | Write output to file |
| `--format <fmt>` | Output format selector when a command supports multiple (e.g., `qr generate --format png\|svg`) |
| `--algo <name>` | Algorithm when it is a parameter rather than a verb (e.g., `encrypt --algo aes-gcm`) |
| `--key <string>`, `--secret <string>` | Symmetric key / secret |
| `--cost <int>` | Bcrypt cost factor |
| `--json` | Switch stdout to JSON (boolean) |
| `--pretty` | Force human formatting (colors/tables) on stdout when not `--json` |
| `--quiet`, `-q` | Bare values only |
| `--dry-run` | Preview destructive action without performing it |
| `--yes`, `--force` | Bypass any confirmation |
| `--verbose`, `-v` | Verbose progress to stderr |
| `--help`, `-h` | Help |
| `--version` | Version |

Algorithms appear as positional verbs when they govern the command (`ubertool hash sha256 "hello"`) and as flags when they are one parameter among others (`ubertool encrypt --algo aes-gcm --key … "hello"`). This rule is consistent across the surface.

---

## 5. Output contract

### Default (no `--json`)
- Single-value tools: just the value to stdout, no label, no trailing newline beyond one.
- Multi-field results: stable `key: value\n` lines on stdout.
- No colors, no headers, no decoration. Pipe-safe.

### `--json`
- Data goes flat to stdout. **Nothing else ever lands on stdout** in this mode. Progress, warnings, verbose logging — stderr only.
- Errors flip the document shape (see §7) but still go to stdout.

### `--pretty`
- TTY-detected on stdout by default; can be forced.
- Colors and tables allowed on stdout *only when not `--json`*. `--json` and `--pretty` together is allowed only to mean "JSON to stdout, plus human echo to stderr."

### `--quiet` / `-q`
- For list-shaped outputs: bare values, one per line, no headers.
- For single-value outputs: identical to default. (Quiet is meaningful when there is something to suppress.)

### Examples
```
$ ubertool base64 encode "hello"
aGVsbG8=

$ echo -n "hello" | ubertool base64 encode
aGVsbG8=

$ ubertool base64 encode "hello" --json
{"encoded":"aGVsbG8="}

$ ubertool hash sha256 --in ./file.bin
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

$ ubertool ipv4 subnet 10.0.0.0/24 --json
{"cidr":"10.0.0.0/24","network":"10.0.0.0","broadcast":"10.0.0.255",
 "first_host":"10.0.0.1","last_host":"10.0.0.254","host_count":254,
 "mask":"255.255.255.0","prefix":24}

$ ubertool qr generate "https://example.com" --out qr.png
# (silent on success; exit 0; file written)
```

---

## 6. Input resolution

A single rule used identically by every command that takes a primary input:

1. Positional arg present → use it as the literal value.
2. No positional + `--in <path>` → read from file.
3. No positional + `--in -` or stdin is a pipe → read stdin.
4. No positional, no `--in`, stdin is a TTY → **fail with exit 2 and a message naming both options.** Never hang.

Output sink: stdout by default. `--out <path>` writes to file. Binary outputs (PNG, PDF) require either an explicit `--out` or a piped (non-TTY) stdout — dumping binary into a terminal raises exit 2 with the typed error `binary_to_tty_refused`.

---

## 7. Error envelope

### Without `--json` (default)
- Message to **stderr**, formatted as `error: <code>: <message>` plus an optional `hint: <remediation>` line.
- Nothing on stdout.
- Non-zero exit.

### With `--json`
- Error JSON to **stdout** (per agent-cli-design Rule 7 — agent reads stdout for structured errors too).
- Human message also to stderr.
- Non-zero exit.

```
$ ubertool jwt verify --secret wrong eyJhbGc...
error: signature_mismatch: JWT signature does not verify with the provided secret
hint: confirm the secret is correct and matches the alg in the JWT header
# exit 5

$ ubertool jwt verify --secret wrong eyJhbGc... --json
{
  "error": "signature_mismatch",
  "message": "JWT signature does not verify with the provided secret",
  "input": {"alg":"HS256","kid":null},
  "hint": "confirm the secret is correct and matches the alg in the JWT header",
  "retriable": false
}
# stderr: error: signature_mismatch: JWT signature does not verify...
# exit 5
```

Canonical error JSON field names:

| Field | Type | Meaning |
|---|---|---|
| `error` | string (snake_case) | Typed error code |
| `message` | string | Human-readable message |
| `input` | any (optional) | Echo of the failing input — never raw secrets |
| `hint` | string (optional) | Suggested next step |
| `retriable` | boolean | Whether retrying with the same inputs could plausibly succeed |

`input` is filtered: secrets (`--secret`, `--key`, password values) are replaced with the literal string `"<redacted>"` even when echoed.

---

## 8. Exit codes

| Code | Name | Meaning | Example |
|---|---|---|---|
| 0 | `Ok` | Success | — |
| 1 | `Generic` | Unexpected failure | panic, internal error |
| 2 | `Usage` | Bad CLI invocation | unknown flag, missing required arg, stdin is TTY with no input, binary-to-TTY refusal |
| 3 | `Invalid` | Input validation failed | malformed JSON/YAML/TOML/XML, regex won't compile, IBAN check digit fails, JWT structure malformed |
| 4 | `Io` | Filesystem / I/O failure | `--in` path missing, `--out` path not writable, permission denied |
| 5 | `Crypto` | Cryptographic / integrity failure | bcrypt verify mismatch, JWT signature invalid, decrypt MAC fail, PDF signature invalid |
| 6 | `Unsupported` | Feature not compiled in this build | command exists in spec but not in this binary because its Cargo feature was excluded |

The split between 3 (input bad) and 5 (crypto integrity bad) is load-bearing: an agent retrying `jwt verify` after exit 3 should re-fetch the token (it didn't parse); after exit 5 it should not (it parsed fine — the secret is wrong). Different recovery paths.

Each command's `--help` lists only the exit codes it can actually raise. The top-level `ubertool --help` lists all seven.

---

## 9. The OpenCLI spec as source of truth

`ubertool.ocs.yaml` lives at the repo root. It declares every command, flag, argument, and error code with `summary`, `type`, and (where applicable) `choices`.

`make gen` runs:
1. `ocli validate --spec-file ubertool.ocs.yaml` — schema check.
2. `ocli gen docs --spec-file ubertool.ocs.yaml --output-dir docs/cli --format markdown --dryrun=false` — Markdown docs.
3. Concatenate `docs/cli/**.md` → `docs/llms.txt` for agent discovery.
4. `scripts/verify-spec.sh` — diffs `ubertool --help` and `ubertool <noun> --help` against the spec to catch drift.

`ocli` has no Rust generator as of `1.0.0-alpha.7`, so step (4) substitutes for code generation. The spec is still the contract: flag names, exit codes, examples, and summaries are reviewed at the spec level, and Rust code must match.

The Cargo build depends on `make verify-spec` having passed in CI.

---

## 10. Project layout

Single crate, single binary, modules grouped by noun, with a small shared `core` layer that every command depends on.

```
ubertool/
├── Cargo.toml
├── ubertool.ocs.yaml          # OpenCLI spec — source of truth
├── Makefile                   # gen, build, test, verify-spec
├── README.md  LICENSE
├── src/
│   ├── main.rs                # entry; dispatches to commands::dispatch
│   ├── cli.rs                 # top-level clap App (Parser derive)
│   ├── core/
│   │   ├── mod.rs
│   │   ├── output.rs          # OutputMode, emit_json/emit_text/emit_quiet
│   │   ├── input.rs           # resolve_input(arg, --in, stdin) + TTY fail-fast
│   │   ├── error.rs           # CliError, ErrorCode
│   │   ├── exit.rs            # ExitCode (repr u8)
│   │   └── tty.rs             # is_stdout_tty, is_stdin_tty wrappers
│   ├── commands/
│   │   ├── mod.rs             # Noun enum; dispatch table
│   │   ├── base64/  hash/  hmac/  bcrypt/  jwt/  uuid/  ulid/
│   │   ├── json/  yaml/  toml/  xml/  csv/
│   │   ├── ipv4/  mac/  encrypt/  rsa/  pdf/
│   │   ├── regex/  email/  iban/  phone/  user_agent/  url/
│   │   ├── mime/  http_status/  chmod/  math/  percentage/
│   │   ├── date/  crontab/  qr/  svg_placeholder/  ascii/
│   │   └── …                  # one folder per noun (~30)
│   └── data/                  # bundled lookup tables (include_bytes!/include_str!)
│       ├── oui.csv
│       ├── http_status.json
│       ├── mime_types.json
│       └── nato.json
├── tests/
│   ├── helpers/               # assert_cmd setup, golden-file harness
│   ├── snapshots/             # insta — help text, JSON schemas
│   └── *.rs                   # one integration file per noun
├── benches/                   # criterion — hash, json, regex hot paths
├── fixtures/                  # test inputs: sample JWT, signed PDF, CSV, …
├── docs/cli/                  # generated by `ocli gen docs`
├── docs/llms.txt              # concatenated for agent discovery
├── scripts/
│   ├── verify-spec.sh         # diff `ubertool --help` ↔ spec
│   └── release.sh
└── .github/workflows/
    ├── ci.yml                 # fmt + clippy + test + spec-verify
    └── release.yml            # cross-compile via cargo-dist
```

Average size per noun: ~150 LOC across `mod.rs` + 1–N verb files. No file should exceed ~400 LOC; if one does, the noun is doing too much and gets split.

### Shared infrastructure

`src/core/error.rs`:
```rust
pub struct CliError {
    pub code: ErrorCode,
    pub message: String,
    pub input: Option<serde_json::Value>,
    pub hint: Option<String>,
    pub retriable: bool,
}

pub enum ErrorCode {
    UsageError, InvalidJson, InvalidYaml, InvalidToml, InvalidXml,
    InvalidRegex, InvalidIban, InvalidPhone, InvalidJwt,
    FileNotFound, PermissionDenied, BinaryToTtyRefused,
    SignatureMismatch, DecryptFailed, PdfSignatureInvalid,
    AlgoNotSupported,
    // ... extend as needed
}

impl ErrorCode {
    pub fn exit(&self) -> ExitCode { /* one place mapping codes -> exits */ }
}
```

`src/core/output.rs`:
```rust
pub enum OutputMode { Text, Json, Quiet }

pub fn emit_value<T: Serialize>(value: &T, mode: OutputMode) -> Result<(), CliError>;
pub fn emit_text(s: &str);                 // newline-terminated
pub fn emit_lines<I>(it: I) where I: Iterator<Item = String>;
pub fn emit_binary(bytes: &[u8], out: &Path) -> Result<(), CliError>;
```

`src/core/input.rs`:
```rust
pub enum Input { Bytes(Vec<u8>), Reader(Box<dyn Read>) }

pub fn resolve_input(
    positional: Option<&str>,
    in_path: Option<&Path>,
    stdin_required: bool,
) -> Result<Input, CliError>;
```

`src/core/exit.rs`:
```rust
#[repr(u8)]
pub enum ExitCode {
    Ok = 0, Generic = 1, Usage = 2, Invalid = 3,
    Io = 4, Crypto = 5, Unsupported = 6,
}
```

`main` is the only place that calls `process::exit`. Every command returns `Result<CommandOutput, CliError>`; `main` translates via `ErrorCode::exit()`.

---

## 11. Crate choices

| Family | Crates | Notes |
|---|---|---|
| Encoding | `base64`, `percent-encoding`, `html-escape` | All small |
| Hashing | `sha2`, `sha1`, `md-5`, `sha3`, `digest`, `hmac` | `digest` traits unify `hash <algo>` and `hmac <algo>` |
| Bcrypt | `bcrypt` | Small |
| JWT | `jsonwebtoken` | HS/RS/ES |
| UUID/ULID | `uuid` (v1/v3/v4/v5/v7 features), `ulid` | Small |
| BIP39 | `bip39` | Wordlists bundled |
| Password score | `zxcvbn` (Rust port) | ~500 KB dictionary |
| JSON/YAML/TOML/XML/CSV | `serde`, `serde_json`, `serde_yaml`, `toml`, `quick-xml`, `csv` | Cross-conversion via `serde_json::Value` interchange |
| SQL | `sqlformat` | Pure Rust |
| Markdown | `pulldown-cmark` | ~200 KB |
| Slug/case | `slug`, `convert_case` | Small |
| Roman / temperature / integer-base | hand-written | Trivial |
| Docker-run → Compose | hand-written port of `composerize-ts` algorithm (~400 LOC) | No good Rust crate |
| Regex | `regex`, `regex-generate` | Rust's `regex` lacks lookaround — flagged in `regex test --help` |
| Email | `email_address` | Validate-only is enough for `email normalize` |
| IBAN | `iban_validate` | Small |
| Phone | `phonenumber` | **~3 MB of libphonenumber data — biggest single contributor to binary size** |
| User-agent | `woothee` | ~1 MB regex DB |
| URL | `url` | Small |
| Network | `ipnet`, `std::net` | Small |
| MAC lookup | bundled `oui.csv`, lazy `HashMap` via `OnceLock` | ~1 MB CSV |
| Encryption | `aes-gcm`, `chacha20poly1305`, `argon2`, `pbkdf2` | Self-describing output format `<algo>$<kdf>$<salt>$<nonce>$<ciphertext>` |
| RSA | `rsa`, `pkcs1`, `pkcs8` | PEM in/out |
| OTP | `totp-rs` | Small |
| PDF signature | `lopdf` + hand-written signature-dict extraction | No end-to-end crate |
| QR | `qrcode`, `image` | SVG built into `qrcode`; `image` for PNG (~2 MB) |
| ASCII (figlet) | `figlet-rs` | Bundles standard font |
| Crontab | `cron` + hand-written describer (~200 LOC port) | No good `cron-descriptor` crate |
| Math eval | `evalexpr` | ~150 KB |
| Date | `chrono`, `chrono-tz` | `chrono-tz` adds ~1 MB zoneinfo |
| Random | `rand`, `rand_chacha` | Small |

**Total binary size estimate (release + LTO + opt-z + strip):**
- Full build: **~35–50 MB**.
- Minimal build (encoding + hashing + text + json/yaml): **~5 MB**.

Top size contributors are flagged above. None of them is gratuitous, but a user who needs only encoders/hashers can build a 5 MB binary via the `minimal` feature.

---

## 12. Feature gates

Default = `full`, but every family is feature-gated so users can opt out:

```toml
[features]
default = ["full"]
full = ["encoding","hash","conversion","ids","crypto","network","qr","phone","user-agent","pdf","ascii","math","date","markdown"]
minimal = ["encoding","hash","conversion"]    # ~5 MB
encoding = []
hash = ["dep:sha2","dep:sha1","dep:md-5","dep:sha3","dep:digest","dep:hmac","dep:bcrypt"]
conversion = ["dep:serde_json","dep:serde_yaml","dep:toml","dep:quick-xml","dep:csv","dep:sqlformat"]
ids = ["dep:uuid","dep:ulid","dep:bip39"]
crypto = ["dep:aes-gcm","dep:chacha20poly1305","dep:argon2","dep:pbkdf2","dep:rsa","dep:jsonwebtoken","dep:totp-rs"]
network = ["dep:ipnet"]
qr = ["dep:qrcode","dep:image"]
phone = ["dep:phonenumber"]
user-agent = ["dep:woothee"]
pdf = ["dep:lopdf"]
ascii = ["dep:figlet-rs"]
math = ["dep:evalexpr"]
date = ["dep:chrono","dep:chrono-tz"]
markdown = ["dep:pulldown-cmark"]
```

Commands whose feature is excluded **still appear in `--help`** (marked `(not built)`), and return exit `6 Unsupported` with `error: algo_not_supported` plus a `hint` naming the feature to enable. Agent-friendly: the surface is consistent across builds; only behavior changes.

---

## 13. Testing strategy

Five layers, each catching a specific failure class:

1. **Unit tests** — `#[test]` in each `commands/<noun>/<verb>.rs`. Pure-function semantics. Property tests via `proptest` for invertible pairs (encode↔decode, to-binary↔from-binary, json↔yaml round-trips on supported subsets).

2. **Integration tests** — `tests/<noun>.rs` using `assert_cmd` + `predicates`. Every command, every reachable exit code. Each `--json`-supporting command has an assertion that stdout is `serde_json::from_slice`-parseable in `--json` mode (enforces Rule 1 sub-rule mechanically).

3. **Snapshot tests** — `insta` for `ubertool --help` and each noun's `--help`. Help drift across refactors becomes a deliberate reviewable change.

4. **Spec verification** — `scripts/verify-spec.sh` parses `ubertool.ocs.yaml` and walks `ubertool <noun> [<verb>] --help`, confirming every flag, every documented exit code, and every example in the spec exists in `--help`. Fails CI on drift.

5. **Fitness-checklist test** — `tests/agent_cli_fitness.rs` is one integration test that runs the 13 items from `agent-cli-design/fitness-checklist.md` programmatically:
   - Every data-returning command lists `--json` in `--help`.
   - No command exits `0` on a documented error path.
   - Every command with destructive side effects supports `--dry-run`.
   - Field name `id` (when present) is always `id`, never `<noun>_id`.
   - Etc.

   This is the agent-design contract enforced by code, not review discipline.

Coverage gate: 80% line coverage via `cargo-llvm-cov` in CI.

---

## 14. Build profile

```toml
[profile.release]
lto = "fat"
codegen-units = 1
opt-level = "z"
strip = true
panic = "abort"
```

`cargo build --release` produces the distributable binary. `cargo build --release --no-default-features --features minimal` produces the small build.

---

## 15. Distribution

| Channel | Mechanism |
|---|---|
| Crates.io | `cargo install ubertool` |
| GitHub Releases | `cargo-dist` cross-compiles seven targets (linux gnu/musl × x86_64/aarch64, darwin × x86_64/aarch64, windows × x86_64) per tag |
| `cargo-binstall` | Out-of-the-box from `cargo-dist` release artifacts |
| Homebrew | Tap repo with auto-generated formula; `brew tap <GH_OWNER>/ubertool && brew install ubertool`. `<GH_OWNER>` is the GitHub owner of the new repo — fixed at repo-creation time (see §17). |
| Docker | `ghcr.io/<GH_OWNER>/ubertool:latest` — alpine + static musl binary, ~50 MB image |
| Shell completions | Generated via `clap_complete` at release time, shipped as `_ubertool` (zsh), `ubertool.bash`, `ubertool.fish` |

CI: GitHub Actions. PR CI runs fmt + clippy + test + spec-verify. Release workflow gated on git tags, runs cross-compile + sign + upload + Homebrew formula PR.

---

## 16. Implementation milestones

Detailed task breakdown is deferred to the `superpowers:writing-plans` skill that runs after this design is approved. High-level phases:

| Milestone | Scope | Approx duration |
|---|---|---|
| **M0 — Scaffolding** | Cargo skeleton; `core/` (output, input, error, exit, tty); clap top-level; one tool wired end-to-end (`base64 encode\|decode`) as the canonical template; OpenCLI spec v0; `ocli validate` + `gen docs` working; `verify-spec.sh`; CI fmt/clippy/test/spec-verify; **all five test layers exercised on this one tool**. | ~2 days |
| **M1 — Tier 1 (foundation)** | Hashing (hash + hmac + bcrypt), encoding (url, html), basic-auth, JWT, UUID, ULID, token, password score; JSON↔YAML↔TOML; json minify/prettify. ~20 verbs. | ~1–2 wk |
| **M2 — Tier 2 (converters)** | XML, CSV, case, slugify, list, integer-base, roman, temperature, sql-format, markdown-to-html, text-to-* (binary/unicode/nato), text-stats, text-diff, text-obfuscate, docker-run→compose, safelink. ~20 verbs. | ~2 wk |
| **M3 — Network & calc** | ipv4 (parse/subnet/range/v6 map), mac (new/lookup), ipv6-ula, math, percentage, eta, date convert, crontab (describe/next), chmod. ~12 verbs. | ~1 wk |
| **M4 — Crypto & validators** | encrypt/decrypt, rsa keypair, otp, pdf signature, regex (test/generate), email, iban, phone, user-agent, url. ~12 verbs. | ~1–2 wk |
| **M5 — Lookups, images, memos** | mime, http-status, qr (generate/wifi), svg-placeholder, ascii draw, git memo, regex memo, numeronym, random port. ~10 verbs. | ~1 wk |
| **M6 — Polish & v0.1.0** | Full snapshot suite, `llms.txt`, README with golden examples, Homebrew tap, Docker, cargo-dist release. | ~1 wk |

**Total: ~6–8 weeks** for a v0.1 that ships every portable tool.

Each milestone:
- Gets its own PR.
- Adds its commands and tests.
- Updates `ubertool.ocs.yaml` and re-runs `make gen`.
- Passes the fitness-checklist integration test against the new surface.
- Is reviewable in isolation.

---

## 17. Open questions for follow-up planning

These do not block design approval but need answers during `writing-plans`:

- Repo layout: does `ubertool` live in the existing `it-tools` repo as a sibling top-level directory, or in a new dedicated repo? Recommendation: new repo (`ubertool` cargo project), with a link from `it-tools`'s README. The build toolchain, CI, and language are different enough that combining doesn't pay rent.
- GitHub owner (`<GH_OWNER>` in §15) for the new repo / Homebrew tap / GHCR namespace — needs to be set before any release artifacts can be published.
- Versioning: align with `it-tools` calver (`2026.05.x`) or use semver from `0.1.0`? Recommendation: semver — the spec contract changes independently of the web app.
- License: GPL-3.0 to match `it-tools`, or MIT/Apache-2.0 dual-license to match Rust ecosystem norms? Recommendation: GPL-3.0, same as the source project.
- Telemetry: none. The web app uses plausible-tracker; the CLI ships zero phone-home.

---

## 18. References

- `~/.claude/skills/agent-cli-design/SKILL.md` — the eight rules
- `~/.claude/skills/agent-cli-design/fitness-checklist.md` — the test source-of-truth
- `~/.claude/skills/agent-cli-design/help-text-template.md` — `--help` skeleton
- `~/.claude/skills/opencli-spec-author/SKILL.md` — spec authoring guide
- Source project: `it-tools` (this repo), `src/tools/*` for the existing tool inventory
- Article underlying the design rules: Ugo Enyioha, "Writing CLI Tools That AI Agents Actually Want to Use" (dev.to, Feb 2025)
