use assert_cmd::Command;
use predicates::prelude::*;

fn ubertool() -> Command {
    Command::cargo_bin("ubertool").expect("binary built")
}

#[test]
fn encode_positional_text_mode() {
    ubertool()
        .args(["base64", "encode", "hello"])
        .assert()
        .success()
        .stdout("aGVsbG8=\n");
}

#[test]
fn encode_positional_json_mode_is_pure_json() {
    let assert = ubertool()
        .args(["base64", "encode", "hello", "--json"])
        .assert()
        .success();
    let stdout = assert.get_output().stdout.clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&stdout).expect("stdout must be valid JSON in --json mode");
    assert_eq!(parsed["encoded"], "aGVsbG8=");
}

#[test]
fn encode_stdin_pipe_works() {
    ubertool()
        .args(["base64", "encode"])
        .write_stdin("hello")
        .assert()
        .success()
        .stdout("aGVsbG8=\n");
}

#[test]
fn encode_from_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"hello").unwrap();
    ubertool()
        .args(["base64", "encode", "--in"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("aGVsbG8=\n");
}

#[test]
fn encode_to_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    ubertool()
        .args(["base64", "encode", "hello", "--out"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("");
    let written = std::fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(written, "aGVsbG8=");
}

#[test]
fn decode_positional_text_mode() {
    ubertool()
        .args(["base64", "decode", "aGVsbG8="])
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn decode_positional_json_mode_is_pure_json() {
    let assert = ubertool()
        .args(["base64", "decode", "aGVsbG8=", "--json"])
        .assert()
        .success();
    let stdout = assert.get_output().stdout.clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&stdout).expect("stdout must be valid JSON in --json mode");
    assert_eq!(parsed["decoded"], "hello");
}

#[test]
fn decode_invalid_input_exits_3_text_mode() {
    ubertool()
        .args(["base64", "decode", "!!!"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("error: invalid_base64"))
        .stderr(predicate::str::contains("hint:"))
        .stdout(predicate::str::is_empty());
}

#[test]
fn decode_invalid_input_exits_3_json_mode_with_pure_json_stdout() {
    let assert = ubertool()
        .args(["base64", "decode", "!!!", "--json"])
        .assert()
        .code(3);
    let stdout = assert.get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&stdout)
        .expect("stdout must be valid JSON in --json mode on errors too");
    assert_eq!(parsed["error"], "invalid_base64");
    assert_eq!(parsed["retriable"], false);
    assert!(parsed["hint"].is_string());
    assert_eq!(parsed["input"], "!!!");
}

#[test]
fn decode_into_file_round_trip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    ubertool()
        .args(["base64", "decode", "aGVsbG8=", "--out"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("");
    let written = std::fs::read(tmp.path()).unwrap();
    assert_eq!(written, b"hello");
}
