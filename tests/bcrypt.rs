use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn bcrypt_hash_produces_valid_format() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "secret123", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert!(
        s.starts_with("$2") && s.len() >= 60,
        "expected bcrypt hash format, got: {s}"
    );
}

#[test]
fn bcrypt_verify_wrong_password_exits_5() {
    // Use a freshly-generated hash to avoid version-portability assumptions.
    let hash_out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "secret", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8(hash_out).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "wrongpass", "--hash", &hash])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn bcrypt_verify_malformed_hash_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "secret", "--hash", "not-a-bcrypt-hash"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_bcrypt"));
}

#[test]
fn bcrypt_hash_round_trip_via_verify() {
    let hash_out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "round-trip-pw", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8(hash_out).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "verify", "round-trip-pw", "--hash", &hash])
        .assert()
        .success();
}

#[test]
fn bcrypt_verify_json_mode_emits_verified_true() {
    // Generate a hash first, then verify it in --json mode.
    let hash_out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["bcrypt", "hash", "secret", "--cost", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8(hash_out).unwrap().trim().to_string();
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "bcrypt", "verify", "secret", "--hash", &hash])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
}
