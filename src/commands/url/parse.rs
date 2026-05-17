use clap::Args;
use serde::Serialize;
use url::Url;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    scheme: String,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

pub fn run(args: ParseArgs, out: &Out) -> Result<(), CliError> {
    let parsed = Url::parse(&args.input).map_err(|e| {
        CliError::new(ErrorCode::InvalidUrl, format!("invalid URL: {e}"))
            .with_input(serde_json::json!(args.input))
    })?;
    let username = if parsed.username().is_empty() {
        None
    } else {
        Some(parsed.username().to_string())
    };
    let password = parsed.password().map(|p| p.to_string());
    out.emit_value(&Out0 {
        scheme: parsed.scheme().to_string(),
        username,
        password,
        host: parsed.host_str().map(|h| h.to_string()),
        port: parsed.port(),
        path: parsed.path().to_string(),
        query: parsed.query().map(|q| q.to_string()),
        fragment: parsed.fragment().map(|f| f.to_string()),
    })
}
