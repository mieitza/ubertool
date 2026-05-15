use assert_cmd::Command;

#[test]
fn uuid_new_default_is_v4() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split('-').collect();
    assert_eq!(parts.len(), 5, "expected 5 hyphenated parts in {s}");
    assert!(
        parts[2].starts_with('4'),
        "v4 must start version nibble with 4 in {s}"
    );
}

#[test]
fn uuid_new_v7_emits_time_ordered_uuid() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new", "--version", "7"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split('-').collect();
    assert!(
        parts[2].starts_with('7'),
        "v7 must start version nibble with 7 in {s}"
    );
}

#[test]
fn uuid_new_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "uuid", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["uuid"].is_string());
    let s = v["uuid"].as_str().unwrap();
    assert_eq!(s.len(), 36, "uuid length must be 36 with hyphens");
}

#[test]
fn uuid_new_invalid_version_is_usage_error() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["uuid", "new", "--version", "99"])
        .assert()
        .failure()
        .code(2);
}
