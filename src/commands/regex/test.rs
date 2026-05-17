use regex::Regex;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::TestArgs;

#[derive(Serialize)]
struct Out0 {
    matched: bool,
    groups: Vec<String>,
}

pub fn run(args: TestArgs, out: &Out) -> Result<(), CliError> {
    let re = Regex::new(&args.pattern).map_err(|e| {
        CliError::new(ErrorCode::InvalidRegex, format!("invalid regex: {e}"))
            .with_input(serde_json::json!({"pattern": args.pattern}))
            .with_hint("Rust's regex crate does not support lookaround or backreferences")
    })?;
    if let Some(caps) = re.captures(&args.text) {
        let groups: Vec<String> = caps
            .iter()
            .map(|m| m.map(|x| x.as_str().to_string()).unwrap_or_default())
            .collect();
        out.emit_value(&Out0 {
            matched: true,
            groups,
        })
    } else {
        out.emit_value(&Out0 {
            matched: false,
            groups: Vec::new(),
        })
    }
}
