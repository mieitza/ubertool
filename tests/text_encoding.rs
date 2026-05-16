use assert_cmd::Command;

#[test]
fn text_to_binary_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "A"])
        .assert()
        .success()
        .stdout("01000001\n");
}

#[test]
fn text_to_binary_multi_char() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "AB"])
        .assert()
        .success()
        .stdout("01000001 01000010\n");
}

#[test]
fn text_from_binary_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-binary", "01000001 01000010"])
        .assert()
        .success()
        .stdout("AB\n");
}

#[test]
fn text_round_trip_binary() {
    let bin = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-binary", "Hello"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let bin_s = String::from_utf8(bin).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-binary", &bin_s])
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn text_to_unicode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-unicode", "AB"])
        .assert()
        .success()
        .stdout("U+0041 U+0042\n");
}

#[test]
fn text_from_unicode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "from-unicode", "U+0041 U+0042"])
        .assert()
        .success()
        .stdout("AB\n");
}

#[test]
fn text_to_nato_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-nato", "ABC"])
        .assert()
        .success()
        .stdout("Alpha Bravo Charlie\n");
}

#[test]
fn text_to_nato_with_digit() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["text", "to-nato", "A1B"])
        .assert()
        .success()
        .stdout("Alpha One Bravo\n");
}
