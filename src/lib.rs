pub mod cli;
pub mod commands;
pub mod core;

use std::process::ExitCode as ProcExitCode;

use clap::Parser;

use crate::core::error::CliError;
use crate::core::output::Out;

pub fn run() -> ProcExitCode {
    let parsed = cli::Cli::parse();
    let out = Out::new(parsed.output_mode(), core::tty::is_stdout_tty());

    let result: Result<(), CliError> = match parsed.noun {
        cli::Noun::Base64(a) => commands::base64::dispatch(a, &out),
        cli::Noun::BasicAuth(a) => commands::basic_auth::dispatch(a, &out),
        cli::Noun::Bcrypt(a) => commands::bcrypt::dispatch(a, &out),
        cli::Noun::Hash(a) => commands::hash::dispatch(a, &out),
        cli::Noun::Html(a) => commands::html::dispatch(a, &out),
        cli::Noun::Hmac(a) => commands::hmac::dispatch(a, &out),
        cli::Noun::Json(a) => commands::json::dispatch(a, &out),
        cli::Noun::Jwt(a) => commands::jwt::dispatch(a, &out),
        cli::Noun::Password(a) => commands::password::dispatch(a, &out),
        cli::Noun::Token(a) => commands::token::dispatch(a, &out),
        cli::Noun::Ulid(a) => commands::ulid::dispatch(a, &out),
        cli::Noun::Url(a) => commands::url::dispatch(a, &out),
        cli::Noun::Uuid(a) => commands::uuid::dispatch(a, &out),
    };

    match result {
        Ok(()) => ProcExitCode::SUCCESS,
        Err(err) => {
            out.emit_error(&err);
            ProcExitCode::from(err.exit().as_u8())
        }
    }
}
