use assert_cmd::Command;

#[test]
fn csv_to_json_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "name,age\nalice,30\nbob,40"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["name"], "alice");
    assert_eq!(v[0]["age"], "30");
    assert_eq!(v[1]["name"], "bob");
}

#[test]
fn csv_to_json_via_stdin() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json"])
        .write_stdin("a,b\n1,2\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["a"], "1");
    assert_eq!(v[0]["b"], "2");
}

#[test]
fn csv_to_json_custom_delimiter() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "a;b\n1;2", "--delimiter", ";"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().trim()).expect("valid JSON");
    assert_eq!(v[0]["a"], "1");
}

#[test]
fn csv_invalid_input_exits_2() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["csv", "to-json", "a,b\n1,2", "--delimiter", "ab"])
        .assert()
        .failure()
        .code(2);
}
