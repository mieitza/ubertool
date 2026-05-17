use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn phone_parse_us() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "phone", "parse", "+14155552671"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["country_code"], 1);
    assert_eq!(v["e164"], "+14155552671");
}

#[test]
fn phone_parse_with_default_region() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "phone", "parse", "415 555 2671", "--region", "US"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["e164"], "+14155552671");
}

#[test]
fn phone_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["phone", "parse", "not-a-phone"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_phone"));
}
