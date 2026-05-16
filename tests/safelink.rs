use assert_cmd::Command;

#[test]
fn safelink_outlook_unwraps() {
    let wrapped = "https://nam04.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com%2Fpath%3Fa%3D1&data=...&reserved=0";
    Command::cargo_bin("ubertool").unwrap()
        .args(["safelink", "decode", wrapped])
        .assert().success()
        .stdout("https://example.com/path?a=1\n");
}

#[test]
fn safelink_google_unwraps() {
    let wrapped = "https://www.google.com/url?q=https%3A%2F%2Fexample.com&sa=D";
    Command::cargo_bin("ubertool").unwrap()
        .args(["safelink", "decode", wrapped])
        .assert().success()
        .stdout("https://example.com\n");
}

#[test]
fn safelink_unwrapped_passes_through() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["safelink", "decode", "https://example.com/path"])
        .assert().success()
        .stdout("https://example.com/path\n");
}

#[test]
fn safelink_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args([
            "--json", "safelink", "decode",
            "https://safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com",
        ])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["url"], "https://example.com");
}
