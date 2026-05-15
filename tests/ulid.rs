use assert_cmd::Command;

#[test]
fn ulid_new_text_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["ulid", "new"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s.len(), 26, "expected 26-char ULID, got {s}");
    for c in s.chars() {
        assert!(c.is_ascii_alphanumeric(), "non-alphanumeric in ULID: {s}");
    }
}

#[test]
fn ulid_new_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ulid", "new"]).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let s = v["ulid"].as_str().expect("ulid field must be a string");
    assert_eq!(s.len(), 26);
}

#[test]
fn ulid_new_two_calls_produce_different_ids() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool").unwrap()
            .args(["ulid", "new"]).assert().success().get_output().stdout.clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one(), "two ULIDs must differ");
}
