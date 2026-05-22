use assert_cmd::Command;

fn lines(out: Vec<u8>) -> Vec<serde_json::Value> {
    String::from_utf8(out)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("each line must be valid JSON"))
        .collect()
}

#[test]
fn batch_hash_emits_jsonl() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "--batch"])
        .write_stdin("alice\nbob\ncarol\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs.len(), 3);
    assert_eq!(recs[0]["input"], "alice");
    assert!(recs[0]["hash"].is_string());
    assert_eq!(recs[1]["input"], "bob");
}

#[test]
fn batch_hash_known_vector() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "--batch"])
        .write_stdin("hello\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(
        recs[0]["hash"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn batch_hmac_with_key() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hmac", "sha256", "--key", "k", "--batch"])
        .write_stdin("a\nb\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs.len(), 2);
    assert!(recs[0]["hmac"].is_string());
}

#[test]
fn batch_base64_encode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["base64", "encode", "--batch"])
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs[0]["encoded"], "aGVsbG8=");
    assert_eq!(recs[1]["encoded"], "d29ybGQ=");
}

#[test]
fn batch_base64_decode_mixed_good_and_bad_exits_3() {
    // One good line, one undecodable line. Every line still gets a record;
    // exit code is 3 because at least one failed.
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["base64", "decode", "--batch"])
        .write_stdin("aGVsbG8=\n!!!notbase64!!!\n")
        .assert()
        .failure()
        .code(3)
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(
        recs.len(),
        2,
        "every line gets a record even when one fails"
    );
    assert_eq!(recs[0]["decoded"], "hello");
    assert!(
        recs[1]["error"].is_string(),
        "bad line carries an error field"
    );
}

#[test]
fn batch_url_encode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["url", "encode", "--batch"])
        .write_stdin("a b\nc&d\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs.len(), 2);
    assert!(recs[0]["encoded"].as_str().unwrap().contains("%20"));
}

#[test]
fn batch_html_encode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "encode", "--batch"])
        .write_stdin("<b>\n&\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs[0]["encoded"], "&lt;b&gt;");
}

#[test]
fn batch_skips_empty_lines() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["hash", "sha256", "--batch"])
        .write_stdin("alice\n\n\nbob\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recs = lines(out);
    assert_eq!(recs.len(), 2, "blank lines produce no records");
}
