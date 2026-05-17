use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn url_encode_simple() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "hello world"])
        .assert()
        .success()
        .stdout("hello%20world\n");
}

#[test]
fn url_encode_special_chars() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "a+b=c&d"])
        .assert()
        .success()
        .stdout("a%2Bb%3Dc%26d\n");
}

#[test]
fn url_decode_simple() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "hello%20world"])
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn url_decode_special_chars() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "a%2Bb%3Dc%26d"])
        .assert()
        .success()
        .stdout("a+b=c&d\n");
}

#[test]
fn url_encode_round_trip_via_pipe() {
    let encoded = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "Hello, World! ñ"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let encoded_s = String::from_utf8(encoded).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", &encoded_s])
        .assert()
        .success()
        .stdout("Hello, World! ñ\n");
}

#[test]
fn url_decode_invalid_utf8_returns_exit_3() {
    // %ff%fe is not valid UTF-8.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "decode", "%ff%fe"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn url_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "url", "encode", "hello world"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["encoded"], "hello%20world");
}

#[test]
fn url_parse_full() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "url",
            "parse",
            "https://user:pass@example.com:8080/path?q=1#frag",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["scheme"], "https");
    assert_eq!(v["host"], "example.com");
    assert_eq!(v["port"], 8080);
    assert_eq!(v["path"], "/path");
    assert_eq!(v["query"], "q=1");
    assert_eq!(v["fragment"], "frag");
    assert_eq!(v["username"], "user");
    assert_eq!(v["password"], "pass");
}

#[test]
fn url_parse_minimal() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "url", "parse", "https://example.com"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["scheme"], "https");
    assert_eq!(v["host"], "example.com");
    assert!(v["port"].is_null());
}

#[test]
fn url_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "parse", "not a url"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_url"));
}
