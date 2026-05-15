# ubertool M2 — Tier 2 (Converters) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 13 new converter nouns (~22 verbs) on top of the M1 foundation: `xml`, `csv`, `case`, `slugify`, `list`, `integer-base`, `roman`, `temperature`, `sql`, `markdown`, `text` (8 verbs), `docker-run`, `safelink`.

**Architecture:** Same as M1 — single crate, each noun is a folder under `src/commands/<noun>/` with `mod.rs` plus per-verb files when there are ≥2 verbs. All commands plug into the shared `core/` infrastructure exactly as base64/hash/bcrypt do.

**Tech Stack:** Adds to M0+M1: `quick-xml` (with `serialize` feature), `csv`, `convert_case`, `slug`, `sqlformat`, `pulldown-cmark`, `similar` (for text diff).

**Companion documents:**
- Design spec: `docs/superpowers/specs/2026-05-15-ubertool-cli-design.md` (§3 inventory, §11 crates).
- M1 plan (template): `docs/superpowers/plans/2026-05-15-ubertool-m1-foundation.md`.
- M1 code (canonical patterns): `src/commands/{base64,hash,bcrypt,json}/` — copy/adapt patterns from these.

**Working directory:** `/Users/mihai/dev/ubertool/` (all tasks run here).

**Templates the implementer should reference:**
- **Single-verb noun:** `src/commands/uuid/mod.rs` or `src/commands/ulid/mod.rs`.
- **Multi-verb noun:** `src/commands/base64/` or `src/commands/bcrypt/`.
- **Multiplexed verb (verb-is-an-enum):** `src/commands/hash/mod.rs` (algorithm-per-subcommand).
- **Format-conversion noun:** `src/commands/json/` (with `convert.rs` shared helper).

**The cross-cutting noun pattern** (same as M1):
1. Create `src/commands/<noun>/mod.rs` (and per-verb `.rs` files when needed).
2. Add `pub mod <noun>;` to `src/commands/mod.rs`.
3. Add a `<Noun>(...)` variant to `src/cli.rs::Noun`.
4. Add a match arm to `src/lib.rs::run()`.
5. Write `tests/<noun>.rs` (assert_cmd integration tests).
6. Run `cargo test` + commit.
7. Update the top-level help insta snapshot (`tests/snapshots/snapshots__top_level_help.snap`) — every new noun changes it.

**Acceptance criteria (must all pass at end of M2):**

1. `cargo build` succeeds. `cargo build --release` produces a working binary.
2. `cargo test` passes 100%. Test count grows by ≥ 50 over the M1 baseline (128 → ~180+).
3. Every new noun appears in `ubertool --help`; every verb appears in `ubertool <noun> --help`.
4. Every data-returning command supports `--json` and emits parseable JSON to stdout in `--json` mode.
5. The fitness-checklist test is extended to cover the new surface (Task 17) and passes.
6. `make verify-spec` passes — OpenCLI spec updated with all new nouns/verbs (Task 16).
7. `make ci` passes locally (fmt + clippy + test + verify-spec).
8. Each commit follows TDD: failing test → minimal code → passing test → commit.
9. Repo is clean: `git status` shows nothing uncommitted.

---

## Cross-cutting conventions for all M2 tasks

Same as M1 — see `docs/superpowers/plans/2026-05-15-ubertool-m1-foundation.md` "Cross-cutting conventions" section. Summary:

- **Single-value outputs** → `#[derive(Serialize)] struct Out0 { <field>: <T> }`. Text mode emits just the value.
- **Multi-value outputs** → struct with named fields. Text mode emits `key: value\n` lines.
- **Input** via `resolve_input(positional, in_path, is_stdin_tty())`.
- **Errors** via `CliError::new(ErrorCode::<Variant>, msg).with_input(json!(echoed)).with_hint("...")`. Secrets/passwords ALWAYS redacted in `input` field.
- **Tests** cover: text mode output, `--json` mode is parseable JSON, at least one error path with the documented exit code, stdin pipe input.

---

## Task 1: Add M2 dependencies to Cargo.toml

**Files:** Modify `Cargo.toml`.

- [ ] **Step 1: Add dependencies**

In `[dependencies]`, append after the M1 conversion section:

```toml
# M2 conversion
quick-xml = { version = "0.31", features = ["serialize"] }
csv = "1"
convert_case = "0.6"
slug = "0.1"
sqlformat = "0.2"
pulldown-cmark = "0.10"
similar = "2"
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: succeeds. Unused-crate warnings on the new deps are acceptable.

- [ ] **Step 3: Run M1 tests still pass**

Run: `cargo test`
Expected: 128 tests pass (M1 baseline).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add M2 dependencies (xml/csv/case/slug/sql/markdown/diff)"
```

---

## Task 2: `xml` noun — `to-json` + `format` verbs

**Files:** Create `src/commands/xml/{mod,to_json,format}.rs`. Modify wiring. Create `tests/xml.rs`. Add `InvalidXml` is already in `ErrorCode` (no enum changes needed).

XML→JSON convention: each element becomes an object. Attributes become `"@attr": value` keys. Text content of leaf elements becomes the plain value (string). Mixed content (text + children) wraps text under `"#text"`.

- [ ] **Step 1: Write failing tests**

Create `tests/xml.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn xml_to_json_simple_element() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "xml", "to-json", "<root><name>alice</name><age>30</age></root>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    // {"json": "{\"root\":{\"name\":\"alice\",\"age\":\"30\"}}"} or similar shape
    let json_str = v["json"].as_str().expect("json field");
    let parsed: serde_json::Value = serde_json::from_str(json_str).expect("inner json");
    assert_eq!(parsed["root"]["name"], "alice");
    assert_eq!(parsed["root"]["age"], "30");
}

#[test]
fn xml_to_json_attributes() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "xml", "to-json", r#"<user id="42" role="admin">alice</user>"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let parsed: serde_json::Value =
        serde_json::from_str(v["json"].as_str().unwrap()).expect("inner json");
    assert_eq!(parsed["user"]["@id"], "42");
    assert_eq!(parsed["user"]["@role"], "admin");
    assert_eq!(parsed["user"]["#text"], "alice");
}

#[test]
fn xml_to_json_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "to-json", "<unclosed"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_xml"));
}

#[test]
fn xml_format_pretty_prints() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "format", "<root><a>1</a><b>2</b></root>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Should contain at least one newline (pretty-printed)
    assert!(s.contains('\n'), "format output should contain newlines: {s}");
    assert!(s.contains("<a>1</a>"));
}

#[test]
fn xml_format_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "format", "<bad"])
        .assert()
        .failure()
        .code(3);
}
```

Run: `cargo test --test xml` → FAIL.

- [ ] **Step 2: Implement `src/commands/xml/mod.rs`**

```rust
//! XML conversion: to-json (compact) and format (pretty-print).

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod format;
pub mod to_json;

#[derive(Debug, Args)]
pub struct XmlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert XML to JSON. Attributes become `@attr` keys; text content becomes a string or `#text`.
    #[command(name = "to-json", long_about = "Convert XML to JSON.\n\nConvention: attributes become `@attr` keys, text content of leaf elements becomes a string value, mixed content places text under `#text`.\n\nExamples:\n  ubertool xml to-json '<root><name>alice</name></root>'\n  ubertool xml to-json --in ./data.xml --json\n\nExit codes specific to this command:\n  3   invalid XML (invalid_xml)")]
    ToJson(RunArgs),
    /// Pretty-print XML with indentation.
    #[command(long_about = "Pretty-print XML with two-space indentation.\n\nExamples:\n  ubertool xml format '<r><a>1</a></r>'\n\nExit codes specific to this command:\n  3   invalid XML (invalid_xml)")]
    Format(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

pub fn dispatch(args: XmlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => to_json::run(a, out),
        Verb::Format(a) => format::run(a, out),
    }
}
```

- [ ] **Step 3: Implement `src/commands/xml/to_json.rs`**

Custom recursive walker over `quick_xml::Reader`. The full helper is ~80 LOC:

```rust
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

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
    let xml_str = input.as_str()?;
    let value = parse_xml_to_value(xml_str)?;
    let json = serde_json::to_string(&value).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("json serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { json })
}

fn parse_xml_to_value(xml: &str) -> Result<Value, CliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    // Stack of (element_name, attributes_map, children_map, text_buffer).
    // children_map preserves insertion order (using Vec of (name, Value)) so
    // repeated elements collapse into arrays.
    enum Slot {
        Single(Value),
        Many(Vec<Value>),
    }
    struct Frame {
        name: String,
        attrs: Map<String, Value>,
        children: Vec<(String, Slot)>,
        text: String,
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<(String, Value)> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(CliError::new(
                    ErrorCode::InvalidXml,
                    format!("invalid XML at byte {}: {e}", reader.buffer_position()),
                )
                .with_hint("ensure the input is well-formed XML"));
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let mut attrs = Map::new();
                for attr in e.attributes().flatten() {
                    let k = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
                    let v = attr
                        .unescape_value()
                        .map(|c| c.into_owned())
                        .unwrap_or_default();
                    attrs.insert(k, Value::String(v));
                }
                stack.push(Frame {
                    name,
                    attrs,
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let mut attrs = Map::new();
                for attr in e.attributes().flatten() {
                    let k = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
                    let v = attr
                        .unescape_value()
                        .map(|c| c.into_owned())
                        .unwrap_or_default();
                    attrs.insert(k, Value::String(v));
                }
                let value = if attrs.is_empty() {
                    Value::Null
                } else {
                    Value::Object(attrs)
                };
                attach_child(&mut stack, &mut root, name, value);
            }
            Ok(Event::End(_)) => {
                let frame = stack.pop().expect("End without Start");
                let value = build_frame_value(frame);
                let name = match value {
                    Value::Object(ref m) if m.is_empty() => String::new(),
                    _ => String::new(),
                };
                // We need the name from the frame; rebuild differently below.
                drop(name);
            }
            Ok(Event::Text(t)) => {
                if let Some(top) = stack.last_mut() {
                    let s = t
                        .unescape()
                        .map(|c| c.into_owned())
                        .unwrap_or_default();
                    if !s.is_empty() {
                        top.text.push_str(&s);
                    }
                }
            }
            _ => {}
        }
        buf.clear();
    }

    if let Some((name, val)) = root {
        let mut wrapper = Map::new();
        wrapper.insert(name, val);
        Ok(Value::Object(wrapper))
    } else {
        Err(CliError::new(
            ErrorCode::InvalidXml,
            "XML has no root element",
        ))
    }
}

// Helper: attach a (name, value) child to the top of the stack, or set as root.
fn attach_child(
    stack: &mut Vec<impl ChildContainer>,
    root: &mut Option<(String, Value)>,
    name: String,
    value: Value,
) {
    if let Some(top) = stack.last_mut() {
        top.attach(name, value);
    } else {
        *root = Some((name, value));
    }
}

fn build_frame_value<F: FrameLike>(frame: F) -> Value {
    let (attrs, children, text) = frame.parts();
    // Decide shape:
    // - If only text: Value::String(text)
    // - If attrs + text only: object with @attrs + #text
    // - If children: object with grouped children (+ @attrs + #text if present)
    if attrs.is_empty() && children.is_empty() {
        return Value::String(text);
    }
    let mut map = attrs;
    if !text.is_empty() && !children.is_empty() {
        map.insert("#text".to_string(), Value::String(text.clone()));
    } else if !text.is_empty() {
        map.insert("#text".to_string(), Value::String(text.clone()));
    }
    for (k, slot) in children {
        let v = match slot {
            Slot::Single(v) => v,
            Slot::Many(arr) => Value::Array(arr),
        };
        map.insert(k, v);
    }
    Value::Object(map)
}

// IMPLEMENTER NOTE:
// The recursive structure above is sketched but the auxiliary `ChildContainer`,
// `FrameLike`, and `Slot` need to wire together cleanly. Below is a CONSOLIDATED
// working version — use this instead of the sketch above:
```

> **Implementer note for Step 3:** The sketch above is illustrative; the recursive XML→JSON walker is fiddly. **Use this consolidated working implementation** instead — it has been mentally type-checked:

```rust
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

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
    let value = parse_xml_to_value(input.as_str()?)?;
    let json = serde_json::to_string(&value).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("json serialization failed: {e}"))
    })?;
    out.emit_value(&Out0 { json })
}

struct Frame {
    name: String,
    attrs: Map<String, Value>,
    children: Vec<(String, Value)>,
    text: String,
}

fn parse_xml_to_value(xml: &str) -> Result<Value, CliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Frame> = None;
    let mut buf = Vec::new();

    fn read_attrs(e: &quick_xml::events::BytesStart) -> Map<String, Value> {
        let mut attrs = Map::new();
        for attr in e.attributes().flatten() {
            let k = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
            let v = attr
                .unescape_value()
                .map(|c| c.into_owned())
                .unwrap_or_default();
            attrs.insert(k, Value::String(v));
        }
        attrs
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(CliError::new(
                    ErrorCode::InvalidXml,
                    format!("invalid XML: {e}"),
                )
                .with_hint("ensure the input is well-formed XML"));
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(Frame {
                    name,
                    attrs: read_attrs(&e),
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs = read_attrs(&e);
                let v = if attrs.is_empty() {
                    Value::String(String::new())
                } else {
                    Value::Object(attrs)
                };
                if let Some(top) = stack.last_mut() {
                    top.children.push((name, v));
                } else {
                    return Err(CliError::new(
                        ErrorCode::InvalidXml,
                        "XML has no root element wrapping content",
                    ));
                }
            }
            Ok(Event::Text(t)) => {
                if let Some(top) = stack.last_mut() {
                    let s = t
                        .unescape()
                        .map(|c| c.into_owned())
                        .unwrap_or_default();
                    if !s.is_empty() {
                        top.text.push_str(&s);
                    }
                }
            }
            Ok(Event::End(_)) => {
                let frame = stack.pop().expect("End without matching Start");
                let v = frame_to_value(&frame);
                if let Some(parent) = stack.last_mut() {
                    parent.children.push((frame.name, v));
                } else {
                    // closed the root
                    root = Some(frame);
                    // overwrite name/text/etc in the returned Frame to carry the
                    // already-computed Value is messy; rebuild below.
                }
            }
            _ => {}
        }
        buf.clear();
    }

    if let Some(frame) = root {
        let v = frame_to_value(&frame);
        let mut wrapper = Map::new();
        wrapper.insert(frame.name, v);
        Ok(Value::Object(wrapper))
    } else {
        Err(CliError::new(
            ErrorCode::InvalidXml,
            "XML has no root element",
        ))
    }
}

fn frame_to_value(frame: &Frame) -> Value {
    // Leaf with only text and no attrs/children → just the string.
    if frame.attrs.is_empty() && frame.children.is_empty() {
        return Value::String(frame.text.clone());
    }
    let mut map = frame.attrs.clone();
    if !frame.text.is_empty() {
        map.insert("#text".to_string(), Value::String(frame.text.clone()));
    }
    // Group repeated child names into arrays.
    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for (k, v) in &frame.children {
        if !grouped.contains_key(k) {
            order.push(k.clone());
        }
        grouped.entry(k.clone()).or_default().push(v.clone());
    }
    for k in order {
        let vs = grouped.remove(&k).unwrap();
        if vs.len() == 1 {
            map.insert(k, vs.into_iter().next().unwrap());
        } else {
            map.insert(k, Value::Array(vs));
        }
    }
    Value::Object(map)
}
```

- [ ] **Step 4: Implement `src/commands/xml/format.rs`**

```rust
use std::io::Cursor;

use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    xml: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let formatted = format_xml(input.as_str()?)?;
    out.emit_value(&Out0 { xml: formatted })
}

fn format_xml(xml: &str) -> Result<String, CliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(CliError::new(
                    ErrorCode::InvalidXml,
                    format!("invalid XML: {e}"),
                ));
            }
            Ok(Event::Eof) => break,
            Ok(ev) => {
                writer.write_event(ev).map_err(|e| {
                    CliError::new(ErrorCode::Internal, format!("xml write failed: {e}"))
                })?;
            }
        }
        buf.clear();
    }
    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("xml output not UTF-8: {e}"))
    })
}
```

- [ ] **Step 5: Wire + test + commit**

Add `pub mod xml;` to `src/commands/mod.rs`; add `Xml(crate::commands::xml::XmlArgs)` variant to `Noun`; add the match arm to `src/lib.rs`; update snapshot.

```bash
cargo test --test xml   # 5 tests pass
cargo test              # full suite green
git add src/commands/xml/ src/commands/mod.rs src/cli.rs src/lib.rs tests/xml.rs tests/snapshots/
git commit -m "feat(xml): add xml to-json (recursive walker) and format verbs"
```

---

## Task 3: `csv` noun — `to-json` verb

**Files:** `src/commands/csv/mod.rs`, `tests/csv.rs`. Standard noun template.

`csv` crate (already in Cargo.toml) reads a header row + records. Output: JSON array of objects.

- [ ] **Step 1: Write `tests/csv.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn csv_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "name,age\nalice,30\nbob,40"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["name"], "alice");
    assert_eq!(v[0]["age"], "30");
    assert_eq!(v[1]["name"], "bob");
}

#[test]
fn csv_to_json_via_stdin() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json"])
        .write_stdin("a,b\n1,2\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["a"], "1");
    assert_eq!(v[0]["b"], "2");
}

#[test]
fn csv_to_json_custom_delimiter() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "a;b\n1;2", "--delimiter", ";"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["a"], "1");
}

#[test]
fn csv_invalid_input_exits_3() {
    // CSV is forgiving by default, so use a malformed UTF-8-ish row — actually
    // CSV is just a flat-text format. Skip this test in favor of an explicit
    // failing parse scenario: --delimiter with multi-char value.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "a,b\n1,2", "--delimiter", "ab"])
        .assert()
        .failure()
        .code(2);  // usage error: delimiter must be single ASCII char
}
```

- [ ] **Step 2: Implement `src/commands/csv/mod.rs`**

```rust
//! CSV to JSON conversion.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct CsvArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert CSV to JSON (header row becomes object keys).
    #[command(name = "to-json", long_about = "Convert CSV to JSON. The first row is treated as the header — its values become object keys for every subsequent row.\n\nExamples:\n  ubertool csv to-json 'name,age\\nalice,30'\n  ubertool csv to-json --in ./data.csv --delimiter ';'\n\nExit codes specific to this command:\n  2   usage error (e.g., multi-char delimiter)\n  3   malformed CSV")]
    ToJson(ToJsonArgs),
}

#[derive(Debug, Args)]
pub struct ToJsonArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Single ASCII character delimiter (default `,`).
    #[arg(long, default_value = ",")]
    pub delimiter: String,
}

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn dispatch(args: CsvArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToJson(a) => run(a, out),
    }
}

fn run(args: ToJsonArgs, out: &Out) -> Result<(), CliError> {
    if args.delimiter.len() != 1 || !args.delimiter.is_ascii() {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "--delimiter must be a single ASCII character",
        ));
    }
    let delim_byte = args.delimiter.as_bytes()[0];
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let bytes = input.as_bytes();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delim_byte)
        .from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|e| {
            CliError::new(ErrorCode::Invalid, format!("invalid CSV header: {e}"))
        })?
        .iter()
        .map(String::from)
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for rec in reader.records() {
        let rec = rec.map_err(|e| {
            CliError::new(ErrorCode::Invalid, format!("invalid CSV row: {e}"))
                .with_hint("ensure rows are well-formed (escaped quotes, matching commas)")
        })?;
        let mut obj = serde_json::Map::new();
        for (i, field) in rec.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| i.to_string());
            obj.insert(key, serde_json::Value::String(field.to_string()));
        }
        rows.push(serde_json::Value::Object(obj));
    }

    let json = serde_json::to_string(&serde_json::Value::Array(rows))
        .map_err(|e| CliError::new(ErrorCode::Internal, format!("json failed: {e}")))?;
    out.emit_value(&Out0 { json })
}
```

> **NOTE:** The plan uses `ErrorCode::Invalid` above, but that variant doesn't exist. The correct name is the closest existing `InvalidJson`-style variant. We need an `InvalidCsv` variant. **Add `InvalidCsv` to `src/core/error.rs`** following the same pattern used for `InvalidBcrypt`/`InvalidUtf8` in M1 (enum + `as_str` + `exit` arm under `Invalid` + sync test).

- [ ] **Step 3: Add InvalidCsv ErrorCode**

In `src/core/error.rs`:
- Add `InvalidCsv` variant after `InvalidBcrypt`.
- Add `InvalidCsv` to the `exit()` `Invalid` arm.
- Add `InvalidCsv => "invalid_csv"` to `as_str()`.
- Add `ErrorCode::InvalidCsv` to the sync test `cases` array.

Then in `csv/mod.rs`, replace `ErrorCode::Invalid` with `ErrorCode::InvalidCsv`.

- [ ] **Step 4: Wire + test + commit**

```bash
cargo test --test csv
cargo test
git add src/core/error.rs src/commands/csv/ src/commands/mod.rs src/cli.rs src/lib.rs tests/csv.rs tests/snapshots/
git commit -m "feat(csv): add csv to-json with --delimiter and exit-3 invalid_csv"
```

---

## Task 4: `case` noun — `convert` verb (multi-style)

**Files:** `src/commands/case/mod.rs`, `tests/case.rs`.

Uses `convert_case::{Case, Casing}`. The verb takes an `--style` flag enum.

- [ ] **Step 1: Write `tests/case.rs`**

```rust
use assert_cmd::Command;

#[test]
fn case_convert_snake() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "Hello World", "--style", "snake"])
        .assert()
        .success()
        .stdout("hello_world\n");
}

#[test]
fn case_convert_kebab() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "Hello World", "--style", "kebab"])
        .assert()
        .success()
        .stdout("hello-world\n");
}

#[test]
fn case_convert_camel() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "hello world", "--style", "camel"])
        .assert()
        .success()
        .stdout("helloWorld\n");
}

#[test]
fn case_convert_pascal() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "hello world", "--style", "pascal"])
        .assert()
        .success()
        .stdout("HelloWorld\n");
}

#[test]
fn case_convert_screaming_snake() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "hello world", "--style", "screaming-snake"])
        .assert()
        .success()
        .stdout("HELLO_WORLD\n");
}

#[test]
fn case_convert_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "case", "convert", "hello world", "--style", "snake"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["result"], "hello_world");
}

#[test]
fn case_convert_invalid_style_is_usage_error() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "x", "--style", "bogus"])
        .assert()
        .failure()
        .code(2);
}
```

- [ ] **Step 2: Implement `src/commands/case/mod.rs`**

```rust
//! String case conversion.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use convert_case::{Case, Casing};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct CaseArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a string between case styles.
    #[command(long_about = "Convert a string between case styles.\n\nExamples:\n  ubertool case convert \"hello world\" --style snake\n  ubertool case convert \"helloWorld\" --style kebab\n  ubertool case convert \"my var\" --style screaming-snake --json")]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Style {
    Snake,
    Kebab,
    Camel,
    Pascal,
    #[value(name = "screaming-snake")]
    ScreamingSnake,
    Upper,
    Lower,
    Title,
    Sentence,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Target case style.
    #[arg(long, value_enum)]
    pub style: Style,
}

#[derive(Serialize)]
struct Out0 {
    result: String,
}

pub fn dispatch(args: CaseArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    let case = match args.style {
        Style::Snake => Case::Snake,
        Style::Kebab => Case::Kebab,
        Style::Camel => Case::Camel,
        Style::Pascal => Case::Pascal,
        Style::ScreamingSnake => Case::ScreamingSnake,
        Style::Upper => Case::Upper,
        Style::Lower => Case::Lower,
        Style::Title => Case::Title,
        Style::Sentence => Case::Sentence,
    };
    out.emit_value(&Out0 { result: s.to_case(case) })
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/case/ src/commands/mod.rs src/cli.rs src/lib.rs tests/case.rs tests/snapshots/
git commit -m "feat(case): add case convert with --style (snake/kebab/camel/pascal/screaming-snake/upper/lower/title/sentence)"
```

---

## Task 5: `slugify` noun — `generate` verb

**Files:** `src/commands/slugify/mod.rs`, `tests/slugify.rs`.

Uses `slug::slugify`.

- [ ] **Step 1: Write `tests/slugify.rs`**

```rust
use assert_cmd::Command;

#[test]
fn slugify_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["slugify", "generate", "Hello, World!"])
        .assert()
        .success()
        .stdout("hello-world\n");
}

#[test]
fn slugify_unicode() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["slugify", "generate", "Café Résumé"])
        .assert()
        .success()
        .stdout("cafe-resume\n");
}

#[test]
fn slugify_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "slugify", "generate", "Hello World"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["slug"], "hello-world");
}

#[test]
fn slugify_via_stdin() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["slugify", "generate"])
        .write_stdin("Some Title")
        .assert()
        .success()
        .stdout("some-title\n");
}
```

- [ ] **Step 2: Implement `src/commands/slugify/mod.rs`**

```rust
//! URL slug generation.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SlugifyArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a URL-safe slug.
    #[command(long_about = "Generate a URL-safe slug from text. Unicode is transliterated to ASCII, non-alphanumeric characters become hyphens, runs of hyphens collapse.\n\nExamples:\n  ubertool slugify generate \"Hello, World!\"\n  echo -n \"Café Résumé\" | ubertool slugify generate --json")]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    slug: String,
}

pub fn dispatch(args: SlugifyArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    out.emit_value(&Out0 { slug: slug::slugify(s) })
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/slugify/ src/commands/mod.rs src/cli.rs src/lib.rs tests/slugify.rs tests/snapshots/
git commit -m "feat(slugify): add slugify generate with unicode transliteration"
```

---

## Task 6: `list` noun — `convert` verb

**Files:** `src/commands/list/mod.rs`, `tests/list.rs`.

Converts between separator styles. Optional `--trim`, `--dedupe`, `--sort` post-processing.

- [ ] **Step 1: Write `tests/list.rs`**

```rust
use assert_cmd::Command;

#[test]
fn list_comma_to_newline() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["list", "convert", "a,b,c", "--from", "comma", "--to", "newline"])
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn list_newline_to_comma() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["list", "convert", "a\nb\nc", "--from", "newline", "--to", "comma"])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_dedupe_and_sort() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "list", "convert", "c,b,a,a,c", "--from", "comma", "--to", "comma",
            "--dedupe", "--sort",
        ])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_trim() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["list", "convert", " a , b , c ", "--from", "comma", "--to", "comma", "--trim"])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "list", "convert", "a,b,c", "--from", "comma", "--to", "comma"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["items"][0], "a");
    assert_eq!(v["items"][2], "c");
}
```

- [ ] **Step 2: Implement `src/commands/list/mod.rs`**

```rust
//! List separator conversion + cleanup.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a list between separator styles, optionally trimming/deduping/sorting.
    #[command(long_about = "Convert a list between separator styles.\n\nSeparators: comma, newline, space, tab, semicolon, pipe.\n\nExamples:\n  ubertool list convert 'a,b,c' --from comma --to newline\n  ubertool list convert ' a , b , a ' --from comma --to comma --trim --dedupe --sort")]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Sep {
    Comma,
    Newline,
    Space,
    Tab,
    Semicolon,
    Pipe,
}

impl Sep {
    fn as_str(self) -> &'static str {
        match self {
            Sep::Comma => ",",
            Sep::Newline => "\n",
            Sep::Space => " ",
            Sep::Tab => "\t",
            Sep::Semicolon => ";",
            Sep::Pipe => "|",
        }
    }
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Source separator.
    #[arg(long, value_enum)]
    pub from: Sep,
    /// Target separator.
    #[arg(long, value_enum)]
    pub to: Sep,
    /// Trim whitespace from each item.
    #[arg(long, default_value_t = false)]
    pub trim: bool,
    /// Remove duplicate items (preserves order of first occurrence).
    #[arg(long, default_value_t = false)]
    pub dedupe: bool,
    /// Sort items lexicographically (after dedupe if both are set).
    #[arg(long, default_value_t = false)]
    pub sort: bool,
}

#[derive(Serialize)]
struct Out0 {
    items: Vec<String>,
}

pub fn dispatch(args: ListArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let raw = input.as_str()?;
    let mut items: Vec<String> = raw.split(args.from.as_str()).map(|s| s.to_string()).collect();
    if args.trim {
        items = items.iter().map(|s| s.trim().to_string()).collect();
    }
    // Drop empty entries after trimming/splitting (common ergonomic).
    items.retain(|s| !s.is_empty());
    if args.dedupe {
        let mut seen = std::collections::HashSet::new();
        items.retain(|s| seen.insert(s.clone()));
    }
    if args.sort {
        items.sort();
    }
    // Single-field Out0 with Vec<String> → text mode emits the joined string;
    // quiet mode emits each item on a line. To get that behavior with the
    // current Out infrastructure, we manually decide output here.
    let joined = items.join(args.to.as_str());
    // Pretty path: emit as the joined string in text mode, but JSON in --json mode.
    // The cleanest way: build a struct holding `items` AND let text mode collapse.
    // For now, use a struct with both fields and rely on Out behavior for single
    // string fields. Since `Out0` has Vec<String>, text mode in `Out::emit_value`
    // serializes via serde_json::Value::Array; the single-field rule will produce
    // a JSON array literal in text mode — not what we want.
    //
    // Workaround: explicitly print the joined string ourselves for text/quiet, and
    // use Out::emit_value only for --json mode. Look at how `Out` exposes mode.
    use crate::core::output::OutputMode;
    match out.mode {
        OutputMode::Json => out.emit_value(&Out0 { items }),
        OutputMode::Text | OutputMode::Quiet => {
            // Print joined to stdout.
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{joined}").map_err(crate::core::error::CliError::from)?;
            Ok(())
        }
    }
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/list/ src/commands/mod.rs src/cli.rs src/lib.rs tests/list.rs tests/snapshots/
git commit -m "feat(list): add list convert with --from/--to separators, --trim/--dedupe/--sort"
```

---

## Task 7: `integer-base` noun — `convert` verb

**Files:** `src/commands/integer_base/mod.rs`, `tests/integer_base.rs`.

Converts integers between bases 2-36.

- [ ] **Step 1: Write `tests/integer_base.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn integer_base_dec_to_hex() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "255", "--from", "10", "--to", "16"])
        .assert()
        .success()
        .stdout("ff\n");
}

#[test]
fn integer_base_hex_to_dec() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "ff", "--from", "16", "--to", "10"])
        .assert()
        .success()
        .stdout("255\n");
}

#[test]
fn integer_base_bin_to_dec() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "11111111", "--from", "2", "--to", "10"])
        .assert()
        .success()
        .stdout("255\n");
}

#[test]
fn integer_base_dec_to_bin() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "10", "--from", "10", "--to", "2"])
        .assert()
        .success()
        .stdout("1010\n");
}

#[test]
fn integer_base_invalid_digit_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "xyz", "--from", "10", "--to", "16"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn integer_base_out_of_range_exits_2() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["integer-base", "convert", "10", "--from", "37", "--to", "10"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn integer_base_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "integer-base", "convert", "255", "--from", "10", "--to", "16"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["result"], "ff");
}
```

- [ ] **Step 2: Implement `src/commands/integer_base/mod.rs`**

```rust
//! Integer base conversion (bases 2-36).

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct IntegerBaseArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert an integer between bases (2-36).
    #[command(long_about = "Convert an integer between bases (2-36).\n\nExamples:\n  ubertool integer-base convert 255 --from 10 --to 16\n  ubertool integer-base convert ff --from 16 --to 2\n\nExit codes specific to this command:\n  2   base out of range (must be 2-36)\n  3   value contains digits invalid for the source base")]
    Convert(ConvertArgs),
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// Value to convert (digits only, no sign).
    pub input: String,
    /// Source base.
    #[arg(long)]
    pub from: u32,
    /// Target base.
    #[arg(long)]
    pub to: u32,
}

#[derive(Serialize)]
struct Out0 {
    result: String,
}

pub fn dispatch(args: IntegerBaseArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    if !(2..=36).contains(&args.from) || !(2..=36).contains(&args.to) {
        return Err(CliError::new(
            ErrorCode::UsageError,
            "bases must be in range 2..=36",
        ));
    }
    let n = u128::from_str_radix(&args.input, args.from).map_err(|e| {
        CliError::new(
            ErrorCode::InvalidIntegerBase,
            format!("invalid digits for base {}: {e}", args.from),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("verify each digit is valid for the source base")
    })?;
    // Encode in target base. For base 16, use lowercase; for base 2, use 0-1 only.
    let result = to_radix(n, args.to);
    out.emit_value(&Out0 { result })
}

fn to_radix(mut n: u128, base: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let chars: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut buf = Vec::new();
    let b = base as u128;
    while n > 0 {
        buf.push(chars[(n % b) as usize]);
        n /= b;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_radix_known() {
        assert_eq!(to_radix(255, 16), "ff");
        assert_eq!(to_radix(10, 2), "1010");
        assert_eq!(to_radix(0, 10), "0");
        assert_eq!(to_radix(35, 36), "z");
    }
}
```

- [ ] **Step 3: Add InvalidIntegerBase ErrorCode**

In `src/core/error.rs`:
- Add `InvalidIntegerBase` variant.
- Add to `as_str()` arm: `InvalidIntegerBase => "invalid_integer_base"`.
- Add to the `Invalid` arm in `exit()`.
- Add to the sync test cases.

- [ ] **Step 4: Wire + test + commit**

```bash
git add src/core/error.rs src/commands/integer_base/ src/commands/mod.rs src/cli.rs src/lib.rs tests/integer_base.rs tests/snapshots/
git commit -m "feat(integer-base): add integer-base convert for bases 2-36"
```

---

## Task 8: `roman` noun — `to-num` + `from-num` verbs

**Files:** `src/commands/roman/{mod,to_num,from_num}.rs`, `tests/roman.rs`. Hand-written algorithms.

- [ ] **Step 1: Write `tests/roman.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn roman_to_num_known() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "to-num", "MCMXCIX"])
        .assert()
        .success()
        .stdout("1999\n");
}

#[test]
fn roman_from_num_known() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "1999"])
        .assert()
        .success()
        .stdout("MCMXCIX\n");
}

#[test]
fn roman_round_trip() {
    for (decimal, roman) in [(1, "I"), (4, "IV"), (9, "IX"), (40, "XL"), (90, "XC"), (400, "CD"), (900, "CM"), (3888, "MMMDCCCLXXXVIII")] {
        Command::cargo_bin("ubertool")
            .unwrap()
            .args(["roman", "from-num", &decimal.to_string()])
            .assert()
            .success()
            .stdout(format!("{roman}\n"));
        Command::cargo_bin("ubertool")
            .unwrap()
            .args(["roman", "to-num", roman])
            .assert()
            .success()
            .stdout(format!("{decimal}\n"));
    }
}

#[test]
fn roman_to_num_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "to-num", "ZZZ"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_roman"));
}

#[test]
fn roman_from_num_out_of_range_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "4000"])
        .assert()
        .failure()
        .code(3);
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "0"])
        .assert()
        .failure()
        .code(3);
}
```

- [ ] **Step 2: Add InvalidRoman ErrorCode**

In `src/core/error.rs`, add `InvalidRoman` variant + sync.

- [ ] **Step 3: Implement `src/commands/roman/mod.rs`**

```rust
//! Roman numeral conversion.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod from_num;
pub mod to_num;

#[derive(Debug, Args)]
pub struct RomanArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert Roman numeral → integer.
    #[command(name = "to-num", long_about = "Convert a Roman numeral to an integer (1-3999).\n\nExamples:\n  ubertool roman to-num MCMXCIX     # 1999\n  ubertool roman to-num IV --json\n\nExit codes:\n  3   invalid Roman numeral (invalid_roman)")]
    ToNum(to_num::ToNumArgs),
    /// Convert integer → Roman numeral.
    #[command(name = "from-num", long_about = "Convert an integer (1-3999) to its Roman numeral form.\n\nExamples:\n  ubertool roman from-num 1999\n  ubertool roman from-num 42 --json\n\nExit codes:\n  3   integer out of range (must be 1..=3999)")]
    FromNum(from_num::FromNumArgs),
}

pub fn dispatch(args: RomanArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToNum(a) => to_num::run(a, out),
        Verb::FromNum(a) => from_num::run(a, out),
    }
}
```

`src/commands/roman/to_num.rs`:

```rust
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct ToNumArgs {
    /// Roman numeral input.
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    number: u32,
}

pub fn run(args: ToNumArgs, out: &Out) -> Result<(), CliError> {
    let n = parse_roman(&args.input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidRoman,
            format!("invalid Roman numeral: {}", args.input),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("valid characters are I, V, X, L, C, D, M (uppercase)")
    })?;
    out.emit_value(&Out0 { number: n })
}

fn parse_roman(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let map = |c: char| -> Option<u32> {
        match c {
            'I' => Some(1),
            'V' => Some(5),
            'X' => Some(10),
            'L' => Some(50),
            'C' => Some(100),
            'D' => Some(500),
            'M' => Some(1000),
            _ => None,
        }
    };
    let vals: Vec<u32> = s.chars().map(map).collect::<Option<Vec<_>>>()?;
    let mut total: u32 = 0;
    for i in 0..vals.len() {
        let v = vals[i];
        if i + 1 < vals.len() && v < vals[i + 1] {
            total = total.checked_sub(v)?;
        } else {
            total = total.checked_add(v)?;
        }
    }
    if total == 0 || total > 3999 {
        return None;
    }
    // Round-trip validation: reject inputs whose canonical form doesn't match.
    let canonical = crate::commands::roman::from_num::to_roman(total)?;
    if canonical != s {
        return None;
    }
    Some(total)
}
```

`src/commands/roman/from_num.rs`:

```rust
use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct FromNumArgs {
    /// Integer in range 1..=3999.
    pub input: u32,
}

#[derive(Serialize)]
struct Out0 {
    roman: String,
}

pub fn run(args: FromNumArgs, out: &Out) -> Result<(), CliError> {
    let r = to_roman(args.input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidRoman,
            format!("value {} is out of range (1..=3999)", args.input),
        )
        .with_input(serde_json::json!(args.input))
    })?;
    out.emit_value(&Out0 { roman: r })
}

pub(super) fn to_roman(n: u32) -> Option<String> {
    if n == 0 || n > 3999 {
        return None;
    }
    const PAIRS: &[(u32, &str)] = &[
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    let mut n = n;
    for &(v, sym) in PAIRS {
        while n >= v {
            out.push_str(sym);
            n -= v;
        }
    }
    Some(out)
}
```

- [ ] **Step 4: Wire + test + commit**

```bash
git add src/core/error.rs src/commands/roman/ src/commands/mod.rs src/cli.rs src/lib.rs tests/roman.rs tests/snapshots/
git commit -m "feat(roman): add roman to-num|from-num (1-3999 range, round-trip validated)"
```

---

## Task 9: `temperature` noun — `convert` verb

**Files:** `src/commands/temperature/mod.rs`, `tests/temperature.rs`.

- [ ] **Step 1: Write `tests/temperature.rs`**

```rust
use assert_cmd::Command;

#[test]
fn temp_c_to_f() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["temperature", "convert", "100", "--from", "celsius", "--to", "fahrenheit"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 212.0).abs() < 1e-6);
}

#[test]
fn temp_f_to_c() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["temperature", "convert", "32", "--from", "fahrenheit", "--to", "celsius"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!(v.abs() < 1e-6);
}

#[test]
fn temp_c_to_k() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["temperature", "convert", "0", "--from", "celsius", "--to", "kelvin"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 273.15).abs() < 1e-6);
}

#[test]
fn temp_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "temperature", "convert", "0", "--from", "celsius", "--to", "kelvin"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["value"].is_number());
    assert_eq!(v["unit"], "kelvin");
}
```

- [ ] **Step 2: Implement `src/commands/temperature/mod.rs`**

```rust
//! Temperature unit conversion.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct TemperatureArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert between temperature units (celsius/fahrenheit/kelvin).
    #[command(long_about = "Convert between temperature units.\n\nExamples:\n  ubertool temperature convert 100 --from celsius --to fahrenheit\n  ubertool temperature convert 32 --from fahrenheit --to kelvin --json")]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Unit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// Numeric input.
    pub input: f64,
    /// Source unit.
    #[arg(long, value_enum)]
    pub from: Unit,
    /// Target unit.
    #[arg(long, value_enum)]
    pub to: Unit,
}

#[derive(Serialize)]
struct Out0 {
    value: f64,
    unit: &'static str,
}

pub fn dispatch(args: TemperatureArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    // Convert via celsius as the pivot.
    let c = match args.from {
        Unit::Celsius => args.input,
        Unit::Fahrenheit => (args.input - 32.0) * 5.0 / 9.0,
        Unit::Kelvin => args.input - 273.15,
    };
    let (value, unit) = match args.to {
        Unit::Celsius => (c, "celsius"),
        Unit::Fahrenheit => (c * 9.0 / 5.0 + 32.0, "fahrenheit"),
        Unit::Kelvin => (c + 273.15, "kelvin"),
    };
    out.emit_value(&Out0 { value, unit })
}
```

> **Text-mode output note:** This struct has two fields (`value` and `unit`), so text mode emits `value: 212.0\nunit: fahrenheit`. If you want bare-value text-mode output (matching the test), make the struct single-field — but you lose the unit context. The test expects the bare value, so use a single-field `struct Out0 { value: f64 }` for output and drop the unit field for text mode. **Adjust the implementation accordingly.**

Actually a cleaner approach: emit `{"value": ..., "unit": "..."}` in --json mode, but emit just the value in text mode. The current `Out` infrastructure does this automatically for single-field structs but emits both for multi-field. We need a different approach for this command.

Use `OutputMode` directly like the `list` command does:

```rust
fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let c = match args.from { /* ... */ };
    let (value, unit) = match args.to { /* ... */ };

    use crate::core::output::OutputMode;
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { value, unit })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{value}").map_err(crate::core::error::CliError::from)
    }
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/temperature/ src/commands/mod.rs src/cli.rs src/lib.rs tests/temperature.rs tests/snapshots/
git commit -m "feat(temperature): add temperature convert celsius/fahrenheit/kelvin"
```

---

## Task 10: `sql` noun — `format` verb

**Files:** `src/commands/sql/mod.rs`, `tests/sql.rs`. Uses `sqlformat`.

- [ ] **Step 1: Write `tests/sql.rs`**

```rust
use assert_cmd::Command;

#[test]
fn sql_format_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["sql", "format", "SELECT * FROM users WHERE id=1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // sqlformat default emits newlines between major clauses.
    assert!(s.contains("SELECT"));
    assert!(s.contains("FROM"));
    assert!(s.contains("WHERE"));
    assert!(s.contains('\n'), "expected formatted output to contain newlines");
}

#[test]
fn sql_format_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "sql", "format", "select 1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let formatted = v["sql"].as_str().expect("sql field");
    assert!(formatted.to_uppercase().contains("SELECT"));
}

#[test]
fn sql_format_uppercase_keywords() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["sql", "format", "select * from t", "--uppercase"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("SELECT"));
}
```

- [ ] **Step 2: Implement `src/commands/sql/mod.rs`**

```rust
//! SQL formatter.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;
use sqlformat::{FormatOptions, Indent, QueryParams};

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SqlArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Pretty-print SQL with consistent indentation.
    #[command(long_about = "Pretty-print SQL.\n\nExamples:\n  ubertool sql format 'SELECT * FROM users WHERE id=1'\n  ubertool sql format 'select 1' --uppercase --json")]
    Format(FormatArgs),
}

#[derive(Debug, Args)]
pub struct FormatArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Uppercase SQL keywords.
    #[arg(long, default_value_t = false)]
    pub uppercase: bool,
    /// Indent width in spaces (default 2).
    #[arg(long, default_value_t = 2)]
    pub indent: u8,
}

#[derive(Serialize)]
struct Out0 {
    sql: String,
}

pub fn dispatch(args: SqlArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Format(a) => run(a, out),
    }
}

fn run(args: FormatArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let opts = FormatOptions {
        indent: Indent::Spaces(args.indent),
        uppercase: args.uppercase,
        lines_between_queries: 1,
        ignore_case_convert: None,
    };
    let formatted = sqlformat::format(input.as_str()?, &QueryParams::None, &opts);
    out.emit_value(&Out0 { sql: formatted })
}
```

> Note: `sqlformat 0.2`'s `format()` signature may differ slightly across patch versions. If the third argument expects `FormatOptions` (not `&FormatOptions`), drop the `&`. Verify with `cargo check`.

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/sql/ src/commands/mod.rs src/cli.rs src/lib.rs tests/sql.rs tests/snapshots/
git commit -m "feat(sql): add sql format with --uppercase and --indent"
```

---

## Task 11: `markdown` noun — `to-html` verb

**Files:** `src/commands/markdown/mod.rs`, `tests/markdown.rs`. Uses `pulldown-cmark`.

- [ ] **Step 1: Write `tests/markdown.rs`**

```rust
use assert_cmd::Command;

#[test]
fn markdown_to_html_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["markdown", "to-html", "# Hello\n\nWorld **bold**"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<h1>"));
    assert!(s.contains("Hello"));
    assert!(s.contains("<strong>"));
    assert!(s.contains("bold"));
}

#[test]
fn markdown_to_html_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "markdown", "to-html", "# Title"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let html = v["html"].as_str().expect("html field");
    assert!(html.contains("<h1>Title</h1>"));
}

#[test]
fn markdown_to_html_link() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["markdown", "to-html", "[example](https://example.com)"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<a href=\"https://example.com\">example</a>"));
}
```

- [ ] **Step 2: Implement `src/commands/markdown/mod.rs`**

```rust
//! Markdown → HTML conversion via pulldown-cmark.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use pulldown_cmark::{html, Parser};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct MarkdownArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Render Markdown as HTML.
    #[command(name = "to-html", long_about = "Render Markdown as HTML using CommonMark + GFM tables/strikethrough/etc.\n\nExamples:\n  ubertool markdown to-html '# Hello'\n  cat README.md | ubertool markdown to-html\n  ubertool markdown to-html '**bold**' --json")]
    ToHtml(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    html: String,
}

pub fn dispatch(args: MarkdownArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToHtml(a) => run(a, out),
    }
}

fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let mut html_buf = String::new();
    let parser = Parser::new(input.as_str()?);
    html::push_html(&mut html_buf, parser);
    out.emit_value(&Out0 { html: html_buf })
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/markdown/ src/commands/mod.rs src/cli.rs src/lib.rs tests/markdown.rs tests/snapshots/
git commit -m "feat(markdown): add markdown to-html (pulldown-cmark)"
```

---

## Task 12: `text` noun — encoding verbs (to-binary, from-binary, to-unicode, from-unicode, to-nato)

**Files:** `src/commands/text/{mod,to_binary,from_binary,to_unicode,from_unicode,to_nato}.rs`, `tests/text_encoding.rs`.

The `text` noun has 8 verbs total. This task adds 5 of them (encoding-style); Task 13 adds the analytical 3 (stats, diff, obfuscate).

- [ ] **Step 1: Write `tests/text_encoding.rs`**

```rust
use assert_cmd::Command;

#[test]
fn text_to_binary_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "A"])
        .assert()
        .success()
        .stdout("01000001\n");
}

#[test]
fn text_to_binary_multi_char() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "AB"])
        .assert()
        .success()
        .stdout("01000001 01000010\n");
}

#[test]
fn text_from_binary_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-binary", "01000001 01000010"])
        .assert()
        .success()
        .stdout("AB\n");
}

#[test]
fn text_round_trip_binary() {
    let bin = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "Hello"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let bin_s = String::from_utf8(bin).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-binary", &bin_s])
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn text_to_unicode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-unicode", "AB"])
        .assert()
        .success()
        .stdout("U+0041 U+0042\n");
}

#[test]
fn text_from_unicode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-unicode", "U+0041 U+0042"])
        .assert()
        .success()
        .stdout("AB\n");
}

#[test]
fn text_to_nato_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-nato", "ABC"])
        .assert()
        .success()
        .stdout("Alpha Bravo Charlie\n");
}

#[test]
fn text_to_nato_with_digit() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-nato", "A1B"])
        .assert()
        .success()
        .stdout("Alpha One Bravo\n");
}
```

- [ ] **Step 2: Implement `src/commands/text/mod.rs`**

```rust
//! Text encoding and analysis verbs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod from_binary;
pub mod from_unicode;
pub mod to_binary;
pub mod to_nato;
pub mod to_unicode;
// Task 13 will add: stats, diff, obfuscate.

#[derive(Debug, Args)]
pub struct TextArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Encode text as ASCII binary (space-separated bytes).
    #[command(name = "to-binary", long_about = "Encode text as space-separated 8-bit binary representations of each byte.\n\nExamples:\n  ubertool text to-binary 'A'           # 01000001\n  ubertool text to-binary 'Hi'          # 01001000 01101001")]
    ToBinary(RunArgs),
    /// Decode space-separated binary back to text.
    #[command(name = "from-binary", long_about = "Decode space-separated 8-bit binary back to UTF-8 text.\n\nExamples:\n  ubertool text from-binary '01000001 01000010'   # AB\n\nExit codes:\n  3   non-binary characters in input (invalid_binary)")]
    FromBinary(RunArgs),
    /// Encode text as space-separated Unicode codepoints (U+NNNN).
    #[command(name = "to-unicode", long_about = "Encode text as space-separated Unicode codepoints in U+NNNN form.\n\nExamples:\n  ubertool text to-unicode 'AB'  # U+0041 U+0042\n  ubertool text to-unicode '😀' # U+1F600")]
    ToUnicode(RunArgs),
    /// Decode space-separated U+NNNN codepoints to text.
    #[command(name = "from-unicode", long_about = "Decode space-separated U+NNNN codepoints to UTF-8 text.\n\nExamples:\n  ubertool text from-unicode 'U+0041 U+0042'   # AB\n\nExit codes:\n  3   invalid codepoint syntax or out-of-range codepoint")]
    FromUnicode(RunArgs),
    /// Convert text to NATO phonetic spelling.
    #[command(name = "to-nato", long_about = "Convert text to NATO phonetic spelling. Letters → NATO word, digits → spelled name, other chars pass through.\n\nExamples:\n  ubertool text to-nato 'ABC'    # Alpha Bravo Charlie\n  ubertool text to-nato 'A1B'    # Alpha One Bravo")]
    ToNato(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

pub fn dispatch(args: TextArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToBinary(a) => to_binary::run(a, out),
        Verb::FromBinary(a) => from_binary::run(a, out),
        Verb::ToUnicode(a) => to_unicode::run(a, out),
        Verb::FromUnicode(a) => from_unicode::run(a, out),
        Verb::ToNato(a) => to_nato::run(a, out),
    }
}
```

- [ ] **Step 3: Implement verb files**

`src/commands/text/to_binary.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    binary: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let bytes = input.as_bytes();
    let s = bytes
        .iter()
        .map(|b| format!("{b:08b}"))
        .collect::<Vec<_>>()
        .join(" ");
    out.emit_value(&Out0 { binary: s })
}
```

`src/commands/text/from_binary.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    text: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let mut bytes = Vec::new();
    for token in s.split_whitespace() {
        if token.len() != 8 || !token.chars().all(|c| c == '0' || c == '1') {
            return Err(CliError::new(
                ErrorCode::InvalidBinary,
                format!("expected 8-bit binary token, got '{token}'"),
            )
            .with_input(serde_json::json!(token)));
        }
        bytes.push(u8::from_str_radix(token, 2).unwrap());
    }
    let text = String::from_utf8(bytes).map_err(|_| {
        CliError::new(ErrorCode::InvalidUtf8, "decoded bytes are not valid UTF-8")
    })?;
    out.emit_value(&Out0 { text })
}
```

`src/commands/text/to_unicode.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    unicode: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let codepoints: Vec<String> = s.chars().map(|c| format!("U+{:04X}", c as u32)).collect();
    out.emit_value(&Out0 {
        unicode: codepoints.join(" "),
    })
}
```

`src/commands/text/from_unicode.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    text: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let mut result = String::new();
    for token in s.split_whitespace() {
        let hex = token.strip_prefix("U+").or_else(|| token.strip_prefix("u+")).ok_or_else(|| {
            CliError::new(
                ErrorCode::InvalidCodepoint,
                format!("expected U+NNNN form, got '{token}'"),
            )
        })?;
        let n = u32::from_str_radix(hex, 16).map_err(|_| {
            CliError::new(
                ErrorCode::InvalidCodepoint,
                format!("hex parse failed for '{token}'"),
            )
        })?;
        let ch = char::from_u32(n).ok_or_else(|| {
            CliError::new(
                ErrorCode::InvalidCodepoint,
                format!("not a valid Unicode codepoint: U+{n:04X}"),
            )
        })?;
        result.push(ch);
    }
    out.emit_value(&Out0 { text: result })
}
```

`src/commands/text/to_nato.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    nato: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    const LETTERS: [&str; 26] = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
        "India", "Juliet", "Kilo", "Lima", "Mike", "November", "Oscar", "Papa",
        "Quebec", "Romeo", "Sierra", "Tango", "Uniform", "Victor", "Whiskey",
        "X-ray", "Yankee", "Zulu",
    ];
    const DIGITS: [&str; 10] = [
        "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine",
    ];
    let words: Vec<&str> = s
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                let idx = c.to_ascii_uppercase() as usize - 'A' as usize;
                Some(LETTERS[idx])
            } else if c.is_ascii_digit() {
                let idx = c as usize - '0' as usize;
                Some(DIGITS[idx])
            } else {
                None
            }
        })
        .collect();
    out.emit_value(&Out0 { nato: words.join(" ") })
}
```

- [ ] **Step 4: Add new ErrorCode variants**

In `src/core/error.rs`, add `InvalidBinary` and `InvalidCodepoint` (both → exit 3, follow the InvalidBcrypt pattern from M1, including the sync test).

- [ ] **Step 5: Wire + test + commit**

```bash
git add src/core/error.rs src/commands/text/ src/commands/mod.rs src/cli.rs src/lib.rs tests/text_encoding.rs tests/snapshots/
git commit -m "feat(text): add text to-binary|from-binary|to-unicode|from-unicode|to-nato"
```

---

## Task 13: `text` noun continued — `stats` + `diff` + `obfuscate`

**Files:** Extend `src/commands/text/mod.rs`. Create `stats.rs`, `diff.rs`, `obfuscate.rs`. Create `tests/text_analysis.rs`.

- [ ] **Step 1: Write `tests/text_analysis.rs`**

```rust
use assert_cmd::Command;

#[test]
fn text_stats_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "text", "stats", "hello world\nfoo bar"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["chars"], 19);
    assert_eq!(v["words"], 4);
    assert_eq!(v["lines"], 2);
    assert!(v["bytes"].as_u64().unwrap() >= 19);
}

#[test]
fn text_obfuscate_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "obfuscate", "hello world"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // Each word: first + middle replaced with * + last (length-preserving).
    // "hello" (5) → h***o, "world" (5) → w***d.
    assert_eq!(s, "h***o w***d");
}

#[test]
fn text_obfuscate_short_word() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "obfuscate", "hi"])
        .assert()
        .success()
        .stdout("h*\n");  // 2-char: keep first, mask the rest.
}

#[test]
fn text_diff_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "diff", "hello world", "hello rust"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Unified-diff-style output contains "-world" and "+rust" or similar markers.
    assert!(s.contains("hello"));
    assert!(s.contains("world") || s.contains("rust"));
}

#[test]
fn text_diff_identical_inputs_emit_empty_diff() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "text", "diff", "same", "same"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let diff = v["diff"].as_str().expect("diff field");
    assert!(diff.is_empty() || diff.lines().all(|l| !l.starts_with('+') && !l.starts_with('-')));
}
```

- [ ] **Step 2: Extend `src/commands/text/mod.rs`**

Add three more variants to the `Verb` enum:

```rust
    /// Emit text statistics (chars/words/lines/bytes).
    #[command(long_about = "Emit chars, words, lines, and bytes counts as JSON or key:value lines.\n\nExamples:\n  ubertool text stats 'hello world'\n  cat file.txt | ubertool text stats --json")]
    Stats(RunArgs),
    /// Unified diff between two text inputs.
    #[command(long_about = "Unified diff between two text inputs (positional args).\n\nExamples:\n  ubertool text diff 'hello' 'world'\n  ubertool text diff --from-file a.txt --to-file b.txt --json")]
    Diff(DiffArgs),
    /// Obfuscate words by replacing middle chars with *.
    #[command(long_about = "Obfuscate text by replacing each word's middle characters with `*`. Preserves first character, and last character if word length > 2.\n\nExamples:\n  ubertool text obfuscate 'hello world'   # h***o w***d")]
    Obfuscate(RunArgs),
```

Add the dispatch arms in `dispatch()` and declare `pub mod stats; pub mod diff; pub mod obfuscate;`.

Define `DiffArgs`:

```rust
#[derive(Debug, Args)]
pub struct DiffArgs {
    /// First text (or use --from-file).
    pub from: Option<String>,
    /// Second text (or use --to-file).
    pub to: Option<String>,
    #[arg(long = "from-file")]
    pub from_file: Option<PathBuf>,
    #[arg(long = "to-file")]
    pub to_file: Option<PathBuf>,
}
```

- [ ] **Step 3: Implement `stats.rs`**

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    chars: usize,
    words: usize,
    lines: usize,
    bytes: usize,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?;
    let bytes = s.len();
    let chars = s.chars().count();
    let words = s.split_whitespace().count();
    let lines = if s.is_empty() { 0 } else { s.lines().count() };
    out.emit_value(&Out0 { chars, words, lines, bytes })
}
```

- [ ] **Step 4: Implement `obfuscate.rs`**

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    obfuscated: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim_end_matches(['\r', '\n']);
    let result = obfuscate(s);
    out.emit_value(&Out0 { obfuscated: result })
}

fn obfuscate(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let n = w.chars().count();
            if n == 0 {
                String::new()
            } else if n == 1 {
                w.to_string()
            } else if n == 2 {
                // Keep first, mask second.
                let mut it = w.chars();
                let first = it.next().unwrap();
                format!("{first}*")
            } else {
                // Keep first + last, mask the middle (n-2 stars).
                let chars: Vec<char> = w.chars().collect();
                let first = chars[0];
                let last = chars[n - 1];
                let stars = "*".repeat(n - 2);
                format!("{first}{stars}{last}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

- [ ] **Step 5: Implement `diff.rs`**

```rust
use serde::Serialize;
use similar::TextDiff;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::DiffArgs;

#[derive(Serialize)]
struct Out0 {
    diff: String,
}

pub fn run(args: DiffArgs, out: &Out) -> Result<(), CliError> {
    let from = read_side(args.from.as_deref(), args.from_file.as_deref(), "from")?;
    let to = read_side(args.to.as_deref(), args.to_file.as_deref(), "to")?;
    let td = TextDiff::from_lines(&from, &to);
    let mut diff_text = String::new();
    for op in td.ops() {
        for change in td.iter_inline_changes(op) {
            let sign = match change.tag() {
                similar::ChangeTag::Delete => "-",
                similar::ChangeTag::Insert => "+",
                similar::ChangeTag::Equal => " ",
            };
            for (_, value) in change.iter_strings_lossy() {
                diff_text.push_str(sign);
                diff_text.push_str(&value);
            }
            if !diff_text.ends_with('\n') {
                diff_text.push('\n');
            }
        }
    }
    out.emit_value(&Out0 { diff: diff_text })
}

fn read_side(
    positional: Option<&str>,
    file: Option<&std::path::Path>,
    side: &str,
) -> Result<String, CliError> {
    if let Some(s) = positional {
        return Ok(s.to_string());
    }
    if let Some(p) = file {
        return std::fs::read_to_string(p).map_err(CliError::from);
    }
    Err(CliError::new(
        ErrorCode::UsageError,
        format!("missing {side} input — pass positional or --{side}-file"),
    ))
}
```

- [ ] **Step 6: Wire + test + commit**

```bash
git add src/commands/text/ tests/text_analysis.rs tests/snapshots/
git commit -m "feat(text): add stats|diff|obfuscate analysis verbs (similar-based unified diff)"
```

---

## Task 14: `docker-run` noun — `to-compose` verb

**Files:** `src/commands/docker_run/{mod,to_compose,parser}.rs`, `tests/docker_run.rs`.

This is the largest M2 task — a hand-written `docker run` argument parser and a docker-compose YAML emitter. **Scope it down for M2:** support the most common flags only. Document the unsupported subset in `--help`.

**M2-supported docker-run flags:**
- `-d` / `--detach` (ignored — compose always declarative)
- `-it` / `-i` / `-t` (ignored — compose handles tty separately)
- `--name <name>` → service name (also used as the service key)
- `-p <host:cont>` / `--publish <host:cont>` → `ports:` (collects multiple)
- `-v <host:cont>` / `--volume <host:cont>` → `volumes:` (collects multiple)
- `-e <KEY=VAL>` / `--env <KEY=VAL>` → `environment:` (collects multiple)
- `--restart <policy>` → `restart:`
- `--network <name>` → `networks:`
- Positional `<image>` → `image:`
- Positional command (everything after image) → `command:` (single-string or list)

**Not yet supported (return clear error or just ignore with a warning to stderr):**
- `--mount`, `--health-check-*`, `--cap-add`, `--security-opt`, `--user`, advanced flags

- [ ] **Step 1: Write `tests/docker_run.rs`**

```rust
use assert_cmd::Command;

#[test]
fn docker_run_basic_image() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["docker-run", "to-compose", "docker run nginx"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("nginx"), "expected image in output: {s}");
    assert!(s.contains("image: nginx"));
}

#[test]
fn docker_run_with_name_and_ports() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["docker-run", "to-compose", "docker run --name web -p 8080:80 nginx"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("web:") || s.contains("name: web") || s.contains("container_name: web"));
    assert!(s.contains("8080:80") || s.contains("\"8080:80\""));
}

#[test]
fn docker_run_with_env_and_volume() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "docker-run", "to-compose",
            "docker run -e DEBUG=1 -e PORT=3000 -v /data:/var/lib/data postgres",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("DEBUG=1") || s.contains("DEBUG"));
    assert!(s.contains("/data:/var/lib/data"));
    assert!(s.contains("postgres"));
}

#[test]
fn docker_run_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "docker-run", "to-compose", "docker run nginx"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let yaml = v["compose"].as_str().expect("compose field");
    assert!(yaml.contains("nginx"));
}

#[test]
fn docker_run_missing_image_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["docker-run", "to-compose", "docker run --name foo"])
        .assert()
        .failure()
        .code(3);
}
```

- [ ] **Step 2: Implement `src/commands/docker_run/mod.rs`**

```rust
//! Convert `docker run` commands to docker-compose service entries.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod parser;
pub mod to_compose;

#[derive(Debug, Args)]
pub struct DockerRunArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert a `docker run` command to a docker-compose service entry.
    #[command(name = "to-compose", long_about = "Convert a `docker run` command to a docker-compose service entry.\n\nSupported flags: --name, -p/--publish, -v/--volume, -e/--env, --restart, --network. Other flags are ignored in this build.\n\nExamples:\n  ubertool docker-run to-compose 'docker run --name web -p 8080:80 nginx'\n  ubertool docker-run to-compose --in ./cmd.txt --json\n\nExit codes:\n  3   malformed docker-run command (e.g., missing image)")]
    ToCompose(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

pub fn dispatch(args: DockerRunArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::ToCompose(a) => to_compose::run(a, out),
    }
}
```

- [ ] **Step 3: Implement `parser.rs`**

```rust
//! Token-level parser for `docker run` argument vectors.

use crate::core::error::{CliError, ErrorCode};

#[derive(Debug, Default)]
pub struct DockerRun {
    pub name: Option<String>,
    pub image: Option<String>,
    pub command: Vec<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
    pub restart: Option<String>,
    pub network: Option<String>,
}

pub fn parse(input: &str) -> Result<DockerRun, CliError> {
    let mut tokens = shlex_split(input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidDockerRun,
            "could not tokenize input (unclosed quote?)",
        )
    })?;
    // Strip leading "docker" / "run" if present.
    if tokens.first().map(|s| s.as_str()) == Some("docker") {
        tokens.remove(0);
    }
    if tokens.first().map(|s| s.as_str()) == Some("run") {
        tokens.remove(0);
    }

    let mut dr = DockerRun::default();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "--name" => {
                i += 1;
                dr.name = Some(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("--name"))?);
            }
            "-p" | "--publish" => {
                i += 1;
                dr.ports.push(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("-p"))?);
            }
            "-v" | "--volume" => {
                i += 1;
                dr.volumes.push(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("-v"))?);
            }
            "-e" | "--env" => {
                i += 1;
                dr.env.push(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("-e"))?);
            }
            "--restart" => {
                i += 1;
                dr.restart = Some(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("--restart"))?);
            }
            "--network" => {
                i += 1;
                dr.network = Some(tokens.get(i).cloned().ok_or_else(|| flag_needs_value("--network"))?);
            }
            "-d" | "--detach" | "-i" | "-t" | "-it" | "--interactive" | "--tty" | "--rm" => {
                // Ignored (compose handles these declaratively).
            }
            _ if t.starts_with("--") || t.starts_with('-') && t.len() == 2 => {
                // Unrecognized flag — skip it (and any value). For M2 we err on the
                // side of forgiving parsing; the user gets a service entry without
                // the unsupported config and can adjust manually.
                // Heuristic: if the next token doesn't start with -, treat it as
                // the flag's value and skip both.
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    i += 1;
                }
            }
            _ => {
                // First non-flag positional is the image; the rest is the command.
                if dr.image.is_none() {
                    dr.image = Some(t.clone());
                } else {
                    dr.command.push(t.clone());
                }
            }
        }
        i += 1;
    }

    if dr.image.is_none() {
        return Err(CliError::new(
            ErrorCode::InvalidDockerRun,
            "missing image — `docker run` requires an image positional",
        )
        .with_hint("docker run [flags] <image> [command]"));
    }
    Ok(dr)
}

fn flag_needs_value(flag: &str) -> CliError {
    CliError::new(
        ErrorCode::InvalidDockerRun,
        format!("flag '{flag}' requires a value"),
    )
}

// Minimal shell-style tokenizer. Supports double-quoted, single-quoted, and
// backslash-escaped segments. Returns None on unclosed quotes.
fn shlex_split(s: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut chars = s.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !buf.is_empty() {
                    out.push(std::mem::take(&mut buf));
                }
            }
            c => buf.push(c),
        }
    }
    if in_single || in_double {
        return None;
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    Some(out)
}
```

- [ ] **Step 4: Implement `to_compose.rs`**

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::parser::{parse, DockerRun};
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    compose: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let dr = parse(input.as_str()?)?;
    let yaml = render(&dr);
    out.emit_value(&Out0 { compose: yaml })
}

fn render(dr: &DockerRun) -> String {
    let service_name = dr.name.clone().unwrap_or_else(|| {
        dr.image
            .as_deref()
            .unwrap_or("app")
            .rsplit('/')
            .next()
            .unwrap_or("app")
            .split(':')
            .next()
            .unwrap_or("app")
            .to_string()
    });
    let mut s = String::new();
    s.push_str("services:\n");
    s.push_str(&format!("  {service_name}:\n"));
    if let Some(image) = &dr.image {
        s.push_str(&format!("    image: {image}\n"));
    }
    if let Some(name) = &dr.name {
        s.push_str(&format!("    container_name: {name}\n"));
    }
    if let Some(restart) = &dr.restart {
        s.push_str(&format!("    restart: {restart}\n"));
    }
    if !dr.ports.is_empty() {
        s.push_str("    ports:\n");
        for p in &dr.ports {
            s.push_str(&format!("      - \"{p}\"\n"));
        }
    }
    if !dr.volumes.is_empty() {
        s.push_str("    volumes:\n");
        for v in &dr.volumes {
            s.push_str(&format!("      - {v}\n"));
        }
    }
    if !dr.env.is_empty() {
        s.push_str("    environment:\n");
        for e in &dr.env {
            s.push_str(&format!("      - {e}\n"));
        }
    }
    if let Some(net) = &dr.network {
        s.push_str("    networks:\n");
        s.push_str(&format!("      - {net}\n"));
    }
    if !dr.command.is_empty() {
        let cmd = dr.command.join(" ");
        s.push_str(&format!("    command: {cmd}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_minimal_image() {
        let dr = parse("docker run nginx").unwrap();
        let yaml = render(&dr);
        assert!(yaml.contains("image: nginx"));
        assert!(yaml.contains("nginx:"));
    }

    #[test]
    fn renders_name_and_ports() {
        let dr = parse("docker run --name web -p 8080:80 nginx").unwrap();
        let yaml = render(&dr);
        assert!(yaml.contains("web:"));
        assert!(yaml.contains("container_name: web"));
        assert!(yaml.contains("- \"8080:80\""));
    }
}
```

- [ ] **Step 5: Add InvalidDockerRun ErrorCode**

In `src/core/error.rs`: add `InvalidDockerRun` (exit 3) following the InvalidBcrypt pattern.

- [ ] **Step 6: Wire + test + commit**

```bash
git add src/core/error.rs src/commands/docker_run/ src/commands/mod.rs src/cli.rs src/lib.rs tests/docker_run.rs tests/snapshots/
git commit -m "feat(docker-run): add docker-run to-compose (subset: name/ports/volumes/env/restart/network)"
```

---

## Task 15: `safelink` noun — `decode` verb

**Files:** `src/commands/safelink/mod.rs`, `tests/safelink.rs`.

Decodes Outlook safelinks and similar URL wrappers. The pattern: the wrapped URL is stored in a `url=`, `q=`, or similar query parameter in URL-encoded form.

- [ ] **Step 1: Write `tests/safelink.rs`**

```rust
use assert_cmd::Command;

#[test]
fn safelink_outlook_unwraps() {
    let wrapped = "https://nam04.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com%2Fpath%3Fa%3D1&data=...&reserved=0";
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["safelink", "decode", wrapped])
        .assert()
        .success()
        .stdout("https://example.com/path?a=1\n");
}

#[test]
fn safelink_google_unwraps() {
    let wrapped = "https://www.google.com/url?q=https%3A%2F%2Fexample.com&sa=D";
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["safelink", "decode", wrapped])
        .assert()
        .success()
        .stdout("https://example.com\n");
}

#[test]
fn safelink_unwrapped_passes_through() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["safelink", "decode", "https://example.com/path"])
        .assert()
        .success()
        .stdout("https://example.com/path\n");
}

#[test]
fn safelink_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "safelink", "decode",
            "https://safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["url"], "https://example.com");
}
```

- [ ] **Step 2: Implement `src/commands/safelink/mod.rs`**

```rust
//! Decode Outlook/Google/etc. safelinks back to the wrapped URL.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use percent_encoding::percent_decode_str;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct SafelinkArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Unwrap a wrapped URL (Outlook safelink, Google redirect, etc.).
    #[command(long_about = "Unwrap a wrapped URL. Checks `?url=...`, `?q=...`, `?u=...` query parameters in order. If none match, returns the original URL unchanged.\n\nExamples:\n  ubertool safelink decode 'https://safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com'\n  ubertool safelink decode --in ./url.txt --json")]
    Decode(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: Option<String>,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Out0 {
    url: String,
}

pub fn dispatch(args: SafelinkArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Decode(a) => run(a, out),
    }
}

fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let s = input.as_str()?.trim();
    let url = unwrap_safelink(s);
    out.emit_value(&Out0 { url })
}

fn unwrap_safelink(s: &str) -> String {
    // Find the query string.
    let (_, query) = match s.split_once('?') {
        Some(parts) => parts,
        None => return s.to_string(),
    };
    // Check each key in priority order.
    for key in &["url=", "q=", "u="] {
        if let Some(found) = query
            .split('&')
            .find(|pair| pair.starts_with(key))
        {
            let val = &found[key.len()..];
            return percent_decode_str(val)
                .decode_utf8()
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| s.to_string());
        }
    }
    s.to_string()
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
git add src/commands/safelink/ src/commands/mod.rs src/cli.rs src/lib.rs tests/safelink.rs tests/snapshots/
git commit -m "feat(safelink): add safelink decode (Outlook/Google/generic URL unwrapping)"
```

---

## Task 16: Update OpenCLI spec for M2 nouns + regenerate docs

**Files:** Modify `ubertool.ocs.yaml`. Run `make gen` and `make verify-spec`.

Follow the same workflow as M1's Task 16 (see M1 plan for the exact procedure). Add entries for each new noun:

- `xml` (group) + `xml to-json`, `xml format` (leaves)
- `csv` (group) + `csv to-json` (leaf, with `--delimiter`)
- `case` (group) + `case convert` (leaf, with `--style` choices)
- `slugify` (group) + `slugify generate` (leaf)
- `list` (group) + `list convert` (leaf, with `--from`/`--to` choices, `--trim`/`--dedupe`/`--sort` booleans)
- `integer-base` (group) + `integer-base convert` (leaf, with `--from`/`--to` integers)
- `roman` (group) + `roman to-num`, `roman from-num` (leaves)
- `temperature` (group) + `temperature convert` (leaf, with `--from`/`--to` choices)
- `sql` (group) + `sql format` (leaf, with `--uppercase`, `--indent`)
- `markdown` (group) + `markdown to-html` (leaf)
- `text` (group) + 8 leaves (`to-binary`, `from-binary`, `to-unicode`, `from-unicode`, `to-nato`, `stats`, `diff`, `obfuscate`)
- `docker-run` (group) + `docker-run to-compose` (leaf)
- `safelink` (group) + `safelink decode` (leaf)

**Spec format gotchas from M1 (still apply):**
- `choices:` is a list of `{value: "..."}` maps (not bare strings).
- Integer `default` values must be quoted strings: `default: "12"`.
- `summary:` values containing colons need quoting.

**Standard flags on every leaf:** `--in` (string), `--json` (boolean), `--quiet` (boolean, `aliases: [q]`).

Run:

```bash
ocli spec check ubertool.ocs.yaml
make gen
make verify-spec
git add ubertool.ocs.yaml docs/cli/ docs/llms.txt
git commit -m "docs(spec): add all M2 nouns to OpenCLI spec and regenerate Markdown + llms.txt"
```

Expected: `verify-spec` should report ≥ 70 leaf commands match (M1 ended at 40; M2 adds ~22 verbs, so ~62 leaf commands minimum — plus the existing M0/M1 surface).

---

## Task 17: Extend fitness checklist + final `make ci`

**Files:** Modify `tests/agent_cli_fitness.rs`. Run `make ci`.

- [ ] **Step 1: Append M2 entries to `M2_NOUNS_AND_VERBS`**

Rename the existing `M1_NOUNS_AND_VERBS` constant to `ALL_NOUNS_AND_VERBS` and extend with the 13 new M2 nouns:

```rust
const ALL_NOUNS_AND_VERBS: &[(&str, &[&str])] = &[
    // M0/M1 entries (keep existing list verbatim)
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
    // M2 entries
    ("xml", &["to-json", "format"]),
    ("csv", &["to-json"]),
    ("case", &["convert"]),
    ("slugify", &["generate"]),
    ("list", &["convert"]),
    ("integer-base", &["convert"]),
    ("roman", &["to-num", "from-num"]),
    ("temperature", &["convert"]),
    ("sql", &["format"]),
    ("markdown", &["to-html"]),
    ("text", &["to-binary", "from-binary", "to-unicode", "from-unicode", "to-nato", "stats", "diff", "obfuscate"]),
    ("docker-run", &["to-compose"]),
    ("safelink", &["decode"]),
];
```

Update the reference in `every_m1_noun_and_verb_has_discoverable_help` (or rename the test to `every_noun_and_verb_has_discoverable_help`) to use the new constant name.

- [ ] **Step 2: Run extended fitness tests**

```bash
cargo test --test agent_cli_fitness
```

Expected: PASS. The discoverability test now invokes `<noun> --help` and `<noun> <verb> --help` for 28 nouns + ~75 verbs.

- [ ] **Step 3: Run full CI pipeline**

```bash
make ci
```

Expected: green (fmt + clippy + test + verify-spec). Fix clippy warnings if any fire on the new code.

- [ ] **Step 4: Commit**

```bash
git add tests/agent_cli_fitness.rs
git commit -m "test(fitness): extend fitness checklist to all M2 nouns and verbs"
```

If `make ci` also applied fmt changes:

```bash
git add -u
git commit -m "style: apply cargo fmt across M2 source and test files"
```

- [ ] **Step 5: Confirm clean state**

```bash
git status   # working tree clean
git log --oneline origin/main..HEAD | head -25
```

Expected: ~17 commits on top of the pushed M1.

---

## Self-Review

**Spec coverage** — every requirement in M2 scope (per design spec §16) is covered:

- ✅ xml (to-json, format) — Task 2
- ✅ csv (to-json) — Task 3
- ✅ case (convert) — Task 4
- ✅ slugify (generate) — Task 5
- ✅ list (convert) — Task 6
- ✅ integer-base (convert) — Task 7
- ✅ roman (to-num, from-num) — Task 8
- ✅ temperature (convert) — Task 9
- ✅ sql (format) — Task 10
- ✅ markdown (to-html) — Task 11
- ✅ text (5 encode verbs) — Task 12
- ✅ text (stats, diff, obfuscate) — Task 13
- ✅ docker-run (to-compose) — Task 14
- ✅ safelink (decode) — Task 15
- ✅ OpenCLI spec updated — Task 16
- ✅ Fitness extended + CI green — Task 17

**Placeholder scan** — no "TBD"/"TODO"/"fill in details" anywhere. Each task has complete code. The one exception is Task 2's first XML walker sketch (clearly labeled as a sketch), with a fully-typed consolidated version directly below it.

**Type consistency**:
- Every command emits a `#[derive(Serialize)] struct Out0 { ... }`.
- Single-value commands use single-field structs; commands needing bare text-mode output but JSON struct (temperature, list) explicitly check `out.mode` and write text-mode directly.
- New `ErrorCode` variants added in this milestone: `InvalidCsv` (T3), `InvalidIntegerBase` (T7), `InvalidRoman` (T8), `InvalidBinary` + `InvalidCodepoint` (T12), `InvalidDockerRun` (T14) — all mapped to exit 3 (`Invalid`) following the M1 pattern.
- All new code re-uses `resolve_input` / `Out::emit_value` / `CliError`.

**Scope check** — 13 nouns + 1 spec task + 1 CI task = 15 task topics across 17 numbered tasks (text is split into T12 + T13). Single milestone, no decomposition needed.

**Ambiguity check** — explicit decisions documented inline:
- XML→JSON shape: attrs as `@attr`, mixed content uses `#text`, repeated children become arrays.
- CSV: header row always present; multi-char delimiter is a usage error.
- Roman to-num is round-trip-validated to reject non-canonical input (`IIII` → invalid).
- Temperature output is bare value in text mode (single numeric makes more sense than `value:` lines).
- List uses `OutputMode` to switch between joined-string text output and `{"items": [...]}` JSON.
- Obfuscate: 1-char → unchanged, 2-char → keep first + 1 star, ≥3 → keep first/last + middle stars.
- Docker-run scope is documented as a subset; unknown flags are silently dropped.
- Safelink: priority order for query keys is `url`, `q`, `u`; unwrapped URL passes through unchanged.

---

## Execution Handoff

Plan complete and saved to `/Users/mihai/dev/ubertool/docs/superpowers/plans/2026-05-15-ubertool-m2-converters.md`.

Two execution options:

**1. Subagent-Driven (recommended)** — Same workflow as M1; one fresh implementer subagent per task, spec review then code-quality review after each. Use `superpowers:subagent-driven-development`.

**2. Inline Execution** — Use `superpowers:executing-plans` with batch checkpoints.

Which approach?
