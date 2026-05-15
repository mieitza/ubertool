use assert_cmd::Command;

#[test]
fn token_new_default_is_hex_64_chars() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["token", "new"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 64);
    assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn token_new_length_16_hex_emits_32_chars() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["token", "new", "--length", "16"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert_eq!(s.trim().len(), 32);
}

#[test]
fn token_new_format_base64() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["token", "new", "--length", "12", "--format", "base64"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert_eq!(s.trim().len(), 16);
}

#[test]
fn token_new_format_alphanumeric() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["token", "new", "--length", "24", "--format", "alphanumeric"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 24);
    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn token_new_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "token", "new"]).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["token"].is_string());
}
