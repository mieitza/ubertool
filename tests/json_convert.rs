use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn json_minify_compacts_whitespace() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "minify", r#"{ "a": 1, "b": [1, 2] }"#])
        .assert()
        .success()
        .stdout("{\"a\":1,\"b\":[1,2]}\n");
}

#[test]
fn json_prettify_indents_two_spaces() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "prettify", r#"{"a":1,"b":2}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("  \"a\": 1"));
}

#[test]
fn json_prettify_custom_indent() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "prettify", r#"{"a":1}"#, "--indent", "4"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("    \"a\": 1"));
}

#[test]
fn json_to_yaml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-yaml", r#"{"name":"alice","age":30}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name: alice"));
    assert!(s.contains("age: 30"));
}

#[test]
fn json_to_toml_basic() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-toml", r#"{"name":"alice","age":30}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("name = \"alice\""));
    assert!(s.contains("age = 30"));
}

#[test]
fn json_to_toml_null_value_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "to-toml", r#"{"a":null}"#])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_toml"));
}

#[test]
fn json_minify_invalid_input_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["json", "minify", "{not json}"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_json"));
}

#[test]
fn json_to_yaml_json_mode_wraps_in_yaml_field() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "json", "to-yaml", r#"{"k":"v"}"#])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let yaml = v["yaml"].as_str().expect("must have yaml field");
    assert!(yaml.contains("k: v"));
}
