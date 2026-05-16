//! Temperature unit conversion.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct TemperatureArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Convert between temperature units (celsius/fahrenheit/kelvin).
    #[command(
        long_about = "Convert between temperature units.\n\nExamples:\n  ubertool temperature convert 100 --from celsius --to fahrenheit\n  ubertool temperature convert 32 --from fahrenheit --to kelvin --json"
    )]
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Unit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: f64,
    #[arg(long, value_enum)]
    pub from: Unit,
    #[arg(long, value_enum)]
    pub to: Unit,
}

#[derive(Serialize)]
struct Out0 {
    value: f64,
    unit: &'static str,
}

pub fn dispatch(args: TemperatureArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Convert(a) => run(a, out),
    }
}

fn run(args: ConvertArgs, out: &Out) -> Result<(), CliError> {
    let c = match args.from {
        Unit::Celsius => args.input,
        Unit::Fahrenheit => (args.input - 32.0) * 5.0 / 9.0,
        Unit::Kelvin => args.input - 273.15,
    };
    let (value, unit) = match args.to {
        Unit::Celsius => (c, "celsius"),
        Unit::Fahrenheit => (c * 9.0 / 5.0 + 32.0, "fahrenheit"),
        Unit::Kelvin => (c + 273.15, "kelvin"),
    };
    if out.mode == OutputMode::Json {
        out.emit_value(&Out0 { value, unit })
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{value}").map_err(CliError::from)
    }
}
