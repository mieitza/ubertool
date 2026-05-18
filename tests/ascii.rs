use assert_cmd::Command;

#[test]
fn ascii_draw_renders_letters() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["ascii", "draw", "Hi"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.lines().count() >= 4);
}

#[test]
fn ascii_draw_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "ascii", "draw", "X"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let art = v["art"].as_str().expect("art field");
    assert!(!art.is_empty());
}

#[test]
fn ascii_draw_empty_input() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["ascii", "draw", ""])
        .assert().success();
}
