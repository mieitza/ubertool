use assert_cmd::Command;

#[test]
fn html_encode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "encode", "<div class=\"x\">hi & bye</div>"])
        .assert()
        .success()
        .stdout("&lt;div class=&quot;x&quot;&gt;hi &amp; bye&lt;/div&gt;\n");
}

#[test]
fn html_decode_basic() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", "&lt;b&gt;hi&lt;/b&gt; &amp; bye"])
        .assert()
        .success()
        .stdout("<b>hi</b> & bye\n");
}

#[test]
fn html_encode_json_mode() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "html", "encode", "<x>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["encoded"], "&lt;x&gt;");
}

#[test]
fn html_round_trip_via_pipe() {
    let encoded = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "encode", "<i>x & y</i>"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(encoded).unwrap().trim().to_string();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", &s])
        .assert()
        .success()
        .stdout("<i>x & y</i>\n");
}

#[test]
fn html_decode_unknown_entities_passthrough() {
    // Unknown entities are left unchanged by html-escape.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["html", "decode", "&notarealentity; &amp;"])
        .assert()
        .success()
        .stdout("&notarealentity; &\n");
}
