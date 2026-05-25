//! Integration tests for `vault` — encrypted local secrets store.
//!
//! All tests use:
//!   --vault-file <tempfile>    so no real vault is touched
//!   UBERTOOL_VAULT_PASSWORD   so no interactive prompt
//!   UBERTOOL_VAULT_NO_KEYRING=1  so keyring is bypassed

use assert_cmd::Command;

/// Build a Command for ubertool with the vault test environment.
///
/// Sets UBERTOOL_VAULT_NO_KEYRING=1 unconditionally.
/// Additional env vars can be passed as (key, value) pairs.
fn ut(env: &[(&str, &str)], args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("ubertool").unwrap();
    cmd.env_clear()
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        )
        .env(
            "PATH",
            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
        )
        .env("UBERTOOL_VAULT_NO_KEYRING", "1");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.args(args);
    cmd
}

#[test]
fn vault_init_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    ut(
        &[("UBERTOOL_VAULT_PASSWORD", "pw")],
        &["vault", "--vault-file"],
    )
    .arg(&vault)
    .arg("init")
    .assert()
    .success();
    assert!(vault.exists(), "vault file should be created");
    // First 8 bytes are the magic.
    let bytes = std::fs::read(&vault).unwrap();
    assert_eq!(&bytes[..8], b"UBERVLT1", "magic bytes must match");
}

#[test]
fn vault_init_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    // Second init without --force should fail with exit 2.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn vault_init_force_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn vault_set_get_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("api-key")
        .arg("sekret-value")
        .assert()
        .success();
    let out = ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("api-key")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim_end(), "sekret-value");
}

#[test]
fn vault_get_json_mode() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("v")
        .assert()
        .success();
    let out = ut(&env, &["--json", "vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("k")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["name"], "k");
    assert_eq!(parsed["value"], "v");
}

#[test]
fn vault_get_missing_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("nope")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn vault_wrong_password_exits_5() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    ut(
        &[("UBERTOOL_VAULT_PASSWORD", "right")],
        &["vault", "--vault-file"],
    )
    .arg(&vault)
    .arg("init")
    .assert()
    .success();
    ut(
        &[("UBERTOOL_VAULT_PASSWORD", "wrong")],
        &["vault", "--vault-file"],
    )
    .arg(&vault)
    .arg("get")
    .arg("any")
    .assert()
    .failure()
    .code(5);
}

#[test]
fn vault_list_never_emits_values() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("do-not-leak")
        .assert()
        .success();
    let out = ut(&env, &["--json", "vault", "--vault-file"])
        .arg(&vault)
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains("do-not-leak"), "list leaked a value: {s}");
    // But should contain the key name.
    assert!(s.contains("k"), "list should contain key name: {s}");
}

#[test]
fn vault_list_text_mode_one_name_per_line() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("alpha")
        .arg("v1")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("beta")
        .arg("v2")
        .assert()
        .success();
    let out = ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Should not contain values.
    assert!(!s.contains("v1"), "list leaked value v1");
    assert!(!s.contains("v2"), "list leaked value v2");
    // Should contain names.
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines.contains(&"alpha"));
    assert!(lines.contains(&"beta"));
}

#[test]
fn vault_delete_requires_yes_non_interactive() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("v")
        .assert()
        .success();
    // No --yes, non-TTY (assert_cmd doesn't use a TTY) → exit 2.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("delete")
        .arg("k")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn vault_delete_with_yes_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("v")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("delete")
        .arg("k")
        .arg("--yes")
        .assert()
        .success();
    // After delete, get should exit 3.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("k")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn vault_delete_not_found_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("delete")
        .arg("nonexistent")
        .arg("--yes")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn vault_export_to_file() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("mykey")
        .arg("myvalue")
        .assert()
        .success();
    let outfile = dir.path().join("export.json");
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("export")
        .arg("--out")
        .arg(&outfile)
        .assert()
        .success();
    let body = std::fs::read_to_string(&outfile).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        v.get("secrets").is_some(),
        "export JSON must have 'secrets' key"
    );
    assert_eq!(v["secrets"]["mykey"]["value"], "myvalue");
}

#[test]
fn vault_export_import_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let vault1 = dir.path().join("v1.enc");
    let vault2 = dir.path().join("v2.enc");
    let dump = dir.path().join("dump.json");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault1)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault1)
        .arg("set")
        .arg("k")
        .arg("v")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault1)
        .arg("export")
        .arg("--out")
        .arg(&dump)
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault2)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault2)
        .arg("import")
        .arg(&dump)
        .assert()
        .success();
    let out = ut(&env, &["vault", "--vault-file"])
        .arg(&vault2)
        .arg("get")
        .arg("k")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim_end(), "v");
}

#[test]
fn vault_import_merge_skips_collision() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let dump = dir.path().join("dump.json");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("original")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("export")
        .arg("--out")
        .arg(&dump)
        .assert()
        .success();
    // Import into same vault — "k" already exists, should be skipped.
    let out = ut(&env, &["--json", "vault", "--vault-file"])
        .arg(&vault)
        .arg("import")
        .arg(&dump)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["skipped"], 1);
    assert_eq!(parsed["imported"], 0);
}

#[test]
fn vault_import_replace_overwrites_collision() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let dump = dir.path().join("dump.json");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("original")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("export")
        .arg("--out")
        .arg(&dump)
        .assert()
        .success();
    // Update k to a new value.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("new-value")
        .assert()
        .success();
    // Import the old dump with --replace — should overwrite.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("import")
        .arg(&dump)
        .arg("--replace")
        .assert()
        .success();
    let out = ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("k")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim_end(), "original");
}

#[test]
fn vault_unlock_then_get_without_password() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("session");
    std::fs::create_dir_all(&session_dir).unwrap();
    let vault = dir.path().join("v.enc");

    let env_init: &[(&str, &str)] = &[
        ("UBERTOOL_VAULT_PASSWORD", "pw"),
        ("UBERTOOL_VAULT_SESSION_DIR", session_dir.to_str().unwrap()),
    ];
    ut(env_init, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    ut(env_init, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("k")
        .arg("v")
        .assert()
        .success();
    ut(env_init, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("unlock")
        .arg("--ttl")
        .arg("1")
        .assert()
        .success();

    // Now remove the password env. Session cache should provide it.
    let env_no_pw: &[(&str, &str)] =
        &[("UBERTOOL_VAULT_SESSION_DIR", session_dir.to_str().unwrap())];
    let out = ut(env_no_pw, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("k")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim_end(), "v");

    // Lock the vault.
    ut(env_no_pw, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("lock")
        .assert()
        .success();

    // After lock, get without password should fail.
    ut(env_no_pw, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("get")
        .arg("k")
        .assert()
        .failure();
}

#[test]
fn vault_lock_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("session");
    std::fs::create_dir_all(&session_dir).unwrap();
    let vault = dir.path().join("v.enc");
    let env = [
        ("UBERTOOL_VAULT_PASSWORD", "pw"),
        ("UBERTOOL_VAULT_SESSION_DIR", session_dir.to_str().unwrap()),
    ];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    // Lock twice — both should succeed.
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("lock")
        .assert()
        .success();
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("lock")
        .assert()
        .success();
}

#[test]
fn vault_set_output_never_echoes_value() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("v.enc");
    let env = [("UBERTOOL_VAULT_PASSWORD", "pw")];
    ut(&env, &["vault", "--vault-file"])
        .arg(&vault)
        .arg("init")
        .assert()
        .success();
    let out = ut(&env, &["--json", "vault", "--vault-file"])
        .arg(&vault)
        .arg("set")
        .arg("mykey")
        .arg("super-secret-xyz-987")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("super-secret-xyz-987"),
        "set output must never echo the value: {s}"
    );
    // Output should have name, created_at, updated_at.
    let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed["name"], "mykey");
    assert!(parsed.get("created_at").is_some());
    assert!(parsed.get("updated_at").is_some());
}
