use assert_cmd::Command;

#[test]
fn port_random_in_ephemeral_range() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let n: u16 = s.trim().parse().expect("port must be u16");
    assert!((49152..=65535).contains(&n), "expected ephemeral range, got {n}");
}

#[test]
fn port_random_custom_range() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random", "--min", "8000", "--max", "8099"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let n: u16 = s.trim().parse().expect("port must be u16");
    assert!((8000..=8099).contains(&n), "expected 8000-8099, got {n}");
}

#[test]
fn port_random_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "port", "random"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["port"].is_number());
}

#[test]
fn port_random_invalid_range_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["port", "random", "--min", "200", "--max", "100"])
        .assert().failure().code(2);
}
