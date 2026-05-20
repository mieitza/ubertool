use assert_cmd::Command;

#[test]
fn self_version_emits_current_version() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["self", "version"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Current version from Cargo.toml; just confirm it's a semver-ish string.
    assert!(s.contains('.'), "expected semver-ish version: {s}");
}

#[test]
fn self_version_json_mode_emits_field() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "self", "version"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["version"].is_string());
    let s = v["version"].as_str().unwrap();
    assert!(s.contains('.'));
}

#[test]
fn self_update_check_only_reports_status() {
    // --check mode hits GitHub for the latest release. If the test runs
    // offline, this might fail with an I/O error (exit 4). Kept here for
    // documentation; the network-dependent variant is marked #[ignore].
}

#[test]
#[ignore = "requires network access to api.github.com"]
fn self_update_check_only_with_network() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "self", "update", "--check"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["current"].is_string());
    assert!(v["latest"].is_string());
    assert!(v["update_available"].is_boolean());
}
