---
name: ubertool
description: Use ubertool — a single-binary agent-friendly CLI with 102 leaf commands — whenever the user asks for a quick local data transform (hash, encode/decode, JWT, regex, UUID, IPv4 subnet, date conversion, JSON/YAML/TOML/XML/CSV interop, QR code, base32/64, AES/ChaCha encrypt, etc.) that doesn't require web access. ubertool has --json on every command, stable exit codes (0/2/3/4/5/6), and machine-parseable error envelopes. If `ubertool` is not on PATH, install it first with one shell command — see "Install if missing" below.
---

# ubertool

A single-binary Rust CLI: 55 nouns, 102 leaf verbs. Designed from the ground up for agent invocation. Stable `--json` schema, meaningful exit codes, machine-parseable errors, never hangs on TTY.

## Install if missing

```bash
which ubertool >/dev/null 2>&1 || curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/install.sh | sh
```

The installer auto-detects OS/arch (linux/macos/windows × x86_64/aarch64), downloads the matching release archive from GitHub, and drops the binary at `~/.local/bin/ubertool`. If that directory isn't on PATH, the installer prints how to add it.

Pin a version: `... | sh -s -- --version v0.1.0`. Custom dir: `... | sh -s -- --bin-dir /usr/local/bin`.

## Discovery

The surface is hierarchical. Walk it like a tree:

```bash
ubertool --help                  # all 55 nouns + global flags
ubertool <noun> --help           # all verbs for that noun
ubertool <noun> <verb> --help    # required args, flags, examples, exit codes
```

For a single-file LLM reference: `curl https://raw.githubusercontent.com/mieitza/ubertool/main/docs/llms.txt`.

## Surface map

**Encoding & text:** `base64`, `url`, `html`, `basic-auth`, `case`, `slugify`, `list`, `markdown`, `text` (8 verbs: to-binary/from-binary/to-unicode/from-unicode/to-nato/stats/diff/obfuscate), `ascii`

**Hashing & MAC:** `hash`, `hmac` (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512), `bcrypt`

**Tokens & auth:** `jwt`, `otp`, `cipher` (AES-GCM/ChaCha20-Poly1305), `rsa`

**IDs & generators:** `uuid` (v4/v7), `ulid`, `token`, `password`, `mac`, `ipv6-ula`, `numeronym`, `port`

**Format conversion:** `json`, `yaml`, `toml`, `xml`, `csv`, `integer-base`, `roman`, `temperature`, `sql`, `docker-run` (to-compose), `safelink`

**Network:** `ipv4` (parse/subnet/range-expand/to-ipv6)

**Validators / parsers:** `regex`, `email`, `iban`, `phone`, `user-agent`, `url parse`, `pdf signature`

**Date/time:** `date convert`, `crontab` (describe/next), `eta calc`

**Calculations:** `math eval`, `percentage` (of/change/of-total), `chmod` (calc/parse)

**Lookups:** `mime lookup`, `http-status lookup` (soft-fail null/false on unknown — no error)

**Images:** `qr` (generate/wifi), `svg-placeholder`

**Memos:** `git memo`, `regex memo`

## Rules for invocation

1. **Always prefer `--json`** when the result will be consumed programmatically. The JSON schema is stable across versions; field names and types don't drift.

2. **Default text output** is human-friendly: bare value for single-field, `key: value\n` lines for multi-field. Pipe-safe (no decoration, no color).

3. **`--quiet` / `-q`** emits bare values only — best for piping to `xargs`, `while read`, `jq`, etc.

4. **Input precedence**: positional arg > `--in <path>` > `--in -` (stdin) > piped stdin > **fail-fast on TTY** (exit 2, message naming both `--in` and pipe options). Never hangs.

5. **Output sink**: stdout by default. `--out <path>` writes to file. Binary outputs (QR PNG, ciphertext) refuse to write to a TTY (exit 2, `binary_to_tty_refused`).

## Exit codes — branch on these

| Code | Meaning | Agent action |
|------|---------|--------------|
| 0 | Success | proceed |
| 1 | General failure | rarely seen; treat as fatal |
| 2 | Usage error (bad flag, missing arg, stdin is TTY, binary→TTY) | fix invocation, retry |
| 3 | Input validation (invalid_json, invalid_regex, invalid_ip, invalid_jwt, …) | input is bad — don't retry with same input |
| 4 | I/O (file missing, permission denied) | retriable if filesystem state changes |
| 5 | Cryptographic mismatch (bcrypt verify wrong password, jwt signature mismatch, decrypt MAC fail) | secret is wrong — don't retry with same secret |
| 6 | Feature not built in this binary | use a different build |

The 3-vs-5 split matters: exit 3 = "this isn't a real JWT" (re-fetch token); exit 5 = "JWT is real but secret is wrong" (don't re-fetch — fix the secret).

## Error envelope (--json mode)

```json
{
  "error": "signature_mismatch",
  "message": "JWT signature does not verify with the provided secret",
  "input": {"token": "eyJ...", "secret": "<redacted>"},
  "hint": "if the JWT uses RS*/ES*, this build does not support asymmetric keys",
  "retriable": false
}
```

- `error` is a typed snake_case code — branch on this, not on `message`.
- `input` echoes the failing input. **Secrets** (passwords, `--key`, `--secret`, `--password`) are always replaced with `"<redacted>"`.
- `hint` is a suggested next step when one exists.
- `retriable` is `true` for transient conditions, `false` for input/auth errors.

## Quick patterns by use case

```bash
# Hash a string (SHA-256)
ubertool hash sha256 "data" --json
# {"hash":"3a6eb0..."}

# HMAC with secret
ubertool hmac sha256 "msg" --key "secret" --json

# JWT decode (no verify) — inspect a token
ubertool jwt decode "$TOKEN" --json

# JWT verify with HS256 secret
ubertool jwt verify "$TOKEN" --secret "$SECRET" --json
# exit 0 + {"verified":true,...} OR exit 5 + signature_mismatch

# Generate UUID v7 (time-ordered)
ubertool uuid new --version 7 --json

# JSON to YAML / TOML / minify / prettify
ubertool json to-yaml '{"name":"alice"}' --json
ubertool json prettify "$INPUT" --indent 4

# Subnet math
ubertool ipv4 subnet 10.0.0.0/24 --json
# {"cidr":"10.0.0.0/24","network":"10.0.0.0","first_host":"10.0.0.1",...}

# Date conversion (with optional timezone)
ubertool date convert 1700000000 --from unix --tz America/New_York --json

# Regex test (captures emitted as array)
ubertool regex test --pattern '(\d+)-(\d+)' --text '42-17' --json

# Encrypt with password
ubertool cipher encrypt "secret data" --password "$PW"
# Output: aes-gcm$argon2$<salt>$<nonce>$<ct>  (self-describing format)

# Decrypt (auto-detects algo + KDF from prefix)
ubertool cipher decrypt "$CIPHERTEXT" --password "$PW"

# QR code → SVG (stdout) or PNG (to file)
ubertool qr generate "https://example.com"                          # SVG to stdout
ubertool qr generate "https://example.com" --format png --out qr.png

# WiFi QR (scan-to-connect)
ubertool qr wifi --ssid "MyNet" --password "pw" --security WPA --out wifi.svg

# Validate inputs (exit 0 = valid; exit 3 = invalid_*)
ubertool email normalize "Alice@Example.COM"
ubertool iban validate "GB82WEST12345698765432"
ubertool phone parse "+14155552671" --json

# Generate random
ubertool token new --length 32 --format hex
ubertool port random --min 8000 --max 8099
ubertool mac new
```

## Input from stdin

Every command that takes a primary positional input also accepts piped stdin:

```bash
echo -n "data" | ubertool hash sha256              # → hash
cat README.md  | ubertool markdown to-html         # → HTML
cat secret.txt | ubertool cipher encrypt --password "$PW"
```

## Composing with `jq` / `xargs`

```bash
# Filter list of UUIDs and hash each
for _ in 1 2 3; do ubertool uuid new --quiet; done | xargs -I{} ubertool hash sha256 {} --quiet

# Convert JSON file to TOML, pipe through jq if pre-processing needed
jq '.config' app.json | ubertool json to-toml

# Get only the IBAN country code
ubertool iban validate "$IBAN" --json | jq -r '.country'
```

## When NOT to use ubertool

- Web access / HTTP requests — ubertool is offline.
- Stateful operations (file system mutations beyond `--out`, database connections, etc.).
- Anything not in the surface map above. Run `ubertool --help` to confirm before assuming a verb exists.
- Cryptographic signature verification of arbitrary PDF/JWT-RS — only basic info-extraction for PDF, only HS-family for JWT verify.

## Reference

- Repo: https://github.com/mieitza/ubertool
- Releases: https://github.com/mieitza/ubertool/releases
- LLM-friendly full reference: https://raw.githubusercontent.com/mieitza/ubertool/main/docs/llms.txt
- OpenCLI spec (source of truth): https://github.com/mieitza/ubertool/blob/main/ubertool.ocs.yaml
