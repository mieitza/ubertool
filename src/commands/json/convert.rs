//! Shared helpers used by all four JSON verbs.

use crate::core::error::{CliError, ErrorCode};

pub fn parse_json(s: &str) -> Result<serde_json::Value, CliError> {
    serde_json::from_str(s).map_err(|e| {
        CliError::new(ErrorCode::InvalidJson, format!("invalid JSON: {e}"))
            .with_hint("ensure the input is well-formed JSON")
    })
}

/// Convert serde_json::Value → toml::Value. TOML lacks a null type; nulls fail
/// fast with `invalid_toml`.
pub fn json_to_toml(v: serde_json::Value) -> Result<toml::Value, CliError> {
    match v {
        serde_json::Value::Null => Err(CliError::new(
            ErrorCode::InvalidToml,
            "TOML does not support null values",
        )
        .with_hint("remove or replace null fields before converting")),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(CliError::new(
                    ErrorCode::InvalidToml,
                    "number out of range for TOML",
                ))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s)),
        serde_json::Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.into_iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(items?))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map {
                table.insert(k, json_to_toml(v)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}
