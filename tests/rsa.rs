use assert_cmd::Command;

#[test]
fn rsa_keypair_emits_pem_pair() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "rsa", "keypair", "--bits", "2048"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let priv_pem = v["private_pem"].as_str().expect("private_pem");
    let pub_pem = v["public_pem"].as_str().expect("public_pem");
    assert!(priv_pem.contains("BEGIN PRIVATE KEY") || priv_pem.contains("BEGIN RSA PRIVATE KEY"));
    assert!(pub_pem.contains("BEGIN PUBLIC KEY") || pub_pem.contains("BEGIN RSA PUBLIC KEY"));
    assert_eq!(v["bits"], 2048);
}

#[test]
fn rsa_keypair_default_bits_2048() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "rsa", "keypair"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["bits"], 2048);
}

#[test]
fn rsa_keypair_unsupported_bits_exits_2() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["rsa", "keypair", "--bits", "100"])
        .assert()
        .failure()
        .code(2);
}
