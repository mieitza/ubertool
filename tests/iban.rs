use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn iban_validate_valid() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "iban", "validate", "GB82WEST12345698765432"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["valid"], true);
    assert_eq!(v["country"], "GB");
}

#[test]
fn iban_validate_invalid_check_digits_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["iban", "validate", "GB00WEST12345698765432"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_iban"));
}

#[test]
fn iban_validate_with_spaces() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["iban", "validate", "GB82 WEST 1234 5698 7654 32"])
        .assert().success();
}
