use assert_cmd::Command;

#[test]
fn password_score_weak() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "password"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let score = v["score"].as_u64().unwrap();
    assert!(
        score <= 1,
        "expected weak score for 'password', got {score}"
    );
}

#[test]
fn password_score_strong() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "password",
            "score",
            "Tr0ub4dor&3-correct-horse-battery-staple",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let score = v["score"].as_u64().unwrap();
    assert!(score >= 3, "expected strong score, got {score}");
}

#[test]
fn password_score_emits_strength_label_and_suggestions() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "abc"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["strength"].is_string(), "must emit strength label");
    assert!(v["guesses_log10"].is_number());
    assert!(v["suggestions"].is_array());
}

#[test]
fn password_score_does_not_echo_password_in_output() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "password", "score", "my-secret-password-xyz"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("my-secret-password-xyz"),
        "password must not appear in score output: {s}"
    );
}
