use std::path::PathBuf;

use clap::Args;

use crate::core::error::CliError;
use crate::core::output::Out;

use super::{render, Format};

#[derive(Debug, Args)]
pub struct GenerateArgs {
    pub input: String,
    #[arg(long, value_enum, default_value = "svg")]
    pub format: Format,
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,
}

pub fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    render(&args.input, args.format, args.out_path.as_ref(), out)
}
