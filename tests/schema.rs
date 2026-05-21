use assert_cmd::Command;

#[test]
fn schema_emits_valid_json_with_info_and_commands() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["schema"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_slice(&out).expect("schema output must be valid JSON");
    assert!(v["info"].is_object(), "schema must have an info block");
    assert!(v["commands"].is_object(), "schema must have a commands map");
    // The full surface has 100+ command entries.
    assert!(v["commands"].as_object().unwrap().len() > 100);
}

#[test]
fn schema_noun_filter_narrows_to_one_noun() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["schema", "--noun", "hash"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let cmds = v["commands"].as_object().unwrap();
    // Every kept key must be a `hash` command.
    assert!(!cmds.is_empty());
    for key in cmds.keys() {
        assert!(
            key.starts_with("ubertool hash "),
            "unexpected key after --noun hash: {key}"
        );
    }
    // hash has the group entry + 8 algorithm leaves = 9 entries.
    assert!(cmds.len() >= 8);
}

#[test]
fn schema_unknown_noun_exits_2() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["schema", "--noun", "nonexistent-noun"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn schema_pipeable_to_jq_style_consumption() {
    // Confirm the top-level shape an agent would rely on.
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["schema", "--noun", "uuid"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["commands"]
        .as_object()
        .unwrap()
        .keys()
        .any(|k| k.contains("uuid new")));
}
