use assert_cmd::Command;

#[test]
fn case_convert_snake() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "Hello World", "--style", "snake"])
        .assert()
        .success()
        .stdout("hello_world\n");
}

#[test]
fn case_convert_kebab() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "Hello World", "--style", "kebab"])
        .assert()
        .success()
        .stdout("hello-world\n");
}

#[test]
fn case_convert_camel() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "hello world", "--style", "camel"])
        .assert()
        .success()
        .stdout("helloWorld\n");
}

#[test]
fn case_convert_pascal() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "hello world", "--style", "pascal"])
        .assert()
        .success()
        .stdout("HelloWorld\n");
}

#[test]
fn case_convert_screaming_snake() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "case",
            "convert",
            "hello world",
            "--style",
            "screaming-snake",
        ])
        .assert()
        .success()
        .stdout("HELLO_WORLD\n");
}

#[test]
fn case_convert_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "case",
            "convert",
            "hello world",
            "--style",
            "snake",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["result"], "hello_world");
}

#[test]
fn case_convert_invalid_style_is_usage_error() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["case", "convert", "x", "--style", "bogus"])
        .assert()
        .failure()
        .code(2);
}
