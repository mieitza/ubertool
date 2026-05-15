use assert_cmd::Command;
use predicates::prelude::*;

// Valid HS256 token: header={"alg":"HS256","typ":"JWT"}, claims={"sub":"1234567890","name":"John Doe","iat":1516239022}
// Signed with secret="secret". (The jwt.io example token uses a different signing key.)
const HS256_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.XbPfbIHMI6arZ3Y922BhjWgQzWXcXNrz0ogtVhfEd2o";

#[test]
fn jwt_decode_emits_header_and_claims() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "jwt", "decode", HS256_TOKEN])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["header"]["alg"], "HS256");
    assert_eq!(v["header"]["typ"], "JWT");
    assert_eq!(v["claims"]["sub"], "1234567890");
    assert_eq!(v["claims"]["name"], "John Doe");
}

#[test]
fn jwt_decode_malformed_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "decode", "not.a.jwt"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}

#[test]
fn jwt_verify_correct_secret_succeeds() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success();
}

#[test]
fn jwt_verify_correct_secret_emits_claims_json() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
    assert_eq!(v["claims"]["sub"], "1234567890");
}

#[test]
fn jwt_verify_wrong_secret_exits_5() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "wrong"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn jwt_verify_secret_is_redacted_from_json_error_input() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "jwt",
            "verify",
            HS256_TOKEN,
            "--secret",
            "wrong-secret-do-not-leak",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("wrong-secret-do-not-leak"),
        "secret leaked in JSON error: {s}"
    );
}

#[test]
fn jwt_verify_malformed_token_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", "garbage", "--secret", "secret"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}
