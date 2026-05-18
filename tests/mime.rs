use assert_cmd::Command;

#[test]
fn mime_lookup_json() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/json");
    assert_eq!(v["extension"], "json");
}

#[test]
fn mime_lookup_filename() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "report.pdf"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/pdf");
    assert_eq!(v["extension"], "pdf");
}

#[test]
fn mime_lookup_dot_prefix() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", ".png"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "image/png");
}

#[test]
fn mime_lookup_unknown_emits_null() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "xyzabc"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["mime"].is_null());
    assert_eq!(v["extension"], "xyzabc");
}

#[test]
fn mime_lookup_case_insensitive() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "mime", "lookup", "JSON"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mime"], "application/json");
}
