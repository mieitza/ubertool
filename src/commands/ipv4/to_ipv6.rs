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
