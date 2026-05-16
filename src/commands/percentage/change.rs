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
        out.emit_value(&Out0 { from: args.from, to: args.to, percentage_change: change })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{change}").map_err(CliError::from)
    }
}
