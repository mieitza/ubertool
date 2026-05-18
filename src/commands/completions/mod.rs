//! Shell completion script generator.

use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Target shell.
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs, _out: &Out) -> Result<(), CliError> {
    let mut cmd = crate::cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
