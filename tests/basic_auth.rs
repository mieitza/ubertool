use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn basic_auth_encode_text_mode() {
    // "Aladdin:open sesame" → "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "basic-auth",
            "encode",
            "--user",
            "Aladdin",
            "--pass",
            "open sesame",
        ])
        .assert()
        .success()
        .stdout("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==\n");
}

#[test]
fn basic_auth_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "basic-auth",
            "encode",
            "--user",
            "user",
            "--pass",
            "pass",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["header"], "Basic dXNlcjpwYXNz");
}

#[test]
fn basic_auth_decode_with_basic_prefix() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "basic-auth", "decode", "Basic dXNlcjpwYXNz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["user"], "user");
    assert_eq!(v["pass"], "pass");
}

#[test]
fn basic_auth_decode_without_basic_prefix() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "basic-auth", "decode", "dXNlcjpwYXNz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["user"], "user");
}

#[test]
fn basic_auth_decode_invalid_base64_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["basic-auth", "decode", "!!!"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_base64"));
}

#[test]
fn basic_auth_decode_missing_colon_exits_3() {
    // "nocolon" base64-encoded — decodes fine but has no colon separator.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["basic-auth", "decode", "bm9jb2xvbg=="])
        .assert()
        .failure()
        .code(3);
}
