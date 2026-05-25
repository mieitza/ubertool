//! Emit the full ubertool command surface as JSON (agent introspection).

use clap::Args;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

/// The OpenCLI spec, bundled at compile time. `make verify-spec` guarantees in
/// CI that this matches the binary's actual command surface.
const SPEC_YAML: &str = include_str!("../../../ubertool.ocs.yaml");

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Emit only the commands belonging to this noun (e.g. `hash`). Omit for the full surface.
    #[arg(long)]
    pub noun: Option<String>,
}

pub fn run(args: SchemaArgs, _out: &Out) -> Result<(), CliError> {
    let mut spec: serde_json::Value = serde_yaml::from_str(SPEC_YAML).map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("bundled spec is not valid YAML: {e}"),
        )
    })?;

    if let Some(noun) = &args.noun {
        filter_to_noun(&mut spec, noun)?;
    }

    let json = serde_json::to_string_pretty(&spec).map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("schema serialization failed: {e}"),
        )
    })?;
    println!("{json}");
    Ok(())
}

fn filter_to_noun(spec: &mut serde_json::Value, noun: &str) -> Result<(), CliError> {
    let commands = spec
        .get_mut("commands")
        .and_then(|c| c.as_object_mut())
        .ok_or_else(|| CliError::new(ErrorCode::Internal, "spec has no commands map"))?;

    // Keep entries whose command signature contains the noun as a path segment.
    // Spec keys look like: "ubertool hash {command} [flags]" or
    // "ubertool hash sha256 [input] [flags]".
    let prefix = format!("ubertool {noun} ");
    let kept: serde_json::Map<String, serde_json::Value> = commands
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if kept.is_empty() {
        return Err(CliError::new(
            ErrorCode::UsageError,
            format!("no commands found for noun '{noun}'"),
        )
        .with_hint("run `ubertool schema` (no --noun) to see all nouns"));
    }
    *commands = kept;
    Ok(())
}
