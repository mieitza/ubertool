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
    /// Email address normalization and validation.
    Email(crate::commands::email::EmailArgs),
    /// IBAN validation (mod-97 check digits).
    Iban(crate::commands::iban::IbanArgs),
    /// Phone number parser (E.164 and regional formats).
    Phone(crate::commands::phone::PhoneArgs),
    /// User-agent string parser.
    #[command(name = "user-agent")]
    UserAgent(crate::commands::user_agent::UserAgentArgs),
    /// HTTP Basic auth header encoding/decoding.
    #[command(name = "basic-auth")]
    BasicAuth(crate::commands::basic_auth::BasicAuthArgs),
    /// Bcrypt password hashing and verification.
    Bcrypt(crate::commands::bcrypt::BcryptArgs),
    /// String case conversion (snake/kebab/camel/pascal/etc.).
    Case(crate::commands::case::CaseArgs),
    /// Authenticated encryption and decryption (AES-256-GCM / ChaCha20-Poly1305 + Argon2id / PBKDF2).
    Cipher(crate::commands::cipher::CipherArgs),
    /// CSV to JSON conversion.
    Csv(crate::commands::csv::CsvArgs),
    /// Cryptographic hash functions (md5, sha1, sha224, sha256, sha384, sha512, sha3-256, sha3-512).
    Hash(crate::commands::hash::HashArgs),
    /// HTML entity encoding and decoding.
    Html(crate::commands::html::HtmlArgs),
    /// Keyed-hash MAC (md5/sha1/sha224/sha256/sha384/sha512/sha3-256/sha3-512).
    Hmac(crate::commands::hmac::HmacArgs),
    /// JSON conversion: to-yaml / to-toml / minify / prettify.
    Json(crate::commands::json::JsonArgs),
    /// Markdown to HTML conversion (CommonMark).
    Markdown(crate::commands::markdown::MarkdownArgs),
    /// Time-based One-Time Password generation and validation (TOTP, RFC 6238).
    Otp(crate::commands::otp::OtpArgs),
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
    /// Integer base conversion (bases 2-36).
    #[command(name = "integer-base")]
    IntegerBase(crate::commands::integer_base::IntegerBaseArgs),
    /// IPv4 utilities (parse, subnet, range-expand, to-ipv6).
    Ipv4(crate::commands::ipv4::Ipv4Args),
    /// List separator conversion (comma/newline/space/tab/semicolon/pipe).
    List(crate::commands::list::ListArgs),
    /// Regular expression utilities (test / generate).
    Regex(crate::commands::regex::RegexArgs),
    /// Roman numeral conversion (to-num / from-num, range 1-3999).
    Roman(crate::commands::roman::RomanArgs),
    /// RSA keypair generation (PKCS#8 PEM, 2048/3072/4096 bits).
    Rsa(crate::commands::rsa::RsaArgs),
    /// URL slug generation.
    Slugify(crate::commands::slugify::SlugifyArgs),
    /// SQL formatter (pretty-print with indentation and keyword casing).
    Sql(crate::commands::sql::SqlArgs),
    /// Temperature unit conversion (celsius / fahrenheit / kelvin).
    Temperature(crate::commands::temperature::TemperatureArgs),
    /// Text encoding and analysis (to-binary, from-binary, to-unicode, from-unicode, to-nato, stats, diff, obfuscate).
    Text(crate::commands::text::TextArgs),
    /// Convert `docker run` command to docker-compose YAML.
    #[command(name = "docker-run")]
    DockerRun(crate::commands::docker_run::DockerRunArgs),
    /// Unwrap Outlook/Google/generic URL wrappers.
    Safelink(crate::commands::safelink::SafelinkArgs),
    /// MAC address utilities (generate random, look up vendor OUI).
    Mac(crate::commands::mac::MacArgs),
    /// RFC4193 Unique Local IPv6 prefix generator.
    #[command(name = "ipv6-ula")]
    Ipv6Ula(crate::commands::ipv6_ula::Ipv6UlaArgs),
    /// Math expression evaluator (evalexpr).
    Math(crate::commands::math::MathArgs),
    /// PDF utilities (signature info extraction).
    Pdf(crate::commands::pdf::PdfArgs),
    /// Percentage calculations (of / change / of-total).
    Percentage(crate::commands::percentage::PercentageArgs),
    /// ETA calculator from progress (done / total / elapsed).
    Eta(crate::commands::eta::EtaArgs),
    /// Date/time format conversion (unix / ISO 8601 / RFC 2822).
    Date(crate::commands::date::DateArgs),
    /// Crontab expression utilities (describe, next).
    Crontab(crate::commands::crontab::CrontabArgs),
    /// Unix file mode conversion (symbolic ↔ octal).
    Chmod(crate::commands::chmod::ChmodArgs),
    /// MIME type lookup from file extension or filename.
    Mime(crate::commands::mime::MimeArgs),
    /// HTTP status code lookup.
    #[command(name = "http-status")]
    HttpStatus(crate::commands::http_status::HttpStatusArgs),
    /// QR code generation (SVG/PNG) including WiFi URIs.
    Qr(crate::commands::qr::QrArgs),
    /// SVG placeholder image generator.
    #[command(name = "svg-placeholder")]
    SvgPlaceholder(crate::commands::svg_placeholder::SvgPlaceholderArgs),
    /// ASCII art rendering (figlet standard font).
    Ascii(crate::commands::ascii::AsciiArgs),
}
