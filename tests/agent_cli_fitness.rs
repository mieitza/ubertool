//! Programmatic enforcement of the design-spec §1 rules — `agent-cli-design`
//! fitness checklist. As new tools land, extend the lists below.

use assert_cmd::Command;
use predicates::prelude::*;

fn ubertool() -> Command {
    Command::cargo_bin("ubertool").expect("binary built")
}

/// Every data-returning command surfaces `--json` in `--help`.
#[test]
fn json_flag_documented_on_every_data_command() {
    let commands: &[&[&str]] = &[
        &["base64", "encode", "--help"],
        &["base64", "decode", "--help"],
    ];
    for cmd in commands {
        let stdout = ubertool()
            .args(*cmd)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let s = String::from_utf8(stdout).unwrap();
        assert!(s.contains("--json"), "help for {:?} is missing --json", cmd);
    }
}

/// The top-level `--help` lists every documented exit code (0..=6).
#[test]
fn top_level_help_documents_exit_codes() {
    let stdout = ubertool()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(stdout).unwrap();
    for code in ["0", "1", "2", "3", "4", "5", "6"] {
        assert!(
            s.contains(code),
            "top-level --help missing exit code {code}"
        );
    }
    assert!(s.contains("Exit codes:"));
}

/// Documented error paths must NOT exit 0.
#[test]
fn documented_errors_never_exit_zero() {
    ubertool()
        .args(["base64", "decode", "!!!"])
        .assert()
        .failure();
}

/// In `--json` mode, stdout is pure JSON on both success AND error paths.
#[test]
fn json_mode_stdout_purity_success_and_error() {
    // Success path.
    let stdout = ubertool()
        .args(["base64", "encode", "hello", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let _: serde_json::Value =
        serde_json::from_slice(&stdout).expect("success path stdout must be JSON");

    // Error path.
    let stdout = ubertool()
        .args(["base64", "decode", "!!!", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_slice(&stdout).expect("error path stdout must be JSON");
    assert_eq!(v["error"], "invalid_base64");
}

/// Typed error codes echo the failing input (Rule 7).
#[test]
fn errors_echo_failing_input() {
    let stdout = ubertool()
        .args(["base64", "decode", "!!!", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(v["input"], "!!!");
}

/// Errors include a `hint` field where useful (Rule 7 sub-rule).
#[test]
fn errors_include_hint_field() {
    let stdout = ubertool()
        .args(["base64", "decode", "!!!", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert!(v["hint"].is_string(), "expected hint field on this error");
}

/// Stderr in text mode echoes `error: <code>: <message>` (Rule 7).
#[test]
fn stderr_text_envelope_format() {
    ubertool()
        .args(["base64", "decode", "!!!"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: invalid_base64:"));
}

/// In `--json` mode, errors STILL emit a human message to stderr (per
/// design-spec §7). stderr must not be silent — agents and humans both
/// want a glanceable failure description.
#[test]
fn json_mode_error_path_writes_to_stderr_too() {
    let output = ubertool()
        .args(["base64", "decode", "!!!", "--json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("error: invalid_base64"),
        "stderr must contain the human error envelope even in --json mode, got: {stderr:?}"
    );
}

const ALL_NOUNS_AND_VERBS: &[(&str, &[&str])] = &[
    // M0/M1
    ("base64", &["encode", "decode"]),
    (
        "hash",
        &[
            "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-256", "sha3-512",
        ],
    ),
    (
        "hmac",
        &[
            "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-256", "sha3-512",
        ],
    ),
    ("bcrypt", &["hash", "verify"]),
    ("url", &["encode", "decode", "parse"]),
    ("html", &["encode", "decode"]),
    ("basic-auth", &["encode", "decode"]),
    ("jwt", &["decode", "verify"]),
    ("uuid", &["new"]),
    ("ulid", &["new"]),
    ("token", &["new"]),
    ("password", &["score"]),
    ("json", &["to-yaml", "to-toml", "minify", "prettify"]),
    ("yaml", &["to-json", "to-toml"]),
    ("toml", &["to-json", "to-yaml"]),
    // M2
    ("xml", &["to-json", "format"]),
    ("csv", &["to-json"]),
    ("case", &["convert"]),
    ("slugify", &["generate"]),
    ("list", &["convert"]),
    ("integer-base", &["convert"]),
    ("roman", &["to-num", "from-num"]),
    ("temperature", &["convert"]),
    ("sql", &["format"]),
    ("markdown", &["to-html"]),
    (
        "text",
        &[
            "to-binary",
            "from-binary",
            "to-unicode",
            "from-unicode",
            "to-nato",
            "stats",
            "diff",
            "obfuscate",
        ],
    ),
    ("docker-run", &["to-compose"]),
    ("safelink", &["decode"]),
    // M3
    ("ipv4", &["parse", "subnet", "range-expand", "to-ipv6"]),
    ("mac", &["new", "lookup"]),
    ("ipv6-ula", &["new"]),
    ("math", &["eval"]),
    ("percentage", &["of", "change", "of-total"]),
    ("eta", &["calc"]),
    ("date", &["convert"]),
    ("crontab", &["describe", "next"]),
    ("chmod", &["calc", "parse"]),
    // M4
    ("cipher", &["encrypt", "decrypt"]),
    ("rsa", &["keypair"]),
    ("otp", &["generate", "validate"]),
    ("pdf", &["signature"]),
    ("regex", &["test", "generate"]),
    ("email", &["normalize"]),
    ("iban", &["validate"]),
    ("phone", &["parse"]),
    ("user-agent", &["parse"]),
];

#[test]
fn every_noun_and_verb_has_discoverable_help() {
    for (noun, verbs) in ALL_NOUNS_AND_VERBS {
        // noun-level help
        Command::cargo_bin("ubertool")
            .unwrap()
            .args([noun, "--help"])
            .assert()
            .success();
        for verb in *verbs {
            Command::cargo_bin("ubertool")
                .unwrap()
                .args([noun, verb, "--help"])
                .assert()
                .success();
        }
    }
}

#[test]
fn data_returning_commands_advertise_json() {
    // The global --json flag is documented at the top level.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}
