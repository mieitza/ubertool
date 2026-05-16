use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn chmod_calc_755() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["chmod", "calc", "rwxr-xr-x"])
        .assert()
        .success()
        .stdout("755\n");
}

#[test]
fn chmod_calc_644() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["chmod", "calc", "rw-r--r--"])
        .assert()
        .success()
        .stdout("644\n");
}

#[test]
fn chmod_calc_777() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["chmod", "calc", "rwxrwxrwx"])
        .assert()
        .success()
        .stdout("777\n");
}

#[test]
fn chmod_parse_755() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "chmod", "parse", "755"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["symbolic"], "rwxr-xr-x");
    assert_eq!(v["owner"]["read"], true);
    assert_eq!(v["owner"]["write"], true);
    assert_eq!(v["owner"]["execute"], true);
    assert_eq!(v["group"]["write"], false);
    assert_eq!(v["other"]["write"], false);
}

#[test]
fn chmod_parse_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["chmod", "parse", "999"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_chmod"));
}

#[test]
fn chmod_calc_invalid_symbolic_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["chmod", "calc", "rwxabcdef"])
        .assert()
        .failure()
        .code(3);
}
