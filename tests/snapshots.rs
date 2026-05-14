//! Snapshot tests for --help output. The whole point: any drift in --help
//! becomes a deliberate, reviewable change.

use assert_cmd::Command;

fn ubertool() -> Command {
    Command::cargo_bin("ubertool").expect("binary built")
}

fn capture_stdout(cmd: &mut Command) -> String {
    let out = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(out).expect("help is utf-8")
}

#[test]
fn snapshot_top_level_help() {
    let s = capture_stdout(ubertool().arg("--help"));
    insta::assert_snapshot!("top_level_help", s);
}

#[test]
fn snapshot_base64_help() {
    let s = capture_stdout(ubertool().args(["base64", "--help"]));
    insta::assert_snapshot!("base64_help", s);
}

#[test]
fn snapshot_base64_encode_help() {
    let s = capture_stdout(ubertool().args(["base64", "encode", "--help"]));
    insta::assert_snapshot!("base64_encode_help", s);
}

#[test]
fn snapshot_base64_decode_help() {
    let s = capture_stdout(ubertool().args(["base64", "decode", "--help"]));
    insta::assert_snapshot!("base64_decode_help", s);
}
