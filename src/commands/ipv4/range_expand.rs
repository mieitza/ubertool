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
