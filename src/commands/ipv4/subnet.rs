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
