use assert_cmd::Command;

#[test]
fn slugify_basic() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["slugify", "generate", "Hello, World!"])
        .assert().success()
        .stdout("hello-world\n");
}

#[test]
fn slugify_unicode() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["slugify", "generate", "Café Résumé"])
        .assert().success()
        .stdout("cafe-resume\n");
}

#[test]
fn slugify_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "slugify", "generate", "Hello World"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["slug"], "hello-world");
}

#[test]
fn slugify_via_stdin() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["slugify", "generate"])
        .write_stdin("Some Title")
        .assert().success()
        .stdout("some-title\n");
}
