use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn sha256_of_hello_text_mode() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "hello"])
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}

#[test]
fn sha256_of_hello_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "hash", "sha256", "hello"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("must be valid JSON");
    assert_eq!(
        v["hash"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn md5_of_empty_input() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "md5", ""])
        .assert()
        .success()
        .stdout("d41d8cd98f00b204e9800998ecf8427e\n");
}

#[test]
fn sha1_known_vector() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha1", "hello"])
        .assert()
        .success()
        .stdout("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\n");
}

#[test]
fn sha512_known_vector() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha512", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca",
        ));
}

#[test]
fn sha3_256_known_vector() {
    // SHA3-256 of "hello"
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha3-256", "hello"])
        .assert()
        .success()
        .stdout("3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392\n");
}

#[test]
fn sha256_via_stdin_pipe() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256"])
        .write_stdin("hello")
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}

#[test]
fn sha256_via_in_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"hello").unwrap();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "--in"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n");
}
