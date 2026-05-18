use assert_cmd::Command;

#[test]
fn svg_placeholder_default_dimensions() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["svg-placeholder", "generate"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("400"));
    assert!(s.contains("300"));
}

#[test]
fn svg_placeholder_custom_dimensions() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "svg-placeholder",
            "generate",
            "--width",
            "800",
            "--height",
            "200",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("800"));
    assert!(s.contains("200"));
    assert!(s.contains("800x200") || s.contains("800 x 200"));
}

#[test]
fn svg_placeholder_custom_text() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["svg-placeholder", "generate", "--text", "Hello"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Hello"));
}

#[test]
fn svg_placeholder_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "svg-placeholder",
            "generate",
            "--width",
            "100",
            "--height",
            "100",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let svg = v["svg"].as_str().expect("svg field");
    assert!(svg.contains("<svg"));
}
