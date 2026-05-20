use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Serialize)]
struct VersionOutput {
    version: &'static str,
}

pub fn run(out: &Out) -> Result<(), CliError> {
    out.emit_value(&VersionOutput {
        version: env!("CARGO_PKG_VERSION"),
    })
}
