use assert_cmd::Command;

#[test]
fn markdown_to_html_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["markdown", "to-html", "# Hello\n\nWorld **bold**"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<h1>"));
    assert!(s.contains("Hello"));
    assert!(s.contains("<strong>"));
    assert!(s.contains("bold"));
}

#[test]
fn markdown_to_html_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "markdown", "to-html", "# Title"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let html = v["html"].as_str().expect("html field");
    assert!(html.contains("<h1>Title</h1>"));
}

#[test]
fn markdown_to_html_link() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["markdown", "to-html", "[example](https://example.com)"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("<a href=\"https://example.com\">example</a>"));
}
