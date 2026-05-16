use std::str::FromStr;

use chrono::Utc;
use cron::Schedule;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::NextArgs;

#[derive(Serialize)]
struct Out0 {
    next: Vec<String>,
}

pub fn run(args: NextArgs, out: &Out) -> Result<(), CliError> {
    // cron crate expects 6 or 7 fields; prepend "0" if 5 fields given.
    let normalized = if args.input.split_whitespace().count() == 5 {
        format!("0 {}", args.input)
    } else {
        args.input.clone()
    };
    let schedule = Schedule::from_str(&normalized).map_err(|e| {
        CliError::new(
            ErrorCode::InvalidCron,
            format!("invalid cron expression: {e}"),
        )
        .with_input(serde_json::json!(args.input))
        .with_hint("format: minute hour day-of-month month day-of-week")
    })?;
    let now = Utc::now();
    let entries: Vec<String> = schedule
        .after(&now)
        .take(args.count)
        .map(|dt| dt.to_rfc3339())
        .collect();
    out.emit_value(&Out0 { next: entries })
}
