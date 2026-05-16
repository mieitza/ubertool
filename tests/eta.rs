use assert_cmd::Command;

#[test]
fn eta_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "eta", "calc", "--done", "50", "--total", "100", "--elapsed", "60"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!((v["remaining_seconds"].as_f64().unwrap() - 60.0).abs() < 1e-6);
    assert!(v["eta_iso"].is_string());
    assert!(v["rate_per_second"].is_number());
}

#[test]
fn eta_done_equals_total_emits_zero_remaining() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "eta", "calc", "--done", "100", "--total", "100", "--elapsed", "60"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["remaining_seconds"].as_f64().unwrap(), 0.0);
}

#[test]
fn eta_zero_done_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["eta", "calc", "--done", "0", "--total", "100", "--elapsed", "60"])
        .assert().failure().code(3);
}
