use assert_cmd::Command;

#[test]
fn ipv6_ula_new_format() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["ipv6-ula", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    assert!(
        s.to_lowercase().starts_with("fd"),
        "ULA must start with fd: {s}"
    );
    assert!(s.ends_with("/48"), "ULA expected /48 prefix: {s}");
}

#[test]
fn ipv6_ula_new_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "ipv6-ula", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["prefix"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .starts_with("fd"));
    assert!(v["global_id"].is_string());
}

#[test]
fn ipv6_ula_two_calls_differ() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool")
            .unwrap()
            .args(["ipv6-ula", "new"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one());
}
