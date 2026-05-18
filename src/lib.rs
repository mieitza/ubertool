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
        cli::Noun::Email(a) => commands::email::dispatch(a, &out),
        cli::Noun::Iban(a) => commands::iban::dispatch(a, &out),
        cli::Noun::Phone(a) => commands::phone::dispatch(a, &out),
        cli::Noun::UserAgent(a) => commands::user_agent::dispatch(a, &out),
        cli::Noun::BasicAuth(a) => commands::basic_auth::dispatch(a, &out),
        cli::Noun::Bcrypt(a) => commands::bcrypt::dispatch(a, &out),
        cli::Noun::Case(a) => commands::case::dispatch(a, &out),
        cli::Noun::Cipher(a) => commands::cipher::dispatch(a, &out),
        cli::Noun::Csv(a) => commands::csv::dispatch(a, &out),
        cli::Noun::Hash(a) => commands::hash::dispatch(a, &out),
        cli::Noun::Html(a) => commands::html::dispatch(a, &out),
        cli::Noun::Hmac(a) => commands::hmac::dispatch(a, &out),
        cli::Noun::Json(a) => commands::json::dispatch(a, &out),
        cli::Noun::Markdown(a) => commands::markdown::dispatch(a, &out),
        cli::Noun::Otp(a) => commands::otp::dispatch(a, &out),
        cli::Noun::Jwt(a) => commands::jwt::dispatch(a, &out),
        cli::Noun::Password(a) => commands::password::dispatch(a, &out),
        cli::Noun::Token(a) => commands::token::dispatch(a, &out),
        cli::Noun::Toml(a) => commands::toml::dispatch(a, &out),
        cli::Noun::Ulid(a) => commands::ulid::dispatch(a, &out),
        cli::Noun::Url(a) => commands::url::dispatch(a, &out),
        cli::Noun::Uuid(a) => commands::uuid::dispatch(a, &out),
        cli::Noun::Yaml(a) => commands::yaml::dispatch(a, &out),
        cli::Noun::Xml(a) => commands::xml::dispatch(a, &out),
        cli::Noun::IntegerBase(a) => commands::integer_base::dispatch(a, &out),
        cli::Noun::Ipv4(a) => commands::ipv4::dispatch(a, &out),
        cli::Noun::List(a) => commands::list::dispatch(a, &out),
        cli::Noun::Regex(a) => commands::regex::dispatch(a, &out),
        cli::Noun::Roman(a) => commands::roman::dispatch(a, &out),
        cli::Noun::Rsa(a) => commands::rsa::dispatch(a, &out),
        cli::Noun::Slugify(a) => commands::slugify::dispatch(a, &out),
        cli::Noun::Sql(a) => commands::sql::dispatch(a, &out),
        cli::Noun::Temperature(a) => commands::temperature::dispatch(a, &out),
        cli::Noun::Text(a) => commands::text::dispatch(a, &out),
        cli::Noun::DockerRun(a) => commands::docker_run::dispatch(a, &out),
        cli::Noun::Safelink(a) => commands::safelink::dispatch(a, &out),
        cli::Noun::Mac(a) => commands::mac::dispatch(a, &out),
        cli::Noun::Ipv6Ula(a) => commands::ipv6_ula::dispatch(a, &out),
        cli::Noun::Math(a) => commands::math::dispatch(a, &out),
        cli::Noun::Pdf(a) => commands::pdf::dispatch(a, &out),
        cli::Noun::Percentage(a) => commands::percentage::dispatch(a, &out),
        cli::Noun::Eta(a) => commands::eta::dispatch(a, &out),
        cli::Noun::Date(a) => commands::date::dispatch(a, &out),
        cli::Noun::Crontab(a) => commands::crontab::dispatch(a, &out),
        cli::Noun::Chmod(a) => commands::chmod::dispatch(a, &out),
        cli::Noun::Mime(a) => commands::mime::dispatch(a, &out),
        cli::Noun::HttpStatus(a) => commands::http_status::dispatch(a, &out),
        cli::Noun::Qr(a) => commands::qr::dispatch(a, &out),
        cli::Noun::SvgPlaceholder(a) => commands::svg_placeholder::dispatch(a, &out),
        cli::Noun::Ascii(a) => commands::ascii::dispatch(a, &out),
        cli::Noun::Git(a) => commands::git::dispatch(a, &out),
        cli::Noun::Numeronym(a) => commands::numeronym::dispatch(a, &out),
        cli::Noun::Port(a) => commands::port::dispatch(a, &out),
    };

    match result {
        Ok(()) => ProcExitCode::SUCCESS,
        Err(err) => {
            out.emit_error(&err);
            ProcExitCode::from(err.exit().as_u8())
        }
    }
}
