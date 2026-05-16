use assert_cmd::Command;

#[test]
fn list_comma_to_newline() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "list", "convert", "a,b,c", "--from", "comma", "--to", "newline",
        ])
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn list_newline_to_comma() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "list", "convert", "a\nb\nc", "--from", "newline", "--to", "comma",
        ])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_dedupe_and_sort() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "list",
            "convert",
            "c,b,a,a,c",
            "--from",
            "comma",
            "--to",
            "comma",
            "--dedupe",
            "--sort",
        ])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_trim() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "list",
            "convert",
            " a , b , c ",
            "--from",
            "comma",
            "--to",
            "comma",
            "--trim",
        ])
        .assert()
        .success()
        .stdout("a,b,c\n");
}

#[test]
fn list_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json", "list", "convert", "a,b,c", "--from", "comma", "--to", "comma",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["items"][0], "a");
    assert_eq!(v["items"][2], "c");
}
