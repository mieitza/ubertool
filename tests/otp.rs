use assert_cmd::Command;
use predicates::prelude::*;

const SECRET: &str = "JBSWY3DPEHPK3PXP";

#[test]
fn otp_generate_emits_6_digit_code() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "generate", "--secret", SECRET])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 6);
    assert!(s.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn otp_generate_validate_round_trip() {
    let code_out = Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "generate", "--secret", SECRET])
        .assert().success().get_output().stdout.clone();
    let code = String::from_utf8(code_out).unwrap().trim().to_string();
    Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "validate", &code, "--secret", SECRET])
        .assert().success();
}

#[test]
fn otp_validate_wrong_code_exits_5() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["otp", "validate", "000000", "--secret", SECRET])
        .assert().failure().code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn otp_validate_secret_redacted_in_error_json() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "otp", "validate", "000000", "--secret", "do-not-leak"])
        .assert().failure().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains("do-not-leak"), "secret leaked in JSON: {s}");
}
