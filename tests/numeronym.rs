use assert_cmd::Command;

#[test]
fn numeronym_i18n() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "internationalization"])
        .assert().success()
        .stdout("i18n\n");
}

#[test]
fn numeronym_k8s() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "kubernetes"])
        .assert().success()
        .stdout("k8s\n");
}

#[test]
fn numeronym_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "numeronym", "generate", "accessibility"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["numeronym"], "a11y");
}

#[test]
fn numeronym_short_word_passes_through() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "hi"])
        .assert().success()
        .stdout("hi\n");
}

#[test]
fn numeronym_single_char_passes_through() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["numeronym", "generate", "x"])
        .assert().success()
        .stdout("x\n");
}
