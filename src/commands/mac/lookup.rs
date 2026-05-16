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
