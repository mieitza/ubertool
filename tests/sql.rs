use assert_cmd::Command;

#[test]
fn sql_format_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["sql", "format", "SELECT * FROM users WHERE id=1"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("SELECT"));
    assert!(s.contains("FROM"));
    assert!(s.contains("WHERE"));
    assert!(s.contains('\n'), "expected formatted output to contain newlines");
}

#[test]
fn sql_format_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "sql", "format", "select 1"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let formatted = v["sql"].as_str().expect("sql field");
    assert!(formatted.to_uppercase().contains("SELECT"));
}

#[test]
fn sql_format_uppercase_keywords() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["sql", "format", "select * from t", "--uppercase"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("SELECT"));
}
