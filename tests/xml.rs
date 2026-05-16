use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn xml_to_json_simple_element() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "xml", "to-json", "<root><name>alice</name><age>30</age></root>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let json_str = v["json"].as_str().expect("json field");
    let parsed: serde_json::Value = serde_json::from_str(json_str).expect("inner json");
    assert_eq!(parsed["root"]["name"], "alice");
    assert_eq!(parsed["root"]["age"], "30");
}

#[test]
fn xml_to_json_attributes() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "xml", "to-json", r#"<user id="42" role="admin">alice</user>"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let parsed: serde_json::Value =
        serde_json::from_str(v["json"].as_str().unwrap()).expect("inner json");
    assert_eq!(parsed["user"]["@id"], "42");
    assert_eq!(parsed["user"]["@role"], "admin");
    assert_eq!(parsed["user"]["#text"], "alice");
}

#[test]
fn xml_to_json_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "to-json", "<unclosed"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_xml"));
}

#[test]
fn xml_format_pretty_prints() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "format", "<root><a>1</a><b>2</b></root>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains('\n'), "format output should contain newlines: {s}");
    assert!(s.contains("<a>1</a>"));
}

#[test]
fn xml_format_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["xml", "format", "<bad"])
        .assert()
        .failure()
        .code(3);
}
