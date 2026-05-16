use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn integer_base_dec_to_hex() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "255", "--from", "10", "--to", "16"])
        .assert().success()
        .stdout("ff\n");
}

#[test]
fn integer_base_hex_to_dec() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "ff", "--from", "16", "--to", "10"])
        .assert().success()
        .stdout("255\n");
}

#[test]
fn integer_base_bin_to_dec() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "11111111", "--from", "2", "--to", "10"])
        .assert().success()
        .stdout("255\n");
}

#[test]
fn integer_base_dec_to_bin() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "10", "--from", "10", "--to", "2"])
        .assert().success()
        .stdout("1010\n");
}

#[test]
fn integer_base_invalid_digit_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "xyz", "--from", "10", "--to", "16"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn integer_base_out_of_range_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["integer-base", "convert", "10", "--from", "37", "--to", "10"])
        .assert().failure().code(2);
}

#[test]
fn integer_base_json_mode() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "integer-base", "convert", "255", "--from", "10", "--to", "16"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["result"], "ff");
}
