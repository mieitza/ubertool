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
    let binary = octets.iter().map(|b| format!("{b:08b}")).collect::<Vec<_>>().join(".");
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
