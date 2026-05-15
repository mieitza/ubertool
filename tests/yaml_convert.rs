use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn yaml_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "name: alice\nage: 30\n"])
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
fn yaml_to_json_pretty() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "a: 1\nb: 2\n", "--pretty"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("  \"a\": 1"));
}

#[test]
fn yaml_to_toml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-toml", "name: alice\nage: 30\n"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name = \"alice\""));
    assert!(s.contains("age = 30"));
}

#[test]
fn yaml_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["yaml", "to-json", "key: [unclosed"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_yaml"));
}
