use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn mac_new_format() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["mac", "new"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let s = s.trim();
    let parts: Vec<&str> = s.split(':').collect();
    assert_eq!(parts.len(), 6);
    for p in &parts {
        assert_eq!(p.len(), 2);
        assert!(p.chars().all(|c| c.is_ascii_hexdigit()));
    }
    let first = u8::from_str_radix(parts[0], 16).unwrap();
    assert_eq!(first & 0x02, 0x02, "locally administered bit must be set");
}

#[test]
fn mac_new_two_calls_differ() {
    fn one() -> String {
        let out = Command::cargo_bin("ubertool")
            .unwrap()
            .args(["mac", "new"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(out).unwrap().trim().to_string()
    }
    assert_ne!(one(), one());
}

#[test]
fn mac_lookup_apple() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "mac", "lookup", "F0:F6:1C:00:11:22"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["vendor"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("apple"));
}

#[test]
fn mac_lookup_unknown_oui_emits_null_vendor() {
    // 02:00:00 is in the locally-administered space — never assigned by IEEE.
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "mac", "lookup", "02:00:00:00:00:00"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(v["vendor"].is_null());
}

#[test]
fn mac_lookup_covers_many_vendors() {
    // Spot-check well-known OUIs that must be in the full IEEE registry.
    for (mac, vendor_substr) in [
        ("F0:F6:1C:00:00:00", "apple"),
        ("00:00:0C:00:00:00", "cisco"),
        ("00:50:56:00:00:00", "vmware"),
        ("DC:A6:32:00:00:00", "raspberry"),
    ] {
        let out = Command::cargo_bin("ubertool")
            .unwrap()
            .args(["--json", "mac", "lookup", mac])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        let got = v["vendor"].as_str().unwrap_or("").to_lowercase();
        assert!(
            got.contains(vendor_substr),
            "OUI {mac}: expected '{vendor_substr}', got '{got}'"
        );
    }
}

#[test]
fn mac_lookup_invalid_mac_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["mac", "lookup", "not-a-mac"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_mac"));
}
