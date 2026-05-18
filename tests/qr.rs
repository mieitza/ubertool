use assert_cmd::Command;

#[test]
fn qr_generate_svg_default() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "generate", "https://example.com"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("</svg>"));
}

#[test]
fn qr_generate_png_to_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "generate", "hello", "--format", "png", "--out"]).arg(tmp.path())
        .assert().success();
    let bytes = std::fs::read(tmp.path()).unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "expected PNG magic");
}

#[test]
fn qr_generate_json_mode_wraps_svg() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "qr", "generate", "hi"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let svg = v["svg"].as_str().expect("svg field");
    assert!(svg.contains("<svg"));
}

#[test]
fn qr_wifi_emits_svg() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "wifi", "--ssid", "MyNet", "--password", "pw", "--security", "WPA"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<svg"));
}

#[test]
fn qr_wifi_nopass_security() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["qr", "wifi", "--ssid", "Public", "--security", "nopass"])
        .assert().success();
}
