use assert_cmd::Command;

#[test]
fn temp_c_to_f() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["temperature", "convert", "100", "--from", "celsius", "--to", "fahrenheit"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 212.0).abs() < 1e-6);
}

#[test]
fn temp_f_to_c() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["temperature", "convert", "32", "--from", "fahrenheit", "--to", "celsius"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!(v.abs() < 1e-6);
}

#[test]
fn temp_c_to_k() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["temperature", "convert", "0", "--from", "celsius", "--to", "kelvin"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let v: f64 = s.trim().parse().unwrap();
    assert!((v - 273.15).abs() < 1e-6);
}

#[test]
fn temp_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "temperature", "convert", "0", "--from", "celsius", "--to", "kelvin"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["value"].is_number());
    assert_eq!(v["unit"], "kelvin");
}
