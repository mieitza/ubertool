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
    bytes[0] = (bytes[0] | 0x02) & 0xfe;
    let s = bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    out.emit_value(&Out0 { mac: s })
}
