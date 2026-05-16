# ubertool M3 — Tier 3 (Network & Calc) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 9 new nouns (~17 verbs) covering networking and calculation utilities on top of the M2 foundation: `ipv4`, `mac`, `ipv6-ula`, `math`, `percentage`, `eta`, `date`, `crontab`, `chmod`.

**Architecture:** Same as M1/M2. Each noun lives at `src/commands/<noun>/`. All commands plug into `core/` infrastructure.

**Tech Stack:** Adds to M0-M2: `ipnet` (IPv4/IPv6 parsing), `evalexpr` (math), `chrono`+`chrono-tz` (date/time), `cron` (cron expressions). Hand-written: chmod, MAC vendor lookup (bundled minimal CSV), ipv6-ula RFC4193, percentage, eta, crontab describer.

**Companion documents:**
- Design spec: `docs/superpowers/specs/2026-05-15-ubertool-cli-design.md` (§3 inventory, §11 crates).
- M2 plan (template): `docs/superpowers/plans/2026-05-15-ubertool-m2-converters.md`.
- M1 code (canonical patterns): `src/commands/{base64,hash,bcrypt,json}/`.

**Working directory:** `/Users/mihai/dev/ubertool/`. M2 just pushed (origin/main at `7a4bf8d`). Test count baseline: 204.

**The cross-cutting noun pattern** (same as M1/M2):
1. Create `src/commands/<noun>/` with `mod.rs` + per-verb files.
2. Add `pub mod <noun>;` to `src/commands/mod.rs`.
3. Add `<Noun>(...)` variant to `src/cli.rs::Noun`.
4. Add match arm to `src/lib.rs::run()`.
5. Write `tests/<noun>.rs`.
6. Run `cargo test`; update top-level help insta snapshot; commit.

**Acceptance criteria:**

1. `cargo build` and `cargo build --release` succeed.
2. `cargo test` passes 100%. Test count ≥ 250 (M2 ended at 204; M3 adds ≥ ~50 tests).
3. Every new noun appears in `ubertool --help`; every verb in `ubertool <noun> --help`.
4. Every data-returning command supports `--json`.
5. Fitness checklist extended to cover M3 nouns (Task 12).
6. `make verify-spec` passes — spec updated with M3 nouns (Task 11). Expected ≥ 78 leaf commands match.
7. `make ci` passes (fmt + clippy + test + verify-spec).
8. Repo clean.

---

## Cross-cutting conventions

Same as M1/M2 (see prior plans). Summary:
- Single-value commands: `#[derive(Serialize)] struct Out0 { <field>: T }`.
- Multi-value: named fields. Text mode emits `key: value\n` lines.
- Input via `resolve_input`.
- Errors via `CliError::new(ErrorCode::<Variant>, ...)`. Secrets redacted.
- Tests cover text mode, `--json` mode (parseable), error path with documented exit code, stdin pipe.

---

## Task 1: Add M3 dependencies

**Files:** Modify `Cargo.toml`.

- [ ] **Step 1: Append in `[dependencies]` after the M2 conversion section**

```toml
# M3 network / calc
ipnet = "2"
evalexpr = "11"
chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }
chrono-tz = "0.8"
cron = "0.12"
```

- [ ] **Step 2: Run `cargo check`**

Expected: succeeds. If a version is unavailable, substitute with the latest compatible and note.

- [ ] **Step 3: Run `cargo test`**

Expected: 204 M2 tests still pass.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add M3 dependencies (ipnet/evalexpr/chrono/chrono-tz/cron)"
```

---

## Task 2: `ipv4` noun — `parse` + `subnet` + `range-expand` + `to-ipv6`

**Files:** `src/commands/ipv4/{mod,parse,subnet,range_expand,to_ipv6}.rs`, `tests/ipv4.rs`. Adds `InvalidIp` ErrorCode.

The `ipv4` noun has 4 verbs. Implementation uses `std::net::Ipv4Addr`, `std::net::Ipv6Addr`, and `ipnet::Ipv4Net` for subnet math.

- [ ] **Step 1: Add `InvalidIp` ErrorCode to `src/core/error.rs`** (variant + `as_str` `"invalid_ip"` + `exit` `Invalid` arm + sync test cases).

- [ ] **Step 2: Write `tests/ipv4.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn ipv4_parse_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ipv4", "parse", "192.168.1.42"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["address"], "192.168.1.42");
    assert_eq!(v["decimal"], 3232235818u64);
    assert!(v["hex"].as_str().unwrap().contains("c0a8012a") || v["hex"].as_str().unwrap().contains("C0A8012A"));
    assert_eq!(v["binary"], "11000000.10101000.00000001.00101010");
    assert_eq!(v["is_private"], true);
}

#[test]
fn ipv4_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["ipv4", "parse", "999.999.999.999"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_ip"));
}

#[test]
fn ipv4_subnet_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ipv4", "subnet", "10.0.0.0/24"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["cidr"], "10.0.0.0/24");
    assert_eq!(v["network"], "10.0.0.0");
    assert_eq!(v["broadcast"], "10.0.0.255");
    assert_eq!(v["first_host"], "10.0.0.1");
    assert_eq!(v["last_host"], "10.0.0.254");
    assert_eq!(v["host_count"], 254);
    assert_eq!(v["mask"], "255.255.255.0");
    assert_eq!(v["prefix"], 24);
}

#[test]
fn ipv4_subnet_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["ipv4", "subnet", "not-a-cidr"])
        .assert().failure().code(3);
}

#[test]
fn ipv4_range_expand_small() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ipv4", "range-expand", "10.0.0.0/30"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let arr = v["addresses"].as_array().expect("addresses array");
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[0], "10.0.0.0");
    assert_eq!(arr[3], "10.0.0.3");
}

#[test]
fn ipv4_range_expand_caps_at_max() {
    // /16 has 65536 addresses; we cap the expansion to avoid OOM.
    Command::cargo_bin("ubertool").unwrap()
        .args(["ipv4", "range-expand", "10.0.0.0/16", "--max", "10"])
        .assert().success();
}

#[test]
fn ipv4_to_ipv6_mapped() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ipv4", "to-ipv6", "192.168.1.1"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    // IPv4-mapped IPv6: ::ffff:192.168.1.1 or ::ffff:c0a8:0101
    let mapped = v["ipv6_mapped"].as_str().expect("ipv6_mapped");
    assert!(mapped.contains("ffff"));
    assert!(mapped.contains("192.168.1.1") || mapped.contains("c0a8"));
}
```

- [ ] **Step 3: Create `src/commands/ipv4/mod.rs`**

```rust
//! IPv4 utilities.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod parse;
pub mod range_expand;
pub mod subnet;
pub mod to_ipv6;

#[derive(Debug, Args)]
pub struct Ipv4Args {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse an IPv4 address and emit its representations.
    #[command(long_about = "Parse an IPv4 address and emit decimal/hex/binary forms, plus private/loopback/multicast classification.\n\nExamples:\n  ubertool ipv4 parse 192.168.1.1\n  ubertool ipv4 parse 8.8.8.8 --json\n\nExit codes:\n  3   invalid IPv4 address (invalid_ip)")]
    Parse(RunArgs),
    /// Compute subnet info for a CIDR block.
    #[command(long_about = "Compute subnet info for a CIDR block (network, broadcast, first/last host, host count, mask).\n\nExamples:\n  ubertool ipv4 subnet 10.0.0.0/24\n  ubertool ipv4 subnet 192.168.1.0/26 --json\n\nExit codes:\n  3   invalid CIDR (invalid_ip)")]
    Subnet(RunArgs),
    /// Expand a CIDR block into a list of addresses (capped by --max).
    #[command(name = "range-expand", long_about = "Expand a CIDR block into the list of contained IPv4 addresses.\n\nThe --max flag (default 1024) caps the expansion so /8 doesn't allocate 16M strings.\n\nExamples:\n  ubertool ipv4 range-expand 10.0.0.0/30\n  ubertool ipv4 range-expand 192.168.1.0/24 --max 100 --json")]
    RangeExpand(RangeExpandArgs),
    /// Convert IPv4 to its IPv4-mapped IPv6 form (::ffff:a.b.c.d).
    #[command(name = "to-ipv6", long_about = "Convert IPv4 to its IPv4-mapped IPv6 form (::ffff:a.b.c.d) and IPv4-compatible form (::a.b.c.d).\n\nExamples:\n  ubertool ipv4 to-ipv6 192.168.1.1\n  ubertool ipv4 to-ipv6 8.8.8.8 --json")]
    ToIpv6(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: String,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RangeExpandArgs {
    pub input: String,
    /// Maximum addresses to emit (default 1024).
    #[arg(long, default_value_t = 1024)]
    pub max: usize,
}

pub fn dispatch(args: Ipv4Args, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => parse::run(a, out),
        Verb::Subnet(a) => subnet::run(a, out),
        Verb::RangeExpand(a) => range_expand::run(a, out),
        Verb::ToIpv6(a) => to_ipv6::run(a, out),
    }
}
```

- [ ] **Step 4: Implement the four verb files**

`src/commands/ipv4/parse.rs`:

```rust
use std::net::Ipv4Addr;
use std::str::FromStr;

use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    address: String,
    decimal: u32,
    hex: String,
    binary: String,
    is_private: bool,
    is_loopback: bool,
    is_multicast: bool,
    is_broadcast: bool,
    is_documentation: bool,
    is_link_local: bool,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let ip = Ipv4Addr::from_str(&args.input).map_err(|_| {
        CliError::new(ErrorCode::InvalidIp, format!("invalid IPv4 address: {}", args.input))
            .with_input(serde_json::json!(args.input))
            .with_hint("expected dotted-quad form like 192.168.1.1")
    })?;
    let octets = ip.octets();
    let decimal: u32 = u32::from_be_bytes(octets);
    let hex = format!("0x{decimal:08x}");
    let binary = octets
        .iter()
        .map(|b| format!("{b:08b}"))
        .collect::<Vec<_>>()
        .join(".");
    out.emit_value(&Out0 {
        address: ip.to_string(),
        decimal,
        hex,
        binary,
        is_private: ip.is_private(),
        is_loopback: ip.is_loopback(),
        is_multicast: ip.is_multicast(),
        is_broadcast: ip.is_broadcast(),
        is_documentation: ip.is_documentation(),
        is_link_local: ip.is_link_local(),
    })
}
```

`src/commands/ipv4/subnet.rs`:

```rust
use std::str::FromStr;

use ipnet::Ipv4Net;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    cidr: String,
    network: String,
    broadcast: String,
    first_host: String,
    last_host: String,
    host_count: u64,
    mask: String,
    prefix: u8,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let net = Ipv4Net::from_str(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidIp, format!("invalid CIDR: {e}"))
            .with_input(serde_json::json!(args.input))
            .with_hint("expected CIDR like 10.0.0.0/24")
    })?;
    let network = net.network();
    let broadcast = net.broadcast();
    let prefix = net.prefix_len();
    let total = if prefix >= 32 { 1u64 } else { 1u64 << (32 - prefix) };
    let usable = if total >= 2 { total - 2 } else { 0 };
    let net_oct = u32::from_be_bytes(network.octets());
    let bcast_oct = u32::from_be_bytes(broadcast.octets());
    let first_host = if total >= 2 {
        std::net::Ipv4Addr::from(net_oct + 1).to_string()
    } else {
        network.to_string()
    };
    let last_host = if total >= 2 {
        std::net::Ipv4Addr::from(bcast_oct - 1).to_string()
    } else {
        broadcast.to_string()
    };
    out.emit_value(&Out0 {
        cidr: format!("{}/{}", network, prefix),
        network: network.to_string(),
        broadcast: broadcast.to_string(),
        first_host,
        last_host,
        host_count: usable,
        mask: net.netmask().to_string(),
        prefix,
    })
}
```

`src/commands/ipv4/range_expand.rs`:

```rust
use std::str::FromStr;

use ipnet::Ipv4Net;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::RangeExpandArgs;

#[derive(Serialize)]
struct Out0 {
    addresses: Vec<String>,
    truncated: bool,
    total: u64,
}

pub fn run(args: RangeExpandArgs, out: &Out) -> Result<(), CliError> {
    let net = Ipv4Net::from_str(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidIp, format!("invalid CIDR: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    let prefix = net.prefix_len();
    let total = if prefix >= 32 { 1u64 } else { 1u64 << (32 - prefix) };
    let mut addresses = Vec::new();
    for (i, ip) in net.hosts().enumerate() {
        if i >= args.max {
            break;
        }
        addresses.push(ip.to_string());
    }
    let truncated = (addresses.len() as u64) < total;
    out.emit_value(&Out0 { addresses, truncated, total })
}
```

`src/commands/ipv4/to_ipv6.rs`:

```rust
use std::net::Ipv4Addr;
use std::str::FromStr;

use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    ipv4: String,
    ipv6_mapped: String,
    ipv6_compatible: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let ip = Ipv4Addr::from_str(&args.input).map_err(|_| {
        CliError::new(ErrorCode::InvalidIp, format!("invalid IPv4: {}", args.input))
    })?;
    let mapped = ip.to_ipv6_mapped();
    let compat = ip.to_ipv6_compatible();
    out.emit_value(&Out0 {
        ipv4: ip.to_string(),
        ipv6_mapped: mapped.to_string(),
        ipv6_compatible: compat.to_string(),
    })
}
```

- [ ] **Step 5: Wire + test + commit**

Add `pub mod ipv4;` to `src/commands/mod.rs`. Add `Ipv4` variant to `Noun` in `src/cli.rs`. Add dispatch arm to `src/lib.rs`. Update snapshot.

```bash
cargo test
git add src/core/error.rs src/commands/ipv4/ src/commands/mod.rs src/cli.rs src/lib.rs tests/ipv4.rs tests/snapshots/
git commit -m "feat(ipv4): add ipv4 parse|subnet|range-expand|to-ipv6 (ipnet)"
```

---

## Task 3: `mac` noun — `new` + `lookup`

**Files:** `src/commands/mac/{mod,new,lookup}.rs`, `src/data/oui.csv` (small curated set), `tests/mac.rs`.

**Scope:** `mac new` generates random 48-bit MACs (locally administered bit set). `mac lookup` looks up the vendor from the first 24 bits (OUI). For M3 we bundle a small curated CSV (~50 well-known vendors). The full IEEE OUI registry can replace this in a later milestone.

- [ ] **Step 1: Create `src/data/oui.csv`**

```csv
oui,vendor
000C29,VMware
001C42,Parallels
080027,Oracle VirtualBox
0050C2,IEEE Registration Authority
525400,QEMU
B827EB,Raspberry Pi Foundation
DCA632,Raspberry Pi Trading
E45F01,Raspberry Pi Trading
ACDE48,PRIVATE
00163E,Xensource
000569,VMware
001CC4,Hewlett Packard
0017F2,Apple
001124,Apple
F0F61C,Apple
04D3CF,Apple
000393,Apple
F0DBF8,Apple
000D93,Apple
000A95,Apple
3C0754,Apple
A45E60,Apple
60FACD,Apple
3CD0F8,Apple
F40F24,Apple
68AB1E,Apple
001CB3,Apple
68A86D,Apple
F0B479,Apple
AC8945,Cisco
0017C5,Cisco
54752E,Cisco
1C1D67,Cisco
D89E3F,Cisco
00059A,Cisco
00081A,Cisco
00904C,Espressif
3C71BF,Espressif
EC62F0,Espressif
246F28,Espressif
FCF5C4,Espressif
0CB815,Espressif
000DB9,PC Engines
1CC0E1,Cisco Meraki
000874,Dell
F8B156,Dell
B083FE,Dell
D43D7E,Lenovo
00163E,Xen
00219B,Dell
F08173,Dell
F45BD4,Microsoft
9C2A70,Microsoft
00125A,Microsoft
58FB84,Microsoft
54271E,Microsoft
B05ADA,Samsung
3868DD,Samsung
000DAE,Samsung
00125B,Texas Instruments
```

(Implementer: this is a sample curated list — feel free to expand or use a different small set. The lookup just needs to demonstrate the mechanism; full IEEE coverage is out of scope for M3.)

- [ ] **Step 2: Write `tests/mac.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn mac_new_format() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["mac", "new"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // colon-separated 6 hex pairs.
    let parts: Vec<&str> = s.split(':').collect();
    assert_eq!(parts.len(), 6);
    for p in &parts {
        assert_eq!(p.len(), 2);
        assert!(p.chars().all(|c| c.is_ascii_hexdigit()));
    }
    // Locally administered bit set: first octet's bit 1 (0x02) must be on.
    let first = u8::from_str_radix(parts[0], 16).unwrap();
    assert_eq!(first & 0x02, 0x02, "locally administered bit must be set");
}

#[test]
fn mac_new_two_calls_differ() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool").unwrap()
            .args(["mac", "new"]).assert().success().get_output().stdout.clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one());
}

#[test]
fn mac_lookup_apple() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mac", "lookup", "F0:F6:1C:00:11:22"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["vendor"].as_str().unwrap().to_lowercase().contains("apple"));
}

#[test]
fn mac_lookup_unknown_oui_emits_null_vendor() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mac", "lookup", "FF:FF:FF:00:00:00"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["vendor"].is_null());
}

#[test]
fn mac_lookup_invalid_mac_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["mac", "lookup", "not-a-mac"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_mac"));
}
```

- [ ] **Step 3: Add `InvalidMac` ErrorCode** to `src/core/error.rs` (variant + as_str + exit Invalid + sync test).

- [ ] **Step 4: Create `src/commands/mac/mod.rs`**

```rust
//! MAC address utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod lookup;
pub mod new;

#[derive(Debug, Args)]
pub struct MacArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random locally-administered MAC address.
    #[command(long_about = "Generate a random 48-bit MAC address with the locally-administered bit set (so it can't collide with real vendor MACs).\n\nExamples:\n  ubertool mac new\n  ubertool mac new --json")]
    New,
    /// Look up the vendor for a MAC address from the bundled OUI table.
    #[command(long_about = "Look up the vendor for a MAC address from the bundled OUI table. The OUI is the first 24 bits.\n\nThe bundled table covers ~50 common vendors. For full IEEE OUI coverage, future builds will ship a complete CSV.\n\nExamples:\n  ubertool mac lookup F0:F6:1C:00:11:22\n  ubertool mac lookup 00:1c:42:aa:bb:cc --json\n\nExit codes:\n  3   invalid MAC address (invalid_mac)")]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    /// MAC address (colon, hyphen, or dot-separated; case-insensitive).
    pub input: String,
}

pub fn dispatch(args: MacArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => new::run(out),
        Verb::Lookup(a) => lookup::run(a, out),
    }
}
```

- [ ] **Step 5: Create `src/commands/mac/new.rs`**

```rust
use rand::RngCore;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Serialize)]
struct Out0 {
    mac: String,
}

pub fn run(out: &Out) -> Result<(), CliError> {
    let mut bytes = [0u8; 6];
    rand::thread_rng().fill_bytes(&mut bytes);
    // Set locally-administered bit (bit 1 of first octet) and clear multicast bit (bit 0).
    bytes[0] = (bytes[0] | 0x02) & 0xfe;
    let s = bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    out.emit_value(&Out0 { mac: s })
}
```

- [ ] **Step 6: Create `src/commands/mac/lookup.rs`**

```rust
use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::LookupArgs;

#[derive(Serialize)]
struct Out0 {
    mac: String,
    oui: String,
    vendor: Option<String>,
}

const OUI_CSV: &str = include_str!("../../data/oui.csv");

fn oui_table() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::new();
        for line in OUI_CSV.lines().skip(1) {
            if let Some((oui, vendor)) = line.split_once(',') {
                map.insert(oui.trim().to_uppercase(), vendor.trim().to_string());
            }
        }
        map
    })
}

pub fn run(args: LookupArgs, out: &Out) -> Result<(), CliError> {
    let normalized = normalize_mac(&args.input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidMac,
            format!("invalid MAC address: {}", args.input),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("expected 6 hex pairs separated by `:`, `-`, or `.`")
    })?;
    let oui = normalized[..6].to_string();
    let vendor = oui_table().get(&oui).cloned();
    out.emit_value(&Out0 {
        mac: format_canonical(&normalized),
        oui,
        vendor,
    })
}

/// Strip all separators, uppercase, validate that 12 hex chars remain.
fn normalize_mac(s: &str) -> Option<String> {
    let mut buf = String::with_capacity(12);
    for c in s.chars() {
        if c == ':' || c == '-' || c == '.' {
            continue;
        }
        if !c.is_ascii_hexdigit() {
            return None;
        }
        buf.push(c.to_ascii_uppercase());
    }
    if buf.len() != 12 {
        return None;
    }
    Some(buf)
}

fn format_canonical(hex12: &str) -> String {
    let mut s = String::with_capacity(17);
    for (i, c) in hex12.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            s.push(':');
        }
        s.push(c);
    }
    s
}
```

- [ ] **Step 7: Wire + test + commit**

Make sure `src/data/` exists. The `include_str!("../../data/oui.csv")` path is relative to the file containing the macro, so from `src/commands/mac/lookup.rs` it resolves to `src/data/oui.csv`.

Add `pub mod mac;` to `src/commands/mod.rs`. Add `Mac(crate::commands::mac::MacArgs)` to `Noun`. Add dispatch arm. Update snapshot.

```bash
cargo test
git add src/core/error.rs src/commands/mac/ src/data/ src/commands/mod.rs src/cli.rs src/lib.rs tests/mac.rs tests/snapshots/
git commit -m "feat(mac): add mac new (locally-administered random) and mac lookup (bundled OUI)"
```

---

## Task 4: `ipv6-ula` noun — `new` verb

**Files:** `src/commands/ipv6_ula/mod.rs`, `tests/ipv6_ula.rs`.

Generates RFC4193 Unique Local IPv6 addresses. ULA prefix is `fc00::/7`. The L-bit (bit 7 of byte 0) must be 1 → `fd00::/8` for "locally assigned". The next 40 bits are a random Global ID. The next 16 bits are the subnet ID (default 0). The result is a /48 prefix.

- [ ] **Step 1: Write `tests/ipv6_ula.rs`**

```rust
use assert_cmd::Command;

#[test]
fn ipv6_ula_new_format() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["ipv6-ula", "new"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    // Must start with fd (locally assigned ULA).
    assert!(s.to_lowercase().starts_with("fd"), "ULA must start with fd: {s}");
    // Must end with /48 prefix.
    assert!(s.ends_with("/48"), "ULA expected /48 prefix: {s}");
}

#[test]
fn ipv6_ula_new_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ipv6-ula", "new"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["prefix"].as_str().unwrap().to_lowercase().starts_with("fd"));
    assert!(v["global_id"].is_string());
}

#[test]
fn ipv6_ula_two_calls_differ() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool").unwrap()
            .args(["ipv6-ula", "new"]).assert().success().get_output().stdout.clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one());
}
```

- [ ] **Step 2: Create `src/commands/ipv6_ula/mod.rs`**

```rust
//! RFC4193 Unique Local Address (ULA) generator.

use clap::{Args, Subcommand};
use rand::RngCore;
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct Ipv6UlaArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random IPv6 ULA /48 prefix.
    #[command(long_about = "Generate a random RFC4193 Unique Local IPv6 Address prefix (fd00::/8 with random 40-bit Global ID).\n\nExamples:\n  ubertool ipv6-ula new\n  ubertool ipv6-ula new --json")]
    New,
}

#[derive(Serialize)]
struct Out0 {
    prefix: String,
    global_id: String,
}

pub fn dispatch(args: Ipv6UlaArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => run(out),
    }
}

fn run(out: &Out) -> Result<(), CliError> {
    let mut buf = [0u8; 5];
    rand::thread_rng().fill_bytes(&mut buf);
    // fd / xx xx xx xx xx / 00 00 / 00 00 00 00 00 00 00 00 / 48
    let global_id = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3], buf[4]
    );
    let prefix = format!(
        "fd{:02x}:{:02x}{:02x}:{:02x}{:02x}::/48",
        buf[0], buf[1], buf[2], buf[3], buf[4]
    );
    out.emit_value(&Out0 { prefix, global_id })
}
```

- [ ] **Step 3: Wire + test + commit**

Add `pub mod ipv6_ula;` to `src/commands/mod.rs`.
Add to `src/cli.rs`:
```rust
    /// RFC4193 Unique Local IPv6 prefix generator.
    #[command(name = "ipv6-ula")]
    Ipv6Ula(crate::commands::ipv6_ula::Ipv6UlaArgs),
```
Add to `src/lib.rs`:
```rust
        cli::Noun::Ipv6Ula(a) => commands::ipv6_ula::dispatch(a, &out),
```

```bash
cargo test
git add src/commands/ipv6_ula/ src/commands/mod.rs src/cli.rs src/lib.rs tests/ipv6_ula.rs tests/snapshots/
git commit -m "feat(ipv6-ula): add ipv6-ula new (RFC4193 random /48 prefix)"
```

---

## Task 5: `math` noun — `eval` verb

**Files:** `src/commands/math/mod.rs`, `tests/math.rs`. Uses `evalexpr`.

- [ ] **Step 1: Write `tests/math.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn math_eval_simple() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["math", "eval", "1 + 2"])
        .assert().success()
        .stdout("3\n");
}

#[test]
fn math_eval_multiplication() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["math", "eval", "6 * 7"])
        .assert().success()
        .stdout("42\n");
}

#[test]
fn math_eval_float() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["math", "eval", "3.14 * 2"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 6.28).abs() < 1e-6);
}

#[test]
fn math_eval_parens_and_precedence() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["math", "eval", "(2 + 3) * 4"])
        .assert().success()
        .stdout("20\n");
}

#[test]
fn math_eval_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["math", "eval", "not math"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_math"));
}

#[test]
fn math_eval_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "math", "eval", "1 + 2"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    // result can be int or float; just confirm it's numeric.
    assert!(v["result"].is_number() || v["result"].as_i64() == Some(3));
}
```

- [ ] **Step 2: Add `InvalidMath` ErrorCode**

In `src/core/error.rs`: variant + `as_str` `"invalid_math"` + `exit` Invalid arm + sync test.

- [ ] **Step 3: Create `src/commands/math/mod.rs`**

```rust
//! Math expression evaluator (evalexpr).

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct MathArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Evaluate a math expression.
    #[command(long_about = "Evaluate a math expression. Supports +, -, *, /, %, ^, parens, common functions (sqrt, sin, cos, etc. via evalexpr).\n\nExamples:\n  ubertool math eval '(2 + 3) * 4'\n  ubertool math eval '3.14 * 2' --json\n\nExit codes:\n  3   invalid math expression (invalid_math)")]
    Eval(EvalArgs),
}

#[derive(Debug, Args)]
pub struct EvalArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    result: serde_json::Value,
}

pub fn dispatch(args: MathArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Eval(a) => run(a, out),
    }
}

fn run(args: EvalArgs, out: &Out) -> Result<(), CliError> {
    let v = evalexpr::eval(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidMath, format!("invalid math expression: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    let json_val = match v {
        evalexpr::Value::Int(i) => serde_json::json!(i),
        evalexpr::Value::Float(f) => serde_json::json!(f),
        evalexpr::Value::Boolean(b) => serde_json::json!(b),
        evalexpr::Value::String(s) => serde_json::json!(s),
        evalexpr::Value::Empty => serde_json::Value::Null,
        evalexpr::Value::Tuple(_) => {
            return Err(CliError::new(
                ErrorCode::InvalidMath,
                "expression evaluated to a tuple — not supported",
            ));
        }
    };
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { result: json_val })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        match &json_val {
            serde_json::Value::String(s) => writeln!(stdout, "{s}").map_err(CliError::from),
            other => writeln!(stdout, "{other}").map_err(CliError::from),
        }
    }
}
```

- [ ] **Step 4: Wire + test + commit**

```bash
cargo test
git add src/core/error.rs src/commands/math/ src/commands/mod.rs src/cli.rs src/lib.rs tests/math.rs tests/snapshots/
git commit -m "feat(math): add math eval (evalexpr expression evaluator)"
```

---

## Task 6: `percentage` noun — `of` + `change` + `of-total`

**Files:** `src/commands/percentage/{mod,of,change,of_total}.rs`, `tests/percentage.rs`.

Three small verbs, pure math.
- `percentage of <percent> <value>` → `percent% of value` (e.g., `20 of 50` → 10).
- `percentage change <from> <to>` → percentage change (e.g., `100 to 125` → +25%).
- `percentage of-total <part> <total>` → what % is `part` of `total` (e.g., `20 of 50` → 40%).

- [ ] **Step 1: Write `tests/percentage.rs`**

```rust
use assert_cmd::Command;

#[test]
fn pct_of() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "of", "20", "50"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 10.0).abs() < 1e-9);
}

#[test]
fn pct_change_positive() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "100", "125"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 25.0).abs() < 1e-9);
}

#[test]
fn pct_change_negative() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "100", "75"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v + 25.0).abs() < 1e-9);
}

#[test]
fn pct_of_total() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "of-total", "20", "50"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 40.0).abs() < 1e-9);
}

#[test]
fn pct_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "percentage", "of-total", "1", "4"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["percentage"].as_f64().unwrap() == 25.0);
}

#[test]
fn pct_change_zero_base_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "0", "100"])
        .assert().failure().code(3);
}
```

- [ ] **Step 2: Create `src/commands/percentage/mod.rs`**

```rust
//! Percentage calculations.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod change;
pub mod of;
pub mod of_total;

#[derive(Debug, Args)]
pub struct PercentageArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Compute "<percent>% of <value>".
    #[command(long_about = "Compute `<percent>% of <value>`.\n\nExamples:\n  ubertool percentage of 20 50    # 10\n  ubertool percentage of 15 80 --json")]
    Of(OfArgs),
    /// Compute percentage change from <from> to <to>.
    #[command(long_about = "Compute percentage change from <from> to <to>: `(to - from) / from * 100`.\n\nExamples:\n  ubertool percentage change 100 125   # 25\n  ubertool percentage change 100 75    # -25\n\nExit codes:\n  3   from value is zero (cannot compute percentage change)")]
    Change(ChangeArgs),
    /// Compute what percent <part> is of <total>.
    #[command(name = "of-total", long_about = "Compute what percent <part> is of <total>.\n\nExamples:\n  ubertool percentage of-total 20 50   # 40")]
    OfTotal(OfTotalArgs),
}

#[derive(Debug, Args)]
pub struct OfArgs {
    pub percent: f64,
    pub value: f64,
}

#[derive(Debug, Args)]
pub struct ChangeArgs {
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Args)]
pub struct OfTotalArgs {
    pub part: f64,
    pub total: f64,
}

pub fn dispatch(args: PercentageArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Of(a) => of::run(a, out),
        Verb::Change(a) => change::run(a, out),
        Verb::OfTotal(a) => of_total::run(a, out),
    }
}
```

- [ ] **Step 3: Implement each verb**

`src/commands/percentage/of.rs`:

```rust
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::{Out, OutputMode};

use super::OfArgs;

#[derive(Serialize)]
struct Out0 {
    percentage: f64,
    value: f64,
    result: f64,
}

pub fn run(args: OfArgs, out: &Out) -> Result<(), CliError> {
    let result = args.percent / 100.0 * args.value;
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 {
            percentage: args.percent,
            value: args.value,
            result,
        })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{result}").map_err(CliError::from)
    }
}
```

`src/commands/percentage/change.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

use super::ChangeArgs;

#[derive(Serialize)]
struct Out0 {
    from: f64,
    to: f64,
    percentage_change: f64,
}

pub fn run(args: ChangeArgs, out: &Out) -> Result<(), CliError> {
    if args.from == 0.0 {
        return Err(CliError::new(
            ErrorCode::InvalidMath,
            "percentage change is undefined when from = 0",
        ));
    }
    let change = (args.to - args.from) / args.from * 100.0;
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 {
            from: args.from,
            to: args.to,
            percentage_change: change,
        })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{change}").map_err(CliError::from)
    }
}
```

`src/commands/percentage/of_total.rs`:

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

use super::OfTotalArgs;

#[derive(Serialize)]
struct Out0 {
    part: f64,
    total: f64,
    percentage: f64,
}

pub fn run(args: OfTotalArgs, out: &Out) -> Result<(), CliError> {
    if args.total == 0.0 {
        return Err(CliError::new(
            ErrorCode::InvalidMath,
            "percentage of total is undefined when total = 0",
        ));
    }
    let pct = args.part / args.total * 100.0;
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 {
            part: args.part,
            total: args.total,
            percentage: pct,
        })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{pct}").map_err(CliError::from)
    }
}
```

- [ ] **Step 4: Wire + test + commit**

```bash
cargo test
git add src/commands/percentage/ src/commands/mod.rs src/cli.rs src/lib.rs tests/percentage.rs tests/snapshots/
git commit -m "feat(percentage): add percentage of|change|of-total"
```

---

## Task 7: `eta` noun — `calc` verb

**Files:** `src/commands/eta/mod.rs`, `tests/eta.rs`.

Calculates ETA from progress: given `done`, `total`, and `elapsed` (seconds), emit `remaining` (seconds) and `eta_iso` (absolute timestamp).

- [ ] **Step 1: Write `tests/eta.rs`**

```rust
use assert_cmd::Command;

#[test]
fn eta_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "eta", "calc", "--done", "50", "--total", "100", "--elapsed", "60"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    // Done 50/100 in 60s → 60s remaining.
    assert!((v["remaining_seconds"].as_f64().unwrap() - 60.0).abs() < 1e-6);
    assert!(v["eta_iso"].is_string());
    assert!(v["rate_per_second"].is_number());
}

#[test]
fn eta_done_equals_total_emits_zero_remaining() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "eta", "calc", "--done", "100", "--total", "100", "--elapsed", "60"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["remaining_seconds"].as_f64().unwrap(), 0.0);
}

#[test]
fn eta_zero_done_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["eta", "calc", "--done", "0", "--total", "100", "--elapsed", "60"])
        .assert().failure().code(3);
}
```

- [ ] **Step 2: Create `src/commands/eta/mod.rs`**

```rust
//! Estimated time of arrival from progress info.

use chrono::Utc;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct EtaArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Calculate ETA from progress (done/total/elapsed).
    #[command(long_about = "Calculate ETA from progress.\n\nGiven `--done` items completed out of `--total` in `--elapsed` seconds, compute remaining time and the absolute ETA timestamp (UTC, ISO 8601).\n\nExamples:\n  ubertool eta calc --done 50 --total 100 --elapsed 60\n\nExit codes:\n  3   --done is 0 (rate is unknown)")]
    Calc(CalcArgs),
}

#[derive(Debug, Args)]
pub struct CalcArgs {
    #[arg(long)]
    pub done: f64,
    #[arg(long)]
    pub total: f64,
    /// Elapsed seconds.
    #[arg(long)]
    pub elapsed: f64,
}

#[derive(Serialize)]
struct Out0 {
    done: f64,
    total: f64,
    elapsed_seconds: f64,
    rate_per_second: f64,
    remaining_seconds: f64,
    eta_iso: String,
}

pub fn dispatch(args: EtaArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Calc(a) => run(a, out),
    }
}

fn run(args: CalcArgs, out: &Out) -> Result<(), CliError> {
    if args.done <= 0.0 {
        return Err(CliError::new(
            ErrorCode::InvalidMath,
            "cannot estimate rate when --done is 0",
        ));
    }
    let rate = args.done / args.elapsed;
    let remaining = if args.done >= args.total {
        0.0
    } else {
        (args.total - args.done) / rate
    };
    let eta = Utc::now() + chrono::Duration::milliseconds((remaining * 1000.0) as i64);
    out.emit_value(&Out0 {
        done: args.done,
        total: args.total,
        elapsed_seconds: args.elapsed,
        rate_per_second: rate,
        remaining_seconds: remaining,
        eta_iso: eta.to_rfc3339(),
    })
}
```

- [ ] **Step 3: Wire + test + commit**

```bash
cargo test
git add src/commands/eta/ src/commands/mod.rs src/cli.rs src/lib.rs tests/eta.rs tests/snapshots/
git commit -m "feat(eta): add eta calc (chrono-based remaining-time + ISO 8601 ETA)"
```

---

## Task 8: `date` noun — `convert` verb

**Files:** `src/commands/date/mod.rs`, `tests/date.rs`. Uses `chrono` + `chrono-tz`.

Converts between unix timestamp, ISO 8601, RFC 2822, and a custom format. Optional `--tz` for timezone-aware output.

- [ ] **Step 1: Write `tests/date.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn date_convert_unix_to_iso() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "1700000000", "--from", "unix"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let iso = v["iso8601"].as_str().expect("iso8601 field");
    // 1700000000 = 2023-11-14T22:13:20Z
    assert!(iso.starts_with("2023-11-14"));
    assert!(v["unix"].as_i64() == Some(1700000000));
}

#[test]
fn date_convert_iso_to_unix() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "2023-11-14T22:13:20Z", "--from", "iso8601"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["unix"].as_i64(), Some(1700000000));
}

#[test]
fn date_convert_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["date", "convert", "not-a-date", "--from", "iso8601"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_date"));
}

#[test]
fn date_convert_with_tz() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "1700000000", "--from", "unix", "--tz", "America/New_York"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let local = v["local"].as_str().expect("local field present when --tz set");
    // Just confirm it contains the date.
    assert!(local.contains("2023-11-14") || local.contains("2023-11-15"));
}
```

- [ ] **Step 2: Add `InvalidDate` ErrorCode**

In `src/core/error.rs`: variant + `as_str` `"invalid_date"` + `exit` Invalid arm + sync test.

- [ ] **Step 3: Create `src/commands/date/mod.rs`**

```rust
//! Date / time format conversion.

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct DateArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert between date/time formats.
    #[command(long_about = "Convert between unix timestamp, ISO 8601, and RFC 2822 representations of a moment in time.\n\nExamples:\n  ubertool date convert 1700000000 --from unix\n  ubertool date convert '2023-11-14T22:13:20Z' --from iso8601 --json\n  ubertool date convert 1700000000 --from unix --tz America/New_York\n\nExit codes:\n  3   invalid date/time input (invalid_date)")]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Unix,
    Iso8601,
    Rfc2822,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: String,
    /// Source format.
    #[arg(long, value_enum)]
    pub from: Format,
    /// Optional timezone (IANA name, e.g., America/New_York).
    #[arg(long)]
    pub tz: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    unix: i64,
    iso8601: String,
    rfc2822: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    local: Option<String>,
}

pub fn dispatch(args: DateArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let dt: DateTime<Utc> = match args.from {
        Format::Unix => {
            let ts: i64 = args.input.parse().map_err(|_| {
                CliError::new(ErrorCode::InvalidDate, format!("invalid unix timestamp: {}", args.input))
            })?;
            Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
                CliError::new(ErrorCode::InvalidDate, format!("timestamp out of range: {ts}"))
            })?
        }
        Format::Iso8601 => DateTime::parse_from_rfc3339(&args.input)
            .map_err(|e| {
                CliError::new(ErrorCode::InvalidDate, format!("invalid ISO 8601: {e}"))
                    .with_input(serde_json::json!(args.input))
            })?
            .with_timezone(&Utc),
        Format::Rfc2822 => DateTime::parse_from_rfc2822(&args.input)
            .map_err(|e| {
                CliError::new(ErrorCode::InvalidDate, format!("invalid RFC 2822: {e}"))
            })?
            .with_timezone(&Utc),
    };

    let local = if let Some(tz_name) = &args.tz {
        let tz: Tz = tz_name.parse().map_err(|_| {
            CliError::new(
                ErrorCode::InvalidDate,
                format!("unknown timezone: {tz_name}"),
            )
            .with_hint("use IANA timezone names like America/New_York or Europe/Berlin")
        })?;
        Some(dt.with_timezone(&tz).to_rfc3339())
    } else {
        None
    };

    out.emit_value(&Out0 {
        unix: dt.timestamp(),
        iso8601: dt.to_rfc3339(),
        rfc2822: dt.to_rfc2822(),
        local,
    })
}
```

- [ ] **Step 4: Wire + test + commit**

```bash
cargo test
git add src/core/error.rs src/commands/date/ src/commands/mod.rs src/cli.rs src/lib.rs tests/date.rs tests/snapshots/
git commit -m "feat(date): add date convert unix/iso8601/rfc2822 with --tz support"
```

---

## Task 9: `crontab` noun — `describe` + `next`

**Files:** `src/commands/crontab/{mod,describe,next}.rs`, `tests/crontab.rs`. Uses `cron` crate for `next`; describer is hand-written.

- [ ] **Step 1: Write `tests/crontab.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn crontab_describe_every_minute() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "describe", "* * * * *"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap().to_lowercase();
    assert!(s.contains("every minute"));
}

#[test]
fn crontab_describe_daily_at_specific_time() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "describe", "30 14 * * *"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("14:30") || s.contains("2:30"));
}

#[test]
fn crontab_next_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "crontab", "next", "0 * * * *"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let arr = v["next"].as_array().expect("next array");
    assert_eq!(arr.len(), 5);  // default count
    for entry in arr {
        assert!(entry.is_string());
    }
}

#[test]
fn crontab_next_custom_count() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "crontab", "next", "0 * * * *", "--count", "3"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["next"].as_array().unwrap().len(), 3);
}

#[test]
fn crontab_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "next", "not a cron"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_cron"));
}
```

- [ ] **Step 2: Add `InvalidCron` ErrorCode**

In `src/core/error.rs`: variant + `as_str` `"invalid_cron"` + `exit` Invalid arm + sync test.

- [ ] **Step 3: Create `src/commands/crontab/mod.rs`**

```rust
//! Crontab expression utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod describe;
pub mod next;

#[derive(Debug, Args)]
pub struct CrontabArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Describe a crontab expression in plain English.
    #[command(long_about = "Describe a crontab expression in plain English.\n\nSupports the common Unix 5-field format: minute hour day-of-month month day-of-week.\n\nExamples:\n  ubertool crontab describe '* * * * *'      # 'every minute'\n  ubertool crontab describe '30 14 * * *'    # 'at 14:30 every day'")]
    Describe(DescribeArgs),
    /// Emit the next N matching timestamps.
    #[command(long_about = "Emit the next N matching timestamps for a crontab expression (defaults to 5; --count overrides).\n\nExamples:\n  ubertool crontab next '0 * * * *'\n  ubertool crontab next '*/15 * * * *' --count 10 --json\n\nExit codes:\n  3   invalid cron expression (invalid_cron)")]
    Next(NextArgs),
}

#[derive(Debug, Args)]
pub struct DescribeArgs {
    pub input: String,
}

#[derive(Debug, Args)]
pub struct NextArgs {
    pub input: String,
    #[arg(long, default_value_t = 5)]
    pub count: usize,
}

pub fn dispatch(args: CrontabArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Describe(a) => describe::run(a, out),
        Verb::Next(a) => next::run(a, out),
    }
}
```

- [ ] **Step 4: Create `src/commands/crontab/describe.rs`**

Hand-written. Handles common patterns; falls back to a literal echo for complex expressions.

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::DescribeArgs;

#[derive(Serialize)]
struct Out0 {
    description: String,
}

pub fn run(args: DescribeArgs, out: &Out) -> Result<(), CliError> {
    let parts: Vec<&str> = args.input.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(CliError::new(
            ErrorCode::InvalidCron,
            format!("expected 5 fields, got {}", parts.len()),
        )
        .with_hint("format: minute hour day-of-month month day-of-week"));
    }
    let description = describe(&parts);
    out.emit_value(&Out0 { description })
}

fn describe(p: &[&str]) -> String {
    let (m, h, dom, mon, dow) = (p[0], p[1], p[2], p[3], p[4]);

    // Match common shortcuts first.
    let all_stars = p.iter().all(|f| *f == "*");
    if all_stars {
        return "every minute".to_string();
    }
    if m != "*" && h != "*" && dom == "*" && mon == "*" && dow == "*" {
        if let (Ok(min), Ok(hour)) = (m.parse::<u32>(), h.parse::<u32>()) {
            return format!("at {hour:02}:{min:02} every day");
        }
    }
    if m == "0" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "at the top of every hour".to_string();
    }
    if let Some(stripped) = m.strip_prefix("*/") {
        if h == "*" && dom == "*" && mon == "*" && dow == "*" {
            if let Ok(step) = stripped.parse::<u32>() {
                return format!("every {step} minutes");
            }
        }
    }

    // Fallback: describe each field literally.
    let mut bits = Vec::new();
    bits.push(field_desc("minute", m));
    bits.push(field_desc("hour", h));
    bits.push(field_desc("day-of-month", dom));
    bits.push(field_desc("month", mon));
    bits.push(field_desc("day-of-week", dow));
    bits.join(", ")
}

fn field_desc(name: &str, v: &str) -> String {
    if v == "*" {
        format!("every {name}")
    } else if let Some(stripped) = v.strip_prefix("*/") {
        format!("every {stripped} {name}s")
    } else {
        format!("{name} = {v}")
    }
}
```

- [ ] **Step 5: Create `src/commands/crontab/next.rs`**

```rust
use std::str::FromStr;

use chrono::Utc;
use cron::Schedule;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::NextArgs;

#[derive(Serialize)]
struct Out0 {
    next: Vec<String>,
}

pub fn run(args: NextArgs, out: &Out) -> Result<(), CliError> {
    // The cron crate expects 6 or 7 fields (seconds optional). Many users pass
    // 5 fields; prepend "0" for seconds when we see 5.
    let normalized = if args.input.split_whitespace().count() == 5 {
        format!("0 {}", args.input)
    } else {
        args.input.clone()
    };
    let schedule = Schedule::from_str(&normalized).map_err(|e| {
        CliError::new(ErrorCode::InvalidCron, format!("invalid cron expression: {e}"))
            .with_input(serde_json::json!(args.input))
            .with_hint("format: minute hour day-of-month month day-of-week")
    })?;

    let now = Utc::now();
    let entries: Vec<String> = schedule
        .after(&now)
        .take(args.count)
        .map(|dt| dt.to_rfc3339())
        .collect();

    out.emit_value(&Out0 { next: entries })
}
```

- [ ] **Step 6: Wire + test + commit**

```bash
cargo test
git add src/core/error.rs src/commands/crontab/ src/commands/mod.rs src/cli.rs src/lib.rs tests/crontab.rs tests/snapshots/
git commit -m "feat(crontab): add crontab describe (hand-written) and next (cron + chrono)"
```

---

## Task 10: `chmod` noun — `calc` + `parse`

**Files:** `src/commands/chmod/{mod,calc,parse}.rs`, `tests/chmod.rs`.

- `chmod calc <symbolic>` → octal (e.g., `rwxr-xr-x` → `755`).
- `chmod parse <octal>` → symbolic + per-class flags (e.g., `755` → `rwxr-xr-x`).

- [ ] **Step 1: Write `tests/chmod.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn chmod_calc_755() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["chmod", "calc", "rwxr-xr-x"])
        .assert().success()
        .stdout("755\n");
}

#[test]
fn chmod_calc_644() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["chmod", "calc", "rw-r--r--"])
        .assert().success()
        .stdout("644\n");
}

#[test]
fn chmod_calc_777() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["chmod", "calc", "rwxrwxrwx"])
        .assert().success()
        .stdout("777\n");
}

#[test]
fn chmod_parse_755() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "chmod", "parse", "755"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["symbolic"], "rwxr-xr-x");
    assert_eq!(v["owner"]["read"], true);
    assert_eq!(v["owner"]["write"], true);
    assert_eq!(v["owner"]["execute"], true);
    assert_eq!(v["group"]["write"], false);
    assert_eq!(v["other"]["write"], false);
}

#[test]
fn chmod_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["chmod", "parse", "999"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_chmod"));
}

#[test]
fn chmod_calc_invalid_symbolic_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["chmod", "calc", "rwxabcdef"])
        .assert().failure().code(3);
}
```

- [ ] **Step 2: Add `InvalidChmod` ErrorCode**

In `src/core/error.rs`: variant + `as_str` `"invalid_chmod"` + `exit` Invalid arm + sync test.

- [ ] **Step 3: Create `src/commands/chmod/mod.rs`**

```rust
//! Unix file mode conversion (symbolic ↔ octal).

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod calc;
pub mod parse;

#[derive(Debug, Args)]
pub struct ChmodArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert symbolic mode (rwxr-xr-x) → octal (755).
    #[command(long_about = "Convert symbolic mode like `rwxr-xr-x` to octal `755`.\n\nExamples:\n  ubertool chmod calc rwxr-xr-x\n  ubertool chmod calc rw-r--r-- --json\n\nExit codes:\n  3   invalid symbolic input (invalid_chmod)")]
    Calc(CalcArgs),
    /// Convert octal mode (755) → symbolic + per-class breakdown.
    #[command(long_about = "Convert octal mode like `755` to symbolic `rwxr-xr-x` plus per-class (owner/group/other) read/write/execute flags.\n\nExamples:\n  ubertool chmod parse 755\n  ubertool chmod parse 644 --json\n\nExit codes:\n  3   octal must be 3 digits each 0-7 (invalid_chmod)")]
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct CalcArgs {
    pub input: String,
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

pub fn dispatch(args: ChmodArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Calc(a) => calc::run(a, out),
        Verb::Parse(a) => parse::run(a, out),
    }
}
```

- [ ] **Step 4: Create `src/commands/chmod/calc.rs`**

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::CalcArgs;

#[derive(Serialize)]
struct Out0 {
    octal: String,
}

pub fn run(args: CalcArgs, out: &Out) -> Result<(), CliError> {
    let s = args.input.as_str();
    if s.len() != 9 {
        return Err(CliError::new(
            ErrorCode::InvalidChmod,
            format!("symbolic mode must be 9 chars, got {}", s.len()),
        )
        .with_input(serde_json::json!(args.input)));
    }
    let bytes = s.as_bytes();
    let owner = class_digit(&bytes[0..3])?;
    let group = class_digit(&bytes[3..6])?;
    let other = class_digit(&bytes[6..9])?;
    let octal = format!("{owner}{group}{other}");
    out.emit_value(&Out0 { octal })
}

fn class_digit(triplet: &[u8]) -> Result<u32, CliError> {
    let mut bits = 0u32;
    let expected: [(u8, u8); 3] = [(b'r', 4), (b'w', 2), (b'x', 1)];
    for (i, (sym, bit)) in expected.iter().enumerate() {
        match triplet[i] {
            c if c == *sym => bits |= u32::from(*bit),
            b'-' => {}
            other => {
                return Err(CliError::new(
                    ErrorCode::InvalidChmod,
                    format!("unexpected char `{}` at position {i} in triplet", other as char),
                ));
            }
        }
    }
    Ok(bits)
}
```

- [ ] **Step 5: Create `src/commands/chmod/parse.rs`**

```rust
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::ParseArgs;

#[derive(Serialize)]
struct ClassFlags {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Serialize)]
struct Out0 {
    octal: String,
    symbolic: String,
    owner: ClassFlags,
    group: ClassFlags,
    other: ClassFlags,
}

pub fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    if args.input.len() != 3 {
        return Err(CliError::new(
            ErrorCode::InvalidChmod,
            format!("expected 3-digit octal, got {}", args.input),
        ));
    }
    let mut digits = [0u32; 3];
    for (i, c) in args.input.chars().enumerate() {
        let d = c
            .to_digit(8)
            .ok_or_else(|| CliError::new(ErrorCode::InvalidChmod, format!("invalid octal digit `{c}`")))?;
        if d > 7 {
            return Err(CliError::new(
                ErrorCode::InvalidChmod,
                format!("digit `{c}` is out of range 0-7"),
            ));
        }
        digits[i] = d;
    }
    let symbolic = format!(
        "{}{}{}",
        triplet(digits[0]),
        triplet(digits[1]),
        triplet(digits[2])
    );
    out.emit_value(&Out0 {
        octal: args.input.clone(),
        symbolic,
        owner: bits_to_flags(digits[0]),
        group: bits_to_flags(digits[1]),
        other: bits_to_flags(digits[2]),
    })
}

fn triplet(d: u32) -> String {
    let r = if d & 4 != 0 { 'r' } else { '-' };
    let w = if d & 2 != 0 { 'w' } else { '-' };
    let x = if d & 1 != 0 { 'x' } else { '-' };
    format!("{r}{w}{x}")
}

fn bits_to_flags(d: u32) -> ClassFlags {
    ClassFlags {
        read: d & 4 != 0,
        write: d & 2 != 0,
        execute: d & 1 != 0,
    }
}
```

- [ ] **Step 6: Wire + test + commit**

```bash
cargo test
git add src/core/error.rs src/commands/chmod/ src/commands/mod.rs src/cli.rs src/lib.rs tests/chmod.rs tests/snapshots/
git commit -m "feat(chmod): add chmod calc (symbolic→octal) and parse (octal→symbolic+flags)"
```

---

## Task 11: Update OpenCLI spec for M3 nouns

**Files:** Modify `ubertool.ocs.yaml`. Run `make gen` and `make verify-spec`.

Follow the M2 Task 16 workflow (which itself followed M1's Task 16). Add entries for each new M3 noun:

- **ipv4** group + 4 leaves: `parse`, `subnet`, `range-expand`, `to-ipv6`. Each takes a positional `input` (string, required) + `--json`/`--quiet`. `range-expand` adds `--max` (integer, default "1024").
- **mac** group + 2 leaves: `new` (flags `--json`/`--quiet` only), `lookup <input>` (required positional + standard).
- **ipv6-ula** group + 1 leaf: `new` (flags only).
- **math** group + 1 leaf: `eval <input>` (required positional + standard).
- **percentage** group + 3 leaves:
  - `of <percent> <value>` (two required numeric positionals; emit as `string` type in spec since alpha-7 doesn't distinguish numerics).
  - `change <from> <to>` (two positionals).
  - `of-total <part> <total>` (two positionals).
- **eta** group + 1 leaf: `calc` (flags `--done`/`--total`/`--elapsed`, each string-type required).
- **date** group + 1 leaf: `convert <input>` + `--from` (choices: unix, iso8601, rfc2822) + `--tz` (optional string).
- **crontab** group + 2 leaves:
  - `describe <input>` (positional required).
  - `next <input>` + `--count` (integer, default "5").
- **chmod** group + 2 leaves: `calc <input>`, `parse <input>` (positional required + standard).

Standard flags on every leaf (unless noted): `--in` (string), `--json` (boolean), `--quiet` (boolean, `aliases: [q]`).

**Spec format gotchas** (still apply from M1/M2):
- `choices:` is a list of `{value: "..."}` maps.
- Integer `default` values must be quoted strings.
- `summary:` values containing colons need quoting.

Run:

```bash
ocli spec check ubertool.ocs.yaml
make gen
make verify-spec
git add ubertool.ocs.yaml docs/cli/ docs/llms.txt
git commit -m "docs(spec): add all M3 nouns to OpenCLI spec and regenerate Markdown + llms.txt"
```

Expected: `make verify-spec` reports ≥ 78 leaf commands (M2 ended at 62; M3 adds ~17 leaves).

---

## Task 12: Extend fitness checklist + final `make ci`

**Files:** Modify `tests/agent_cli_fitness.rs`. Run `make ci`.

- [ ] **Step 1: Extend `ALL_NOUNS_AND_VERBS`**

Append the M3 entries:

```rust
    // M3
    ("ipv4", &["parse", "subnet", "range-expand", "to-ipv6"]),
    ("mac", &["new", "lookup"]),
    ("ipv6-ula", &["new"]),
    ("math", &["eval"]),
    ("percentage", &["of", "change", "of-total"]),
    ("eta", &["calc"]),
    ("date", &["convert"]),
    ("crontab", &["describe", "next"]),
    ("chmod", &["calc", "parse"]),
```

- [ ] **Step 2: Run tests**

```bash
cargo test --test agent_cli_fitness
cargo test
make ci
```

Fix clippy warnings if any. Common candidates on M3 code:
- `clippy::large_enum_variant` on `Noun` — if it fires now (with 28→37 variants), add `#[allow(clippy::large_enum_variant)]` on the enum.
- Unused imports in test files.

- [ ] **Step 3: Commit**

```bash
git add tests/agent_cli_fitness.rs
git commit -m "test(fitness): extend fitness checklist to all M3 nouns and verbs"
```

If fmt applied changes:

```bash
git add -u
git commit -m "style: apply cargo fmt across M3 source and test files"
```

- [ ] **Step 4: Confirm clean state**

```bash
git status   # clean
git log --oneline origin/main..HEAD | head -20
```

Expected: ~13-14 commits ahead of origin/main (the M3 work).

**Do NOT push** unless explicitly told to. **Do NOT create a git tag.**

---

## Self-Review

**Spec coverage** — every requirement in M3 scope per design spec §16:

- ✅ ipv4 (parse, subnet, range-expand, to-ipv6) — Task 2
- ✅ mac (new, lookup) — Task 3
- ✅ ipv6-ula (new) — Task 4
- ✅ math (eval) — Task 5
- ✅ percentage (of, change, of-total) — Task 6
- ✅ eta (calc) — Task 7
- ✅ date (convert) — Task 8
- ✅ crontab (describe, next) — Task 9
- ✅ chmod (calc, parse) — Task 10
- ✅ OpenCLI spec updated — Task 11
- ✅ Fitness extended + CI green — Task 12

**Placeholder scan** — no "TBD"/"TODO" or vague steps. Each task has complete code. The `oui.csv` is a curated sample for M3 (documented as such); full IEEE registry is deferred.

**Type consistency**:
- Every command emits a `#[derive(Serialize)] struct`.
- New ErrorCode variants in M3: `InvalidIp` (T2), `InvalidMac` (T3), `InvalidMath` (T5, also used by T6/T7), `InvalidDate` (T8), `InvalidCron` (T9), `InvalidChmod` (T10). All map to exit 3.
- Numeric commands (math, percentage, eta) switch on `out.mode` to emit bare value in text mode, struct in JSON mode — same pattern as `list` and `temperature`.

**Scope check** — 9 nouns + 1 spec + 1 CI = 11 tasks across 12 numbered. Single milestone.

**Ambiguity check** — decisions documented inline:
- IPv4 range-expand has a `--max` cap (default 1024) to avoid OOM on /8s.
- MAC lookup uses a small bundled OUI subset for M3; full IEEE registry is M3+.
- IPv6-ULA emits `fd` prefix only (locally assigned); randomly-assigned `fc` is RFC-reserved.
- Math eval returns int when input is int-only, float otherwise (preserves type from evalexpr).
- Percentage change errors when `from == 0` (undefined).
- Date convert: `--tz` adds a `local` field when set; otherwise omitted from output.
- Crontab `next` accepts the common 5-field form and prepends `0` for seconds before passing to the `cron` crate.

---

## Execution Handoff

Plan saved to `/Users/mihai/dev/ubertool/docs/superpowers/plans/2026-05-15-ubertool-m3-network-calc.md`. Same workflow as M1/M2 — subagent-driven execution.
