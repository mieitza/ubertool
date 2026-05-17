use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cipher_encrypt_emits_self_describing_format() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "encrypt", "hello", "--password", "pw"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split('$').collect();
    assert_eq!(parts.len(), 5, "expected 5 $-separated parts: {s}");
    assert_eq!(parts[0], "aes-gcm");
    assert_eq!(parts[1], "argon2");
}

#[test]
fn cipher_round_trip_aes_gcm_argon2() {
    let ct = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "encrypt", "the message", "--password", "pw"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "pw"])
        .assert()
        .success()
        .stdout("the message\n");
}

#[test]
fn cipher_round_trip_chacha20_pbkdf2() {
    let ct = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "cipher",
            "encrypt",
            "msg",
            "--password",
            "pw",
            "--algo",
            "chacha20-poly1305",
            "--kdf",
            "pbkdf2",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    let parts: Vec<&str> = ct_s.split('$').collect();
    assert_eq!(parts[0], "chacha20-poly1305");
    assert_eq!(parts[1], "pbkdf2");
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "pw"])
        .assert()
        .success()
        .stdout("msg\n");
}

#[test]
fn cipher_decrypt_wrong_password_exits_5() {
    let ct = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "encrypt", "msg", "--password", "right"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let ct_s = String::from_utf8(ct).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "decrypt", &ct_s, "--password", "wrong"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("decrypt_failed"));
}

#[test]
fn cipher_decrypt_malformed_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["cipher", "decrypt", "garbage", "--password", "pw"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_cipher"));
}

#[test]
fn cipher_password_redacted_in_error_json() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "cipher",
            "decrypt",
            "garbage",
            "--password",
            "do-not-leak",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("do-not-leak"),
        "password leaked in error JSON: {s}"
    );
}
