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
