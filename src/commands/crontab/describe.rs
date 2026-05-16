use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::DescribeArgs;

#[derive(Serialize)]
struct Out0 {
    description: String,
}

pub fn run(args: DescribeArgs, out: &Out) -> Result<(), CliError> {
    let parts: Vec<&str> = args.input.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(CliError::new(
            ErrorCode::InvalidCron,
            format!("expected 5 fields, got {}", parts.len()),
        )
        .with_hint("format: minute hour day-of-month month day-of-week"));
    }
    let description = describe(&parts);
    out.emit_value(&Out0 { description })
}

fn describe(p: &[&str]) -> String {
    let (m, h, dom, mon, dow) = (p[0], p[1], p[2], p[3], p[4]);

    let all_stars = p.iter().all(|f| *f == "*");
    if all_stars {
        return "every minute".to_string();
    }
    if m != "*" && h != "*" && dom == "*" && mon == "*" && dow == "*" {
        if let (Ok(min), Ok(hour)) = (m.parse::<u32>(), h.parse::<u32>()) {
            return format!("at {hour:02}:{min:02} every day");
        }
    }
    if m == "0" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "at the top of every hour".to_string();
    }
    if let Some(stripped) = m.strip_prefix("*/") {
        if h == "*" && dom == "*" && mon == "*" && dow == "*" {
            if let Ok(step) = stripped.parse::<u32>() {
                return format!("every {step} minutes");
            }
        }
    }

    [
        field_desc("minute", m),
        field_desc("hour", h),
        field_desc("day-of-month", dom),
        field_desc("month", mon),
        field_desc("day-of-week", dow),
    ]
    .join(", ")
}

fn field_desc(name: &str, v: &str) -> String {
    if v == "*" {
        format!("every {name}")
    } else if let Some(stripped) = v.strip_prefix("*/") {
        format!("every {stripped} {name}s")
    } else {
        format!("{name} = {v}")
    }
}
