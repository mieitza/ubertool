use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn toml_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-json", "name = \"alice\"\nage = 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert_eq!(v["name"], "alice");
    assert_eq!(v["age"], 30);
}

#[test]
fn toml_to_yaml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-yaml", "name = \"alice\"\nage = 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // serde_yaml may quote strings; check name and alice separately
    assert!(s.contains("name"));
    assert!(s.contains("alice"));
    assert!(s.contains("age"));
    assert!(s.contains("30"));
}

#[test]
fn toml_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["toml", "to-json", "[invalid][[duplicate"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_toml"));
}
