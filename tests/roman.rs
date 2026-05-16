use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn roman_to_num_known() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "to-num", "MCMXCIX"])
        .assert()
        .success()
        .stdout("1999\n");
}

#[test]
fn roman_from_num_known() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "1999"])
        .assert()
        .success()
        .stdout("MCMXCIX\n");
}

#[test]
fn roman_round_trip() {
    for (decimal, roman) in [
        (1, "I"),
        (4, "IV"),
        (9, "IX"),
        (40, "XL"),
        (90, "XC"),
        (400, "CD"),
        (900, "CM"),
        (3888, "MMMDCCCLXXXVIII"),
    ] {
        Command::cargo_bin("ubertool")
            .unwrap()
            .args(["roman", "from-num", &decimal.to_string()])
            .assert()
            .success()
            .stdout(format!("{roman}\n"));
        Command::cargo_bin("ubertool")
            .unwrap()
            .args(["roman", "to-num", roman])
            .assert()
            .success()
            .stdout(format!("{decimal}\n"));
    }
}

#[test]
fn roman_to_num_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "to-num", "ZZZ"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_roman"));
}

#[test]
fn roman_from_num_out_of_range_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "4000"])
        .assert()
        .failure()
        .code(3);
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["roman", "from-num", "0"])
        .assert()
        .failure()
        .code(3);
}
