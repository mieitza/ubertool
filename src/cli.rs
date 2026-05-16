//! Top-level CLI definition. Subcommands (nouns) are added in `Noun`.

use clap::{Parser, Subcommand};

use crate::core::output::OutputMode;

#[derive(Parser, Debug)]
#[command(
    name = "ubertool",
    about = "Developer-focused data-transform utilities — agent-friendly.",
    long_about = "ubertool — agent-friendly developer utilities.\n\
                  \n\
                  Output:\n  \
                  --json   structured JSON on stdout (nothing else lands there)\n  \
                  --quiet  bare values, pipe-friendly\n  \
                  default  human-readable key: value lines\n\
                  \n\
                  Exit codes:\n  \
                  0  success\n  \
                  1  general failure\n  \
                  2  usage error\n  \
                  3  input validation error\n  \
                  4  i/o error\n  \
                  5  cryptographic / integrity failure\n  \
                  6  feature not built in this binary\n",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Output as JSON to stdout. Human messages still go to stderr.
    #[arg(long, global = true)]
    pub json: bool,

    /// Bare values only (pipe-friendly).
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Verbose progress to stderr.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub noun: Noun,
}

impl Cli {
    pub fn output_mode(&self) -> OutputMode {
        if self.json {
            OutputMode::Json
        } else if self.quiet {
            OutputMode::Quiet
        } else {
            OutputMode::Text
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Noun {
    /// Base64 encoding and decoding.
    Base64(crate::commands::base64::Base64Args),
    /// HTTP Basic auth header encoding/decoding.
    #[command(name = "basic-auth")]
    BasicAuth(crate::commands::basic_auth::BasicAuthArgs),
    /// Bcrypt password hashing and verification.
    Bcrypt(crate::commands::bcrypt::BcryptArgs),
    /// Cryptographic hash functions (md5, sha1, sha224, sha256, sha384, sha512, sha3-256, sha3-512).
    Hash(crate::commands::hash::HashArgs),
    /// HTML entity encoding and decoding.
    Html(crate::commands::html::HtmlArgs),
    /// Keyed-hash MAC (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512).
    Hmac(crate::commands::hmac::HmacArgs),
    /// JSON conversion: to-yaml / to-toml / minify / prettify.
    Json(crate::commands::json::JsonArgs),
    /// JSON Web Token decode and verify (HS* only).
    Jwt(crate::commands::jwt::JwtArgs),
    /// Password strength scoring (zxcvbn).
    Password(crate::commands::password::PasswordArgs),
    /// Random token generation (hex / base64 / alphanumeric).
    Token(crate::commands::token::TokenArgs),
    /// ULID generation.
    Ulid(crate::commands::ulid::UlidArgs),
    /// TOML conversion: to-json / to-yaml.
    Toml(crate::commands::toml::TomlArgs),
    /// URL percent-encoding and decoding.
    Url(crate::commands::url::UrlArgs),
    /// UUID generation (v4, v7).
    Uuid(crate::commands::uuid::UuidArgs),
    /// YAML conversion: to-json / to-toml.
    Yaml(crate::commands::yaml::YamlArgs),
    /// XML conversion: to-json / format.
    Xml(crate::commands::xml::XmlArgs),
}
