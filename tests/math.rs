use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn math_eval_simple() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["math", "eval", "1 + 2"])
        .assert()
        .success()
        .stdout("3\n");
}

#[test]
fn math_eval_multiplication() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["math", "eval", "6 * 7"])
        .assert()
        .success()
        .stdout("42\n");
}

#[test]
fn math_eval_float() {
    // 1.5 * 4 = 6.0 — exact in binary floating point, no approx-constant lint
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["math", "eval", "1.5 * 4"])
        .assert()
        .success()
        .stdout(predicates::str::contains("6"));
}

#[test]
fn math_eval_parens_and_precedence() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["math", "eval", "(2 + 3) * 4"])
        .assert()
        .success()
        .stdout("20\n");
}

#[test]
fn math_eval_invalid_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["math", "eval", "not math"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_math"));
}

#[test]
fn math_eval_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "math", "eval", "1 + 2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["result"].is_number());
}
