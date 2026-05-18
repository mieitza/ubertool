use assert_cmd::Command;

#[test]
fn completions_bash_emits_script() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "bash"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("complete -F"), "expected bash completion script: {s}");
    assert!(s.contains("ubertool"));
    assert!(s.lines().count() >= 50);
}

#[test]
fn completions_zsh_emits_script() {
    let out = Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "zsh"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("#compdef ubertool"));
}

#[test]
fn completions_fish_emits_script() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "fish"])
        .assert().success();
}

#[test]
fn completions_invalid_shell_exits_2() {
    Command::cargo_bin("ubertool").unwrap()
        .args(["completions", "csh"])
        .assert().failure().code(2);
}
