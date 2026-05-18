use assert_cmd::Command;

#[test]
fn regex_memo_prints_markdown() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["regex", "memo"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Regex cheat sheet"));
    assert!(s.contains("Anchors"));
    assert!(s.contains("\\d"));
}

#[test]
fn regex_memo_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "regex", "memo"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let memo = v["memo"].as_str().expect("memo field");
    assert!(memo.contains("Regex cheat sheet"));
}
