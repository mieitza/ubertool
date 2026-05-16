use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn crontab_describe_every_minute() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "describe", "* * * * *"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap().to_lowercase();
    assert!(s.contains("every minute"));
}

#[test]
fn crontab_describe_daily_at_specific_time() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "describe", "30 14 * * *"]).assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("14:30"));
}

#[test]
fn crontab_next_basic() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "crontab", "next", "0 * * * *"]).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let arr = v["next"].as_array().expect("next array");
    assert_eq!(arr.len(), 5);
    for entry in arr {
        assert!(entry.is_string());
    }
}

#[test]
fn crontab_next_custom_count() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["--json", "crontab", "next", "0 * * * *", "--count", "3"]).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["next"].as_array().unwrap().len(), 3);
}

#[test]
fn crontab_invalid_exits_3() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["crontab", "next", "not a cron"])
        .assert().failure().code(3)
        .stderr(predicate::str::contains("invalid_cron"));
}
