use assert_cmd::Command;

#[test]
fn pct_of() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "of", "20", "50"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 10.0).abs() < 1e-9);
}

#[test]
fn pct_change_positive() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "100", "125"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 25.0).abs() < 1e-9);
}

#[test]
fn pct_change_negative() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "100", "75"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v + 25.0).abs() < 1e-9);
}

#[test]
fn pct_of_total() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "of-total", "20", "50"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 40.0).abs() < 1e-9);
}

#[test]
fn pct_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "percentage", "of-total", "1", "4"]).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["percentage"].as_f64().unwrap() == 25.0);
}

#[test]
fn pct_change_zero_base_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["percentage", "change", "0", "100"])
        .assert().failure().code(3);
}
