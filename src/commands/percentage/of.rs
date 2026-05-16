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
        out.emit_value(&Out0 { percentage: args.percent, value: args.value, result })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{result}").map_err(CliError::from)
    }
}
