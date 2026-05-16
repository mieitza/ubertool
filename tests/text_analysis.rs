use assert_cmd::Command;

#[test]
fn text_stats_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "text", "stats", "hello world\nfoo bar"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["chars"], 19);
    assert_eq!(v["words"], 4);
    assert_eq!(v["lines"], 2);
    assert!(v["bytes"].as_u64().unwrap() >= 19);
}

#[test]
fn text_obfuscate_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["text", "obfuscate", "hello world"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert_eq!(s, "h***o w***d");
}

#[test]
fn text_obfuscate_short_word() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["text", "obfuscate", "hi"]).assert().success()
        .stdout("h*\n");
}

#[test]
fn text_diff_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["text", "diff", "hello world", "hello rust"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("hello"));
    assert!(s.contains("world") || s.contains("rust"));
}

#[test]
fn text_diff_identical_inputs_emit_empty_diff() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "text", "diff", "same", "same"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let diff = v["diff"].as_str().expect("diff field");
    assert!(diff.is_empty() || diff.lines().all(|l| !l.starts_with('+') && !l.starts_with('-')));
}
