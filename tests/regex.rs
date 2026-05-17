use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn regex_test_match() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "regex",
            "test",
            "--pattern",
            r"^\d+$",
            "--text",
            "12345",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], true);
}

#[test]
fn regex_test_no_match() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "regex",
            "test",
            "--pattern",
            r"^\d+$",
            "--text",
            "abc",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], false);
}

#[test]
fn regex_test_with_groups() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "regex",
            "test",
            "--pattern",
            r"(\d+)-(\d+)",
            "--text",
            "42-17",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["matched"], true);
    let groups = v["groups"].as_array().expect("groups");
    assert_eq!(groups[0], "42-17");
    assert_eq!(groups[1], "42");
    assert_eq!(groups[2], "17");
}

#[test]
fn regex_test_invalid_pattern_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["regex", "test", "--pattern", "[unclosed", "--text", "x"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_regex"));
}

#[test]
fn regex_generate_basic() {
    // Use [0-9] instead of \d to guarantee ASCII digits (regex_generate uses
    // Unicode \d which may produce multibyte characters).
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["regex", "generate", "[0-9]{3}"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 3);
    assert!(s.chars().all(|c| c.is_ascii_digit()));
}
