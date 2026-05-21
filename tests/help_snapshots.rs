//! Per-noun --help snapshot suite. Catches help-text drift across refactors.

use assert_cmd::Command;
use std::process::Output;

const NOUNS: &[&str] = &[
    "base64",
    "basic-auth",
    "bcrypt",
    "hash",
    "hmac",
    "html",
    "json",
    "jwt",
    "password",
    "toml",
    "token",
    "ulid",
    "url",
    "uuid",
    "yaml",
    "xml",
    "csv",
    "case",
    "slugify",
    "list",
    "integer-base",
    "roman",
    "temperature",
    "sql",
    "markdown",
    "text",
    "docker-run",
    "safelink",
    "ipv4",
    "mac",
    "ipv6-ula",
    "math",
    "percentage",
    "eta",
    "date",
    "crontab",
    "chmod",
    "cipher",
    "rsa",
    "otp",
    "pdf",
    "regex",
    "email",
    "iban",
    "phone",
    "user-agent",
    "mime",
    "http-status",
    "qr",
    "svg-placeholder",
    "ascii",
    "git",
    "numeronym",
    "port",
    "completions",
    "schema",
    "self",
];

fn run_help(noun: &str) -> String {
    let Output {
        stdout,
        status,
        stderr,
    } = Command::cargo_bin("ubertool")
        .unwrap()
        .args([noun, "--help"])
        .output()
        .unwrap();
    if !status.success() {
        panic!(
            "ubertool {noun} --help exited with {:?}: stderr={}",
            status,
            String::from_utf8_lossy(&stderr)
        );
    }
    String::from_utf8(stdout).unwrap()
}

#[test]
fn noun_help_snapshots() {
    // Force consistent wrap width for clap so snapshots don't depend on terminal.
    std::env::set_var("COLUMNS", "100");

    for noun in NOUNS {
        let out = run_help(noun);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(*noun);
        settings.bind(|| {
            insta::assert_snapshot!("noun_help", out);
        });
    }
}
