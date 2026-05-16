use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn date_convert_unix_to_iso() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "1700000000", "--from", "unix"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let iso = v["iso8601"].as_str().expect("iso8601 field");
    assert!(iso.starts_with("2023-11-14"));
    assert_eq!(v["unix"].as_i64(), Some(1700000000));
}

#[test]
fn date_convert_iso_to_unix() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "2023-11-14T22:13:20Z", "--from", "iso8601"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["unix"].as_i64(), Some(1700000000));
}

#[test]
fn date_convert_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["date", "convert", "not-a-date", "--from", "iso8601"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_date"));
}

#[test]
fn date_convert_with_tz() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "date", "convert", "1700000000", "--from", "unix", "--tz", "America/New_York"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let local = v["local"].as_str().expect("local field present when --tz set");
    assert!(local.contains("2023-11-14") || local.contains("2023-11-15"));
}
