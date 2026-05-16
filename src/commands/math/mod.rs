//! Math expression evaluator (evalexpr).

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct MathArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Evaluate a math expression.
    #[command(
        long_about = "Evaluate a math expression. Supports +, -, *, /, %, ^, parens, common functions (sqrt, sin, cos, etc.).\n\nExamples:\n  ubertool math eval '(2 + 3) * 4'\n  ubertool math eval '3.14 * 2' --json\n\nExit codes:\n  3   invalid math expression (invalid_math)"
    )]
    Eval(EvalArgs),
}

#[derive(Debug, Args)]
pub struct EvalArgs {
    pub input: String,
}

#[derive(Serialize)]
struct Out0 {
    result: serde_json::Value,
}

pub fn dispatch(args: MathArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Eval(a) => run(a, out),
    }
}

fn run(args: EvalArgs, out: &Out) -> Result<(), CliError> {
    let v = evalexpr::eval(&args.input).map_err(|e| {
        CliError::new(
            ErrorCode::InvalidMath,
            format!("invalid math expression: {e}"),
        )
        .with_input(serde_json::json!(args.input))
    })?;
    let json_val = match v {
        evalexpr::Value::Int(i) => serde_json::json!(i),
        evalexpr::Value::Float(f) => serde_json::json!(f),
        evalexpr::Value::Boolean(b) => serde_json::json!(b),
        evalexpr::Value::String(s) => serde_json::json!(s),
        evalexpr::Value::Empty => serde_json::Value::Null,
        evalexpr::Value::Tuple(_) => {
            return Err(CliError::new(
                ErrorCode::InvalidMath,
                "expression evaluated to a tuple — not supported",
            ));
        }
    };
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { result: json_val })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        match &json_val {
            serde_json::Value::String(s) => writeln!(stdout, "{s}").map_err(CliError::from),
            other => writeln!(stdout, "{other}").map_err(CliError::from),
        }
    }
}
