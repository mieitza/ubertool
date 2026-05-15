use assert_cmd::Command;

#[test]
fn hmac_sha256_known_vector() {
    // HMAC-SHA256("key", "The quick brown fox jumps over the lazy dog")
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "hmac",
            "sha256",
            "The quick brown fox jumps over the lazy dog",
            "--key",
            "key",
        ])
        .assert()
        .success()
        .stdout("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8\n");
}

#[test]
fn hmac_sha256_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "hmac", "sha256", "hello", "--key", "secret",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("must be valid JSON");
    assert!(v["hmac"].is_string(), "hmac field must be a string");
}

#[test]
fn hmac_sha1_known_vector() {
    // HMAC-SHA1("key", "The quick brown fox jumps over the lazy dog")
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "hmac",
            "sha1",
            "The quick brown fox jumps over the lazy dog",
            "--key",
            "key",
        ])
        .assert()
        .success()
        .stdout("de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9\n");
}

#[test]
fn hmac_via_stdin_with_key_flag() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hmac", "sha256", "--key", "key"])
        .write_stdin("The quick brown fox jumps over the lazy dog")
        .assert()
        .success()
        .stdout("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8\n");
}

#[test]
fn hmac_missing_key_is_usage_error() {
    // clap should refuse with exit 2 because --key is required.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hmac", "sha256", "hello"])
        .assert()
        .failure()
        .code(2);
}
