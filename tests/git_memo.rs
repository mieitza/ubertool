use assert_cmd::Command;

#[test]
fn git_memo_prints_markdown() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["git", "memo"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Git cheat sheet"));
    assert!(s.contains("git add"));
    assert!(s.contains("git commit"));
}

#[test]
fn git_memo_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "git", "memo"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let memo = v["memo"].as_str().expect("memo field");
    assert!(memo.contains("Git cheat sheet"));
}
