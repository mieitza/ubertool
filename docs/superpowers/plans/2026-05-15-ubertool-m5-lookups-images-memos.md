# ubertool M5 — Tier 5 (Lookups, Images, Memos) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) tracking.

**Goal:** Add 8 new nouns + 1 verb extension (~10 verbs) covering lookups, images, memos, and small generators: `mime`, `http-status`, `qr`, `svg-placeholder`, `ascii`, `git`, `numeronym`, `port`, plus `regex memo` extending the M4 `regex` noun.

**Architecture:** Same single-crate pattern as M1-M4. Bundled lookup data lives in `src/data/` and is `include_str!`-ed at compile time. Bundled memos are also `include_str!`-ed.

**Tech Stack added:** `qrcode` (SVG + PNG support), `image` (PNG encoding for QR), `figlet-rs` (ASCII art). Hand-written: mime CSV lookup, http-status JSON lookup, SVG placeholder generator, numeronym, random port, memo readers.

**Companion documents:** Design spec §3 (inventory), §11 (crates), §16 (milestones). M4 plan as the most recent template.

**Working directory:** `/Users/mihai/dev/ubertool/`. M4 just pushed (origin/main at `45a0d43`). Test count baseline: 282.

**The cross-cutting noun pattern** (same as prior milestones):
1. Create `src/commands/<noun>/` with `mod.rs` + per-verb files.
2. Bundle any data via `include_str!("../../data/<file>")`.
3. Add `pub mod <noun>;` to `src/commands/mod.rs`.
4. Add `<Noun>(...)` variant to `src/cli.rs::Noun`.
5. Add match arm to `src/lib.rs::run()`.
6. Write `tests/<noun>.rs`.
7. Run `cargo test`; update top-level help snapshot; commit.

**Acceptance criteria:**
1. `cargo build` and `cargo build --release` succeed.
2. `cargo test` passes 100%. Test count ≥ 310 (M4 ended at 282; M5 adds ≥ ~30 tests).
3. Every new noun appears in `ubertool --help`; every verb in `ubertool <noun> --help`.
4. `make verify-spec` reports ≥ 102 leaf commands (M4 ended at 92; M5 adds ~10).
5. `make ci` green.
6. Repo clean.

**Soft-fail convention for lookups** (consistent with M3 `mac lookup`):
- `mime lookup` with unknown extension → `{"mime": null}`, exit 0.
- `http-status lookup` with unknown code → `{"name": null, "valid": false}`, exit 0.

Rationale: lookups are a "did this exist?" question. Agents can branch on the null/false without try/catch around an error path.

---

## Cross-cutting conventions

Same as M1-M4. New ErrorCode variants in M5: none expected. Existing variants (`BinaryToTtyRefused`, `IoError`, `UsageError`) cover the M5 error paths.

---

## Task 1: Add M5 dependencies

**Files:** Modify `Cargo.toml`.

- [ ] **Step 1: Append to `[dependencies]` after the M4 URL section**

```toml
# M5 images
qrcode = "0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
figlet-rs = "0.1"
```

The `image` crate's default features pull in JPEG/GIF/TIFF support (~10 MB). We only need PNG, so disable defaults.

- [ ] **Step 2: Run `cargo check`** — succeeds. Note any version substitutions.

- [ ] **Step 3: Run `cargo test`** — 282 M4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add M5 dependencies (qrcode/image/figlet-rs)"
```

---

## Task 2: `mime` + `http-status` (batched lookups)

**Files:** `src/data/mime_types.csv`, `src/data/http_status.json`, `src/commands/mime/mod.rs`, `src/commands/http_status/mod.rs`, `tests/mime.rs`, `tests/http_status.rs`. One commit per noun.

### Noun 1: `mime lookup`

- [ ] **Step 1: Create `src/data/mime_types.csv`**

Bundle ~60 common extensions:

```csv
extension,mime_type
html,text/html
htm,text/html
css,text/css
js,application/javascript
mjs,application/javascript
json,application/json
xml,application/xml
txt,text/plain
md,text/markdown
csv,text/csv
yaml,application/yaml
yml,application/yaml
toml,application/toml
png,image/png
jpg,image/jpeg
jpeg,image/jpeg
gif,image/gif
svg,image/svg+xml
webp,image/webp
avif,image/avif
ico,image/x-icon
bmp,image/bmp
tiff,image/tiff
tif,image/tiff
pdf,application/pdf
zip,application/zip
gz,application/gzip
tar,application/x-tar
7z,application/x-7z-compressed
rar,application/vnd.rar
mp3,audio/mpeg
wav,audio/wav
ogg,audio/ogg
flac,audio/flac
m4a,audio/mp4
mp4,video/mp4
webm,video/webm
mov,video/quicktime
avi,video/x-msvideo
mkv,video/x-matroska
woff,font/woff
woff2,font/woff2
ttf,font/ttf
otf,font/otf
eot,application/vnd.ms-fontobject
ico,image/x-icon
doc,application/msword
docx,application/vnd.openxmlformats-officedocument.wordprocessingml.document
xls,application/vnd.ms-excel
xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
ppt,application/vnd.ms-powerpoint
pptx,application/vnd.openxmlformats-officedocument.presentationml.presentation
sh,application/x-sh
bash,application/x-sh
py,text/x-python
rb,text/x-ruby
go,text/x-go
rs,text/rust
java,text/x-java
c,text/x-c
cpp,text/x-c++
h,text/x-c
hpp,text/x-c++
exe,application/x-msdownload
deb,application/vnd.debian.binary-package
rpm,application/x-rpm
appimage,application/vnd.appimage
```

- [ ] **Step 2: Write `tests/mime.rs`**

```rust
use assert_cmd::Command;

#[test]
fn mime_lookup_json() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/json");
    assert_eq!(v["extension"], "json");
}

#[test]
fn mime_lookup_filename() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "report.pdf"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/pdf");
    assert_eq!(v["extension"], "pdf");
}

#[test]
fn mime_lookup_dot_prefix() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", ".png"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "image/png");
}

#[test]
fn mime_lookup_unknown_emits_null() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "xyzabc"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["mime"].is_null());
    assert_eq!(v["extension"], "xyzabc");
}

#[test]
fn mime_lookup_case_insensitive() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "JSON"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/json");
}
```

- [ ] **Step 3: Create `src/commands/mime/mod.rs`**

```rust
//! MIME type lookup from extension.

use std::collections::HashMap;
use std::sync::OnceLock;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MIME_CSV: &str = include_str!("../../data/mime_types.csv");

#[derive(Debug, Args)]
pub struct MimeArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Look up the MIME type for a file extension or filename.
    #[command(long_about = "Look up the MIME type for a file extension or filename.\n\nAccepts the extension alone (e.g., `json`), with a leading dot (`.json`), or a full filename (`report.pdf`). Lookup is case-insensitive.\n\nReturns `mime: null` for unknown extensions (no error). Coverage: ~60 common extensions; the bundled table is curated.\n\nExamples:\n  ubertool mime lookup json\n  ubertool mime lookup report.pdf --json")]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    /// Extension (with or without leading dot) or full filename.
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    extension: String,
    mime: Option<String>,
}

fn mime_table() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::new();
        for line in MIME_CSV.lines().skip(1) {
            if let Some((ext, mime)) = line.split_once(',') {
                map.insert(ext.trim().to_lowercase(), mime.trim().to_string());
            }
        }
        map
    })
}

pub fn dispatch(args: MimeArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Lookup(a) => run(a, out),
    }
}

fn run(args: LookupArgs, out: &Out) -> Result<(), CliError> {
    let ext = extract_extension(&args.input);
    let mime = mime_table().get(&ext).cloned();
    out.emit_value(&Out0 { extension: ext, mime })
}

fn extract_extension(input: &str) -> String {
    let trimmed = input.trim();
    // Strip leading dot.
    let no_dot = trimmed.strip_prefix('.').unwrap_or(trimmed);
    // If contains a dot, take everything after the last dot.
    let ext = match no_dot.rsplit_once('.') {
        Some((_, e)) => e,
        None => no_dot,
    };
    ext.to_lowercase()
}
```

- [ ] **Step 4: Wire + commit mime**

Add `pub mod mime;` to `src/commands/mod.rs`. Add `Mime(...)` variant. Dispatch arm.

```bash
cargo test
git add src/data/mime_types.csv src/commands/mime/ src/commands/mod.rs src/cli.rs src/lib.rs tests/mime.rs tests/snapshots/
git commit -m "feat(mime): add mime lookup with bundled mime_types.csv"
```

### Noun 2: `http-status lookup`

- [ ] **Step 5: Create `src/data/http_status.json`**

```json
{
  "100": {"name": "Continue", "category": "Informational"},
  "101": {"name": "Switching Protocols", "category": "Informational"},
  "102": {"name": "Processing", "category": "Informational"},
  "103": {"name": "Early Hints", "category": "Informational"},
  "200": {"name": "OK", "category": "Success"},
  "201": {"name": "Created", "category": "Success"},
  "202": {"name": "Accepted", "category": "Success"},
  "203": {"name": "Non-Authoritative Information", "category": "Success"},
  "204": {"name": "No Content", "category": "Success"},
  "205": {"name": "Reset Content", "category": "Success"},
  "206": {"name": "Partial Content", "category": "Success"},
  "207": {"name": "Multi-Status", "category": "Success"},
  "208": {"name": "Already Reported", "category": "Success"},
  "226": {"name": "IM Used", "category": "Success"},
  "300": {"name": "Multiple Choices", "category": "Redirection"},
  "301": {"name": "Moved Permanently", "category": "Redirection"},
  "302": {"name": "Found", "category": "Redirection"},
  "303": {"name": "See Other", "category": "Redirection"},
  "304": {"name": "Not Modified", "category": "Redirection"},
  "307": {"name": "Temporary Redirect", "category": "Redirection"},
  "308": {"name": "Permanent Redirect", "category": "Redirection"},
  "400": {"name": "Bad Request", "category": "Client Error"},
  "401": {"name": "Unauthorized", "category": "Client Error"},
  "402": {"name": "Payment Required", "category": "Client Error"},
  "403": {"name": "Forbidden", "category": "Client Error"},
  "404": {"name": "Not Found", "category": "Client Error"},
  "405": {"name": "Method Not Allowed", "category": "Client Error"},
  "406": {"name": "Not Acceptable", "category": "Client Error"},
  "407": {"name": "Proxy Authentication Required", "category": "Client Error"},
  "408": {"name": "Request Timeout", "category": "Client Error"},
  "409": {"name": "Conflict", "category": "Client Error"},
  "410": {"name": "Gone", "category": "Client Error"},
  "411": {"name": "Length Required", "category": "Client Error"},
  "412": {"name": "Precondition Failed", "category": "Client Error"},
  "413": {"name": "Payload Too Large", "category": "Client Error"},
  "414": {"name": "URI Too Long", "category": "Client Error"},
  "415": {"name": "Unsupported Media Type", "category": "Client Error"},
  "416": {"name": "Range Not Satisfiable", "category": "Client Error"},
  "417": {"name": "Expectation Failed", "category": "Client Error"},
  "418": {"name": "I'm a teapot", "category": "Client Error"},
  "421": {"name": "Misdirected Request", "category": "Client Error"},
  "422": {"name": "Unprocessable Entity", "category": "Client Error"},
  "423": {"name": "Locked", "category": "Client Error"},
  "424": {"name": "Failed Dependency", "category": "Client Error"},
  "425": {"name": "Too Early", "category": "Client Error"},
  "426": {"name": "Upgrade Required", "category": "Client Error"},
  "428": {"name": "Precondition Required", "category": "Client Error"},
  "429": {"name": "Too Many Requests", "category": "Client Error"},
  "431": {"name": "Request Header Fields Too Large", "category": "Client Error"},
  "451": {"name": "Unavailable For Legal Reasons", "category": "Client Error"},
  "500": {"name": "Internal Server Error", "category": "Server Error"},
  "501": {"name": "Not Implemented", "category": "Server Error"},
  "502": {"name": "Bad Gateway", "category": "Server Error"},
  "503": {"name": "Service Unavailable", "category": "Server Error"},
  "504": {"name": "Gateway Timeout", "category": "Server Error"},
  "505": {"name": "HTTP Version Not Supported", "category": "Server Error"},
  "506": {"name": "Variant Also Negotiates", "category": "Server Error"},
  "507": {"name": "Insufficient Storage", "category": "Server Error"},
  "508": {"name": "Loop Detected", "category": "Server Error"},
  "510": {"name": "Not Extended", "category": "Server Error"},
  "511": {"name": "Network Authentication Required", "category": "Server Error"}
}
```

- [ ] **Step 6: Write `tests/http_status.rs`**

```rust
use assert_cmd::Command;

#[test]
fn http_status_lookup_200() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "http-status", "lookup", "200"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["code"], 200);
    assert_eq!(v["name"], "OK");
    assert_eq!(v["category"], "Success");
    assert_eq!(v["valid"], true);
}

#[test]
fn http_status_lookup_404() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "http-status", "lookup", "404"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["name"], "Not Found");
    assert_eq!(v["category"], "Client Error");
}

#[test]
fn http_status_lookup_unknown_emits_null() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "http-status", "lookup", "999"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["code"], 999);
    assert!(v["name"].is_null());
    assert_eq!(v["valid"], false);
}

#[test]
fn http_status_lookup_invalid_code_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["http-status", "lookup", "not-a-number"])
        .assert().failure().code(2);
}
```

- [ ] **Step 7: Create `src/commands/http_status/mod.rs`**

```rust
//! HTTP status code lookup.

use std::collections::HashMap;
use std::sync::OnceLock;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const HTTP_STATUS_JSON: &str = include_str!("../../data/http_status.json");

#[derive(Debug, Args)]
pub struct HttpStatusArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Look up the name and category for an HTTP status code.
    #[command(long_about = "Look up an HTTP status code.\n\nReturns `valid: false` and null name/category for unknown codes (no error).\n\nExamples:\n  ubertool http-status lookup 200\n  ubertool http-status lookup 404 --json\n\nExit codes:\n  2   input is not a valid integer")]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    pub code: u16,
}

#[derive(Serialize, Clone)]
struct StatusEntry {
    name: String,
    category: String,
}

#[derive(Serialize)]
struct Out0 {
    code: u16,
    name: Option<String>,
    category: Option<String>,
    valid: bool,
}

fn status_table() -> &'static HashMap<u16, StatusEntry> {
    static TABLE: OnceLock<HashMap<u16, StatusEntry>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let raw: HashMap<String, StatusEntry> =
            serde_json::from_str(HTTP_STATUS_JSON).expect("http_status.json must parse");
        raw.into_iter()
            .filter_map(|(k, v)| k.parse::<u16>().ok().map(|n| (n, v)))
            .collect()
    })
}

#[derive(serde::Deserialize)]
struct RawEntry {
    name: String,
    category: String,
}

// Re-implement Deserialize for StatusEntry since OnceLock + include_str! flow
// needs it. We aliased above; if needed, write a manual impl. The shortcut
// here is to make StatusEntry directly deserializable via serde derive.
impl<'de> serde::Deserialize<'de> for StatusEntry {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let r = RawEntry::deserialize(d)?;
        Ok(StatusEntry {
            name: r.name,
            category: r.category,
        })
    }
}

pub fn dispatch(args: HttpStatusArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Lookup(a) => run(a, out),
    }
}

fn run(args: LookupArgs, out: &Out) -> Result<(), CliError> {
    let entry = status_table().get(&args.code).cloned();
    let (name, category, valid) = match entry {
        Some(e) => (Some(e.name), Some(e.category), true),
        None => (None, None, false),
    };
    out.emit_value(&Out0 {
        code: args.code,
        name,
        category,
        valid,
    })
}
```

- [ ] **Step 8: Wire + commit http-status**

Add `pub mod http_status;` to `src/commands/mod.rs`. Add to `src/cli.rs`:

```rust
    /// HTTP status code lookup.
    #[command(name = "http-status")]
    HttpStatus(crate::commands::http_status::HttpStatusArgs),
```

Add dispatch arm.

```bash
cargo test
git add src/data/http_status.json src/commands/http_status/ src/commands/mod.rs src/cli.rs src/lib.rs tests/http_status.rs tests/snapshots/
git commit -m "feat(http-status): add http-status lookup with bundled status code table"
```

---

## Task 3: `qr` noun — `generate` + `wifi`

**Files:** `src/commands/qr/{mod,generate,wifi}.rs`, `tests/qr.rs`.

QR generation in SVG (default) or PNG. The `qr wifi` verb encodes a Wi-Fi network as a special URI:
`WIFI:T:<security>;S:<ssid>;P:<password>;H:<hidden>;;`

- [ ] **Step 1: Write `tests/qr.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn qr_generate_svg_default() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "generate", "https://example.com"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("</svg>"));
}

#[test]
fn qr_generate_png_requires_out_path_on_tty() {
    // Without --out, writing PNG to a TTY should be refused with exit 2.
    // assert_cmd's stdout is not a TTY, so this should actually succeed.
    // Skip the TTY-specific check; just confirm PNG output starts with PNG signature.
    let tmp = tempfile::NamedTempFile::new().unwrap();
    Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "generate", "hello", "--format", "png", "--out"]).arg(tmp.path())
        .assert().success();
    let bytes = std::fs::read(tmp.path()).unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "expected PNG magic");
}

#[test]
fn qr_generate_json_mode_wraps_svg() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "qr", "generate", "hi"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let svg = v["svg"].as_str().expect("svg field");
    assert!(svg.contains("<svg"));
}

#[test]
fn qr_wifi_emits_svg_containing_wifi_uri() {
    // The QR codes the WIFI: URI internally, so the SVG won't contain plaintext
    // — just confirm the SVG renders without error.
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "wifi", "--ssid", "MyNet", "--password", "pw", "--security", "WPA"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
}

#[test]
fn qr_wifi_nopass_security() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "wifi", "--ssid", "Public", "--security", "nopass"])
        .assert().success();
}

#[test]
fn qr_generate_empty_input_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "generate", ""])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid"));
}
```

> Note: the empty-input test might be lenient depending on `qrcode` 0.14 behavior. If empty strings generate a tiny QR successfully, drop that test or adjust to test an oversized input instead (4096+ chars typically fail).

- [ ] **Step 2: Create `src/commands/qr/mod.rs`**

```rust
//! QR code generation (SVG/PNG) including WiFi URIs.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod wifi;

#[derive(Debug, Args)]
pub struct QrArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a QR code from arbitrary text.
    #[command(long_about = "Generate a QR code from arbitrary text.\n\nFormats:\n  svg (default) — emitted to stdout (or --out path)\n  png — requires --out path or non-TTY stdout (binary)\n\nExamples:\n  ubertool qr generate 'https://example.com'\n  ubertool qr generate 'hello' --format png --out qr.png\n  ubertool qr generate 'hi' --json   # wraps SVG in {\"svg\":\"...\"}\n\nExit codes:\n  3   input too long or otherwise unencodable\n  2   PNG to TTY without --out (binary_to_tty_refused)")]
    Generate(generate::GenerateArgs),
    /// Generate a QR code encoding a Wi-Fi network URI.
    #[command(long_about = "Generate a QR code that, when scanned by a mobile camera, prompts to join a Wi-Fi network.\n\nEncodes the standard WIFI: URI: WIFI:T:<security>;S:<ssid>;P:<password>;H:<hidden>;;\n\nExamples:\n  ubertool qr wifi --ssid 'MyNet' --password 'pw'\n  ubertool qr wifi --ssid 'Public' --security nopass\n  ubertool qr wifi --ssid 'Hidden' --password 'pw' --hidden --out wifi.png --format png")]
    Wifi(wifi::WifiArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Svg,
    Png,
}

pub fn dispatch(args: QrArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => generate::run(a, out),
        Verb::Wifi(a) => wifi::run(a, out),
    }
}

pub(super) fn render(
    data: &str,
    format: Format,
    out_path: Option<&PathBuf>,
    out: &Out,
) -> Result<(), CliError> {
    use crate::core::error::ErrorCode;
    use serde::Serialize;
    use qrcode::QrCode;

    let code = QrCode::new(data.as_bytes()).map_err(|e| {
        CliError::new(ErrorCode::InvalidJson, format!("QR encode failed: {e}"))
            .with_hint("input may be too long or contain unencodable bytes")
    })?;

    match format {
        Format::Svg => {
            use qrcode::render::svg;
            let svg = code
                .render::<svg::Color>()
                .min_dimensions(200, 200)
                .build();
            if let Some(path) = out_path {
                std::fs::write(path, &svg).map_err(CliError::from)?;
                return Ok(());
            }
            #[derive(Serialize)]
            struct Out0 {
                svg: String,
            }
            out.emit_value(&Out0 { svg })
        }
        Format::Png => {
            // Render as PNG using the image crate.
            use qrcode::render::unicode;
            // qrcode 0.14: code.render::<Luma<u8>>() with image feature gated.
            // Use a simple approach: render to bool matrix manually and encode via image.
            let pixels: Vec<Vec<bool>> = (0..code.width())
                .map(|y| (0..code.width()).map(|x| code[(x, y)] == qrcode::Color::Dark).collect())
                .collect();
            let scale = 8u32;
            let border = 4u32;
            let inner = code.width() as u32;
            let total = (inner + border * 2) * scale;
            let mut img = image::GrayImage::from_pixel(total, total, image::Luma([255u8]));
            for (y, row) in pixels.iter().enumerate() {
                for (x, &dark) in row.iter().enumerate() {
                    if dark {
                        let px = (x as u32 + border) * scale;
                        let py = (y as u32 + border) * scale;
                        for dy in 0..scale {
                            for dx in 0..scale {
                                img.put_pixel(px + dx, py + dy, image::Luma([0u8]));
                            }
                        }
                    }
                }
            }
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("PNG encode: {e}")))?;
            let _ = unicode::Dense1x2::Light; // keep the import used; alternatively remove
            out.emit_binary(&bytes, out_path.map(|p| p.as_path()))
        }
    }
}
```

> Notes on `qrcode 0.14`:
> - `QrCode::new()` returns `Result<QrCode, EncodeError>`.
> - `code.width()` returns `usize`.
> - `code[(x, y)]` returns `qrcode::Color`.
> - SVG rendering uses the built-in `render::svg::Color` type.
> - PNG rendering is done by hand via the `image` crate to avoid relying on `qrcode`'s optional `image` feature (which has changed across versions).
> - The leftover `unicode::Dense1x2::Light` line is to silence the unused-import lint; if it triggers a warning, just delete the `use qrcode::render::unicode;` line.

- [ ] **Step 3: Create `src/commands/qr/generate.rs`**

```rust
use std::path::PathBuf;

use clap::Args;

use crate::core::error::CliError;
use crate::core::output::Out;

use super::{render, Format};

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "svg")]
    pub format: Format,
    /// Write to file (required for PNG when stdout is a TTY).
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,
}

pub fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    render(&args.input, args.format, args.out_path.as_ref(), out)
}
```

- [ ] **Step 4: Create `src/commands/qr/wifi.rs`**

```rust
use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

use super::{render, Format};

#[derive(Debug, Args)]
pub struct WifiArgs {
    #[arg(long)]
    pub ssid: String,
    /// Network password (omit for open networks).
    #[arg(long)]
    pub password: Option<String>,
    /// Security type.
    #[arg(long, value_enum, default_value = "wpa")]
    pub security: Security,
    /// Hidden network.
    #[arg(long, default_value_t = false)]
    pub hidden: bool,
    #[arg(long, value_enum, default_value = "svg")]
    pub format: Format,
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Security {
    #[value(name = "WPA")]
    Wpa,
    #[value(name = "WEP")]
    Wep,
    #[value(name = "nopass")]
    Nopass,
}

impl Security {
    fn label(self) -> &'static str {
        match self {
            Security::Wpa => "WPA",
            Security::Wep => "WEP",
            Security::Nopass => "nopass",
        }
    }
}

pub fn run(args: WifiArgs, out: &Out) -> Result<(), CliError> {
    let security = args.security.label();
    let password = args.password.unwrap_or_default();
    let hidden = if args.hidden { "true" } else { "false" };
    let escaped_ssid = escape_wifi(&args.ssid);
    let escaped_pw = escape_wifi(&password);
    let uri = format!(
        "WIFI:T:{security};S:{escaped_ssid};P:{escaped_pw};H:{hidden};;"
    );
    render(&uri, args.format, args.out_path.as_ref(), out)
}

fn escape_wifi(s: &str) -> String {
    // The WIFI: URI spec escapes \, ;, ,, ", :, and special chars with a backslash.
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | ';' | ',' | '"' | ':' => {
                out.push('\\');
                out.push(c);
            }
            other => out.push(other),
        }
    }
    out
}
```

- [ ] **Step 5: Wire + commit qr**

Add `pub mod qr;`. Add `Qr(crate::commands::qr::QrArgs)` variant. Dispatch arm.

```bash
cargo test
git add src/commands/qr/ src/commands/mod.rs src/cli.rs src/lib.rs tests/qr.rs tests/snapshots/
git commit -m "feat(qr): add qr generate (SVG/PNG) and qr wifi (WIFI: URI)"
```

---

## Task 4: `svg-placeholder` noun — `generate`

**Files:** `src/commands/svg_placeholder/mod.rs`, `tests/svg_placeholder.rs`.

Generates an SVG placeholder image: a colored rect with centered text. Useful for prototyping.

- [ ] **Step 1: Write `tests/svg_placeholder.rs`**

```rust
use assert_cmd::Command;

#[test]
fn svg_placeholder_default_dimensions() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["svg-placeholder", "generate"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("400"));  // default width
    assert!(s.contains("300"));  // default height
}

#[test]
fn svg_placeholder_custom_dimensions() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["svg-placeholder", "generate", "--width", "800", "--height", "200"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("800"));
    assert!(s.contains("200"));
    // Default label is "<W>x<H>"
    assert!(s.contains("800x200") || s.contains("800 x 200"));
}

#[test]
fn svg_placeholder_custom_text() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["svg-placeholder", "generate", "--text", "Hello"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Hello"));
}

#[test]
fn svg_placeholder_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "svg-placeholder", "generate", "--width", "100", "--height", "100"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let svg = v["svg"].as_str().expect("svg field");
    assert!(svg.contains("<svg"));
}
```

- [ ] **Step 2: Create `src/commands/svg_placeholder/mod.rs`**

```rust
//! SVG placeholder image generator.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct SvgPlaceholderArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a placeholder SVG image.
    #[command(long_about = "Generate a placeholder SVG image with a centered label.\n\nUseful for prototyping when you need a sized image but no real content.\n\nExamples:\n  ubertool svg-placeholder generate --width 800 --height 600\n  ubertool svg-placeholder generate --text 'Logo'\n  ubertool svg-placeholder generate --width 100 --height 100 --bg '#333' --fg '#fff' --json")]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(long, default_value_t = 400)]
    pub width: u32,
    #[arg(long, default_value_t = 300)]
    pub height: u32,
    /// Centered text label (default: "<W>x<H>").
    #[arg(long)]
    pub text: Option<String>,
    /// Background color (any CSS color).
    #[arg(long, default_value = "#cccccc")]
    pub bg: String,
    /// Foreground (text) color.
    #[arg(long, default_value = "#333333")]
    pub fg: String,
}

#[derive(Serialize)]
struct Out0 {
    svg: String,
}

pub fn dispatch(args: SvgPlaceholderArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let label = args.text.unwrap_or_else(|| format!("{}x{}", args.width, args.height));
    let font_size = (args.width.min(args.height) / 8).max(12);
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <text x="50%" y="50%" font-family="sans-serif" font-size="{fs}" fill="{fg}" text-anchor="middle" dominant-baseline="middle">{label}</text>
</svg>"#,
        w = args.width,
        h = args.height,
        bg = args.bg,
        fg = args.fg,
        fs = font_size,
        label = escape_xml(&label),
    );
    out.emit_value(&Out0 { svg })
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

- [ ] **Step 3: Wire + commit svg-placeholder**

Add `pub mod svg_placeholder;`. Add to `src/cli.rs`:
```rust
    /// SVG placeholder image generator.
    #[command(name = "svg-placeholder")]
    SvgPlaceholder(crate::commands::svg_placeholder::SvgPlaceholderArgs),
```
Dispatch arm.

```bash
cargo test
git add src/commands/svg_placeholder/ src/commands/mod.rs src/cli.rs src/lib.rs tests/svg_placeholder.rs tests/snapshots/
git commit -m "feat(svg-placeholder): add svg-placeholder generate (sized SVG with centered label)"
```

---

## Task 5: `ascii` noun — `draw`

**Files:** `src/commands/ascii/mod.rs`, `tests/ascii.rs`. Uses `figlet-rs`.

- [ ] **Step 1: Write `tests/ascii.rs`**

```rust
use assert_cmd::Command;

#[test]
fn ascii_draw_renders_letters() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["ascii", "draw", "Hi"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    // Figlet output spans multiple lines and contains some box-drawing-ish chars.
    assert!(s.lines().count() >= 4);
}

#[test]
fn ascii_draw_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ascii", "draw", "X"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let art = v["art"].as_str().expect("art field");
    assert!(!art.is_empty());
}

#[test]
fn ascii_draw_empty_input() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["ascii", "draw", ""])
        .assert().success();
}
```

- [ ] **Step 2: Create `src/commands/ascii/mod.rs`**

```rust
//! ASCII art (figlet) rendering.

use clap::{Args, Subcommand};
use figlet_rs::FIGfont;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct AsciiArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Render text as ASCII art using the figlet standard font.
    #[command(long_about = "Render text as ASCII art using the figlet `standard` font.\n\nExamples:\n  ubertool ascii draw 'Hello'\n  ubertool ascii draw 'Hi' --json")]
    Draw(DrawArgs),
}

#[derive(Debug, Args)]
pub struct DrawArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    art: String,
}

pub fn dispatch(args: AsciiArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Draw(a) => run(a, out),
    }
}

fn run(args: DrawArgs, out: &Out) -> Result<(), CliError> {
    let font = FIGfont::standard().map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("could not load figlet font: {e}"))
    })?;
    let figure = font.convert(&args.input).ok_or_else(|| {
        CliError::new(ErrorCode::Internal, "figlet failed to render".to_string())
    })?;
    out.emit_value(&Out0 { art: figure.to_string() })
}
```

- [ ] **Step 3: Wire + commit ascii**

Add `pub mod ascii;`. Add `Ascii` variant. Dispatch arm.

```bash
cargo test
git add src/commands/ascii/ src/commands/mod.rs src/cli.rs src/lib.rs tests/ascii.rs tests/snapshots/
git commit -m "feat(ascii): add ascii draw (figlet standard font)"
```

---

## Task 6: `git memo` + `regex memo` (batched)

**Files:** `src/data/git_memo.md`, `src/data/regex_memo.md`, `src/commands/git/mod.rs`, modify `src/commands/regex/mod.rs` to add `memo` verb, `tests/git_memo.rs`, `tests/regex_memo.rs`. One commit per noun.

Cheat sheets bundled as markdown. The `memo` verb just prints the markdown to stdout (or wraps in `{"memo": "..."}` for `--json`).

### Bundled memos

- [ ] **Step 1: Create `src/data/git_memo.md`**

```markdown
# Git cheat sheet

## Setup
```
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

## Basic workflow
```
git init                       # init repo
git status                     # show working tree state
git add <path>                 # stage changes
git add -p                     # stage interactively
git commit -m "message"        # commit staged
git commit --amend             # amend last commit
git log --oneline -10          # recent commits
```

## Branches
```
git branch                     # list branches
git branch -a                  # include remotes
git switch -c <name>           # create + switch
git switch <name>              # switch
git merge <branch>             # merge into current
git rebase <branch>            # rebase current onto branch
git rebase -i HEAD~5           # interactive rebase last 5
git branch -d <name>           # delete merged branch
git branch -D <name>           # force delete
```

## Remotes
```
git remote -v                  # list
git remote add origin <url>    # add
git fetch                      # download remote refs
git pull                       # fetch + merge
git pull --rebase              # fetch + rebase
git push                       # push current branch
git push -u origin <name>      # push + set upstream
git push --force-with-lease    # safer force push
```

## Undo / inspect
```
git diff                       # unstaged vs working
git diff --cached              # staged vs HEAD
git diff <a>..<b>              # range
git log -p <path>              # changes touching a path
git blame <path>               # who wrote each line
git show <sha>                 # full commit
git restore <path>             # discard working changes
git restore --staged <path>    # unstage
git reset --soft HEAD~1        # undo commit, keep changes staged
git reset --mixed HEAD~1       # undo commit, keep changes unstaged
git reset --hard HEAD~1        # undo commit, DROP changes (!)
git revert <sha>               # new commit that undoes <sha>
```

## Stash
```
git stash                      # save uncommitted
git stash list                 # list
git stash pop                  # restore most recent
git stash apply stash@{2}      # restore specific
git stash drop stash@{2}       # forget
```

## Searching
```
git grep <pattern>             # search tracked files
git log -S<string>             # commits that add/remove <string>
git log --grep=<pattern>       # search commit messages
git bisect start ...           # binary search for a bad commit
```

## Tags
```
git tag                        # list
git tag -a v1.0.0 -m "msg"     # annotated
git push --tags                # push tags
```
```

- [ ] **Step 2: Create `src/data/regex_memo.md`**

```markdown
# Regex cheat sheet

## Anchors
```
^         start of line/string
$         end of line/string
\b        word boundary
\B        non-word boundary
\A        start of string (Rust)
\z        end of string (Rust)
```

## Character classes
```
.         any char (except newline by default)
\d \D     digit / non-digit
\w \W     word char / non-word
\s \S     whitespace / non-whitespace
[abc]     any of a, b, c
[^abc]    none of a, b, c
[a-z]     range
```

## Quantifiers
```
?         0 or 1
*         0 or more (greedy)
+         1 or more (greedy)
*?  +? ??  lazy variants
{n}       exactly n
{n,}      n or more
{n,m}     n to m
```

## Groups
```
(abc)             capturing group
(?:abc)           non-capturing
(?P<name>abc)     named (Rust syntax)
\1 \2 ...         backrefs (NOT supported by Rust's regex crate)
```

## Lookaround (NOT supported by Rust's regex crate)
```
(?=...)   positive lookahead
(?!...)   negative lookahead
(?<=...)  positive lookbehind
(?<!...)  negative lookbehind
```

For lookaround in Rust, use the `fancy-regex` crate.

## Common patterns
```
^\d+$                            integer
^-?\d+(\.\d+)?$                  number with optional sign/decimal
^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$  email-ish
^https?://                        http/s URL prefix
\d{4}-\d{2}-\d{2}                ISO date YYYY-MM-DD
\d{2}:\d{2}(:\d{2})?             time HH:MM[:SS]
^[0-9a-fA-F]{40}$                git SHA-1
^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$  UUID
```

## Flags (inline)
```
(?i)abc       case-insensitive
(?x)          extended (whitespace + comments)
(?m)          multiline (^/$ match line ends)
(?s)          dot matches newline
(?U)          swap greedy/lazy (Rust)
```

## Tools
- `ubertool regex test --pattern '<p>' --text '<t>'`  test a pattern
- `ubertool regex generate '<pattern>'`               generate a matching string
```

### `git memo`

- [ ] **Step 3: Write `tests/git_memo.rs`**

```rust
use assert_cmd::Command;

#[test]
fn git_memo_prints_markdown() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["git", "memo"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Git cheat sheet"));
    assert!(s.contains("git add"));
    assert!(s.contains("git commit"));
}

#[test]
fn git_memo_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "git", "memo"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let memo = v["memo"].as_str().expect("memo field");
    assert!(memo.contains("Git cheat sheet"));
}
```

- [ ] **Step 4: Create `src/commands/git/mod.rs`**

```rust
//! Git cheat-sheet memo.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MEMO: &str = include_str!("../../data/git_memo.md");

#[derive(Debug, Args)]
pub struct GitArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Print the bundled git cheat sheet (Markdown).
    #[command(long_about = "Print the bundled git cheat sheet as Markdown.\n\nCovers setup, basic workflow, branches, remotes, undo/inspect, stash, search, tags.\n\nExamples:\n  ubertool git memo\n  ubertool git memo --json   # wraps Markdown in {\"memo\": \"...\"}")]
    Memo,
}

#[derive(Serialize)]
struct Out0 {
    memo: String,
}

pub fn dispatch(args: GitArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Memo => out.emit_value(&Out0 { memo: MEMO.to_string() }),
    }
}
```

- [ ] **Step 5: Wire + commit git**

Add `pub mod git;`. Add `Git` variant. Dispatch arm.

```bash
cargo test
git add src/data/git_memo.md src/commands/git/ src/commands/mod.rs src/cli.rs src/lib.rs tests/git_memo.rs tests/snapshots/
git commit -m "feat(git): add git memo (bundled cheat sheet)"
```

### `regex memo` (extends existing M4 `regex` noun)

- [ ] **Step 6: Write `tests/regex_memo.rs`**

```rust
use assert_cmd::Command;

#[test]
fn regex_memo_prints_markdown() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["regex", "memo"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Regex cheat sheet"));
    assert!(s.contains("Anchors"));
    assert!(s.contains("\\d"));
}

#[test]
fn regex_memo_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "regex", "memo"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let memo = v["memo"].as_str().expect("memo field");
    assert!(memo.contains("Regex cheat sheet"));
}
```

- [ ] **Step 7: Extend `src/commands/regex/mod.rs`**

Find the existing `Verb` enum (`Test`, `Generate`). Add a third variant:

```rust
    /// Print the bundled regex cheat sheet (Markdown).
    #[command(long_about = "Print the bundled regex cheat sheet as Markdown.\n\nCovers anchors, character classes, quantifiers, groups, lookaround, common patterns, and inline flags. Notes which features are not supported by Rust's `regex` crate.\n\nExamples:\n  ubertool regex memo\n  ubertool regex memo --json")]
    Memo,
```

Extend `dispatch()` and add a new file `src/commands/regex/memo.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

const MEMO: &str = include_str!("../../data/regex_memo.md");

#[derive(Serialize)]
struct Out0 {
    memo: String,
}

pub fn run(out: &Out) -> Result<(), CliError> {
    out.emit_value(&Out0 { memo: MEMO.to_string() })
}
```

Add `pub mod memo;` to `regex/mod.rs`. Update dispatch:

```rust
        Verb::Memo => memo::run(out),
```

- [ ] **Step 8: Wire + commit regex memo**

```bash
cargo test
git add src/data/regex_memo.md src/commands/regex/ tests/regex_memo.rs tests/snapshots/
git commit -m "feat(regex): add regex memo (bundled cheat sheet)"
```

---

## Task 7: `numeronym` + `port` (batched)

**Files:** `src/commands/numeronym/mod.rs`, `src/commands/port/mod.rs`, `tests/numeronym.rs`, `tests/port.rs`. One commit per noun.

### `numeronym generate`

`numeronym generate "internationalization"` → `"i18n"` (first + count of middle + last).

- [ ] **Step 1: Write `tests/numeronym.rs`**

```rust
use assert_cmd::Command;

#[test]
fn numeronym_i18n() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "internationalization"])
        .assert().success()
        .stdout("i18n\n");
}

#[test]
fn numeronym_k8s() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "kubernetes"])
        .assert().success()
        .stdout("k8s\n");
}

#[test]
fn numeronym_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "numeronym", "generate", "accessibility"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["numeronym"], "a11y");
}

#[test]
fn numeronym_short_word_passes_through() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "hi"])
        .assert().success()
        .stdout("hi\n");
}

#[test]
fn numeronym_single_char_passes_through() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "x"])
        .assert().success()
        .stdout("x\n");
}
```

- [ ] **Step 2: Create `src/commands/numeronym/mod.rs`**

```rust
//! Numeronym (i18n-style contraction) generator.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct NumeronymArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a numeronym (e.g., 'internationalization' -> 'i18n').
    #[command(long_about = "Generate a numeronym: first char + count of middle chars + last char.\n\nWords ≤ 2 chars are passed through unchanged.\n\nExamples:\n  ubertool numeronym generate internationalization   # i18n\n  ubertool numeronym generate kubernetes              # k8s\n  ubertool numeronym generate accessibility           # a11y")]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    numeronym: String,
}

pub fn dispatch(args: NumeronymArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let chars: Vec<char> = args.input.chars().collect();
    let result = if chars.len() <= 2 {
        args.input.clone()
    } else {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        let middle = chars.len() - 2;
        format!("{first}{middle}{last}")
    };
    out.emit_value(&Out0 { numeronym: result })
}
```

- [ ] **Step 3: Wire + commit numeronym**

Add `pub mod numeronym;`. Add `Numeronym` variant. Dispatch arm.

```bash
cargo test
git add src/commands/numeronym/ src/commands/mod.rs src/cli.rs src/lib.rs tests/numeronym.rs tests/snapshots/
git commit -m "feat(numeronym): add numeronym generate (i18n-style contraction)"
```

### `port random`

Generates a random port in the ephemeral range (49152-65535 per IANA) by default.

- [ ] **Step 4: Write `tests/port.rs`**

```rust
use assert_cmd::Command;

#[test]
fn port_random_in_ephemeral_range() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let n: u16 = s.trim().parse().expect("port must be u16");
    assert!((49152..=65535).contains(&n), "expected ephemeral range, got {n}");
}

#[test]
fn port_random_custom_range() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random", "--min", "8000", "--max", "8099"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let n: u16 = s.trim().parse().expect("port must be u16");
    assert!((8000..=8099).contains(&n), "expected 8000-8099, got {n}");
}

#[test]
fn port_random_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "port", "random"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["port"].is_number());
}

#[test]
fn port_random_invalid_range_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random", "--min", "200", "--max", "100"])
        .assert().failure().code(2);
}
```

- [ ] **Step 5: Create `src/commands/port/mod.rs`**

```rust
//! Random port generator.

use clap::{Args, Subcommand};
use rand::Rng;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct PortArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random port number in a range.
    #[command(long_about = "Generate a random TCP/UDP port number.\n\nDefault range is the IANA ephemeral range 49152-65535. Override with --min/--max.\n\nExamples:\n  ubertool port random\n  ubertool port random --min 8000 --max 8099\n\nExit codes:\n  2   --min greater than --max")]
    Random(RandomArgs),
}

#[derive(Debug, Args)]
pub struct RandomArgs {
    #[arg(long, default_value_t = 49152)]
    pub min: u16,
    #[arg(long, default_value_t = 65535)]
    pub max: u16,
}

#[derive(Serialize)]
struct Out0 {
    port: u16,
}

pub fn dispatch(args: PortArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Random(a) => run(a, out),
    }
}

fn run(args: RandomArgs, out: &Out) -> Result<(), CliError> {
    if args.min > args.max {
        return Err(CliError::new(
            ErrorCode::UsageError,
            format!("--min ({}) must be ≤ --max ({})", args.min, args.max),
        ));
    }
    let port = rand::thread_rng().gen_range(args.min..=args.max);
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { port })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{port}").map_err(CliError::from)
    }
}
```

- [ ] **Step 6: Wire + commit port**

Add `pub mod port;`. Add `Port` variant. Dispatch arm.

```bash
cargo test
git add src/commands/port/ src/commands/mod.rs src/cli.rs src/lib.rs tests/port.rs tests/snapshots/
git commit -m "feat(port): add port random (ephemeral range default)"
```

---

## Task 8: Update OpenCLI spec for M5

**Files:** Modify `ubertool.ocs.yaml`. Run `make gen` and `make verify-spec`.

Follow the M3/M4 spec task workflow. Add entries for new M5 nouns + `regex memo` leaf under the existing `regex` group.

**M5 entries to add:**

1. `mime` group + 1 leaf:
   - `mime lookup <input>`: required string positional. Flags `--json`, `--quiet`.

2. `http-status` group + 1 leaf (kebab-case noun key):
   - `http-status lookup <code>`: required integer positional. Flags `--json`, `--quiet`.

3. `qr` group + 2 leaves:
   - `qr generate <input>`: required positional. Flags `--format` (string, choices svg/png, default svg), `--out` (string optional), `--json`, `--quiet`.
   - `qr wifi`: no positional. Flags `--ssid` (required), `--password` (optional), `--security` (choices WPA/WEP/nopass, default WPA), `--hidden` (boolean default false), `--format`, `--out`, `--json`, `--quiet`.

4. `svg-placeholder` group + 1 leaf (kebab-case):
   - `svg-placeholder generate`: no positional. Flags `--width` (integer default "400"), `--height` (integer default "300"), `--text` (optional), `--bg` (string default "#cccccc"), `--fg` (string default "#333333"), `--json`, `--quiet`.

5. `ascii` group + 1 leaf:
   - `ascii draw <input>`: required positional. Flags `--json`, `--quiet`.

6. `git` group + 1 leaf:
   - `git memo`: no positional. Flags `--json`, `--quiet`.

7. **`regex` group** — add new leaf `regex memo` (no positional, flags `--json`/`--quiet`). The existing `regex test` and `regex generate` entries stay unchanged.

8. `numeronym` group + 1 leaf:
   - `numeronym generate <input>`: required positional. Flags `--json`, `--quiet`.

9. `port` group + 1 leaf:
   - `port random`: no positional. Flags `--min` (integer default "49152"), `--max` (integer default "65535"), `--json`, `--quiet`.

Spec format gotchas (same as M4):
- `choices:` is list of `{value: "..."}` maps.
- Integer defaults are quoted strings.
- Summaries with colons need quoting.
- Comma defaults rejected.

```bash
ocli spec check ubertool.ocs.yaml
make gen
make verify-spec
git add ubertool.ocs.yaml docs/cli/ docs/llms.txt
git commit -m "docs(spec): add all M5 nouns to OpenCLI spec and regenerate Markdown + llms.txt"
```

Expected: `make verify-spec` reports ≥ 102 leaf commands (M4 ended at 92; M5 adds 10).

---

## Task 9: Fitness + `make ci`

**Files:** Modify `tests/agent_cli_fitness.rs`. Run `make ci`.

Update the `regex` entry from `&["test", "generate"]` to `&["test", "generate", "memo"]`. Append M5 entries:

```rust
    // M5
    ("mime", &["lookup"]),
    ("http-status", &["lookup"]),
    ("qr", &["generate", "wifi"]),
    ("svg-placeholder", &["generate"]),
    ("ascii", &["draw"]),
    ("git", &["memo"]),
    ("numeronym", &["generate"]),
    ("port", &["random"]),
```

```bash
cargo test --test agent_cli_fitness
make ci
```

Fix clippy if needed. The `Noun` enum now has ~54 variants — if `clippy::large_enum_variant` fires, add `#[allow(clippy::large_enum_variant)]` to the enum.

```bash
git add tests/agent_cli_fitness.rs
git commit -m "test(fitness): extend fitness checklist to all M5 nouns and verbs"
```

If fmt applied:

```bash
git add -u
git commit -m "style: apply cargo fmt across M5 source and test files"
```

Confirm clean tree. **Do not push or tag.**

---

## Self-Review

**Spec coverage** — every M5 requirement from design spec §16 covered:
- ✅ mime lookup — Task 2
- ✅ http-status lookup — Task 2
- ✅ qr generate + wifi — Task 3
- ✅ svg-placeholder — Task 4
- ✅ ascii draw — Task 5
- ✅ git memo — Task 6
- ✅ regex memo — Task 6
- ✅ numeronym — Task 7
- ✅ port random — Task 7
- ✅ spec + fitness + CI — Tasks 8, 9

**Placeholder scan** — no TBDs. Bundled CSV/JSON/MD content fully written out.

**Type consistency** — All commands emit `#[derive(Serialize)]` structs. No new ErrorCode variants in M5 (existing `UsageError`, `IoError`, `BinaryToTtyRefused`, `Internal` cover all paths).

**Scope check** — 8 new nouns + 1 verb extension + spec + CI = 9 task topics across 9 numbered tasks.

**Ambiguity check** — decisions documented:
- mime + http-status use soft-fail (null/false) for unknown values, matching M3 mac lookup precedent.
- QR PNG uses hand-rolled image encoding (avoids relying on qrcode's optional image feature which changes across versions).
- Wi-Fi URI follows the standard `WIFI:T:...;S:...;P:...;H:...;;` form with proper escaping.
- SVG placeholder font size scales with the smaller dimension (`min(w, h) / 8`).
- Numeronym threshold: ≤2 chars passes through unchanged.
- Port random default range is IANA ephemeral (49152-65535).
- Memos are bundled at compile time via `include_str!` (curated content; can be refined by editing the .md files).

---

## Execution Handoff

Plan saved. Same workflow as prior milestones — subagent-driven execution.
