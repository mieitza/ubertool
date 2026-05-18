use assert_cmd::Command;

#[test]
fn http_status_lookup_200() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "http-status", "lookup", "200"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["code"], 200);
    assert_eq!(v["name"], "OK");
    assert_eq!(v["category"], "Success");
    assert_eq!(v["valid"], true);
}

#[test]
fn http_status_lookup_404() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "http-status", "lookup", "404"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["name"], "Not Found");
    assert_eq!(v["category"], "Client Error");
}

#[test]
fn http_status_lookup_unknown_emits_null() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "http-status", "lookup", "999"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["code"], 999);
    assert!(v["name"].is_null());
    assert_eq!(v["valid"], false);
}

#[test]
fn http_status_lookup_invalid_code_exits_2() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["http-status", "lookup", "not-a-number"])
        .assert()
        .failure()
        .code(2);
}
