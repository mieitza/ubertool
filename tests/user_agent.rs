use assert_cmd::Command;

#[test]
fn ua_parse_chrome() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "user-agent", "parse",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["browser"].as_str().unwrap().to_lowercase().contains("chrome"));
    assert!(v["os"].as_str().unwrap().to_lowercase().contains("mac"));
}

#[test]
fn ua_parse_unknown_emits_unknown_fields() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "user-agent", "parse", "definitely-not-a-real-ua"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["browser"].is_string());
}
