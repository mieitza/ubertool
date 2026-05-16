use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn email_normalize_lowercases() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "email", "normalize", "Alice@Example.COM"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["email"], "alice@example.com");
    assert_eq!(v["valid"], true);
}

#[test]
fn email_normalize_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["email", "normalize", "not-an-email"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_email"));
}
