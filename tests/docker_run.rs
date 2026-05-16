use assert_cmd::Command;

#[test]
fn docker_run_basic_image() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["docker-run", "to-compose", "docker run nginx"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("nginx"), "expected image in output: {s}");
    assert!(s.contains("image: nginx"));
}

#[test]
fn docker_run_with_name_and_ports() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "docker-run",
            "to-compose",
            "docker run --name web -p 8080:80 nginx",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("web:") || s.contains("container_name: web"));
    assert!(s.contains("8080:80"));
}

#[test]
fn docker_run_with_env_and_volume() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "docker-run",
            "to-compose",
            "docker run -e DEBUG=1 -e PORT=3000 -v /data:/var/lib/data postgres",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("DEBUG=1"));
    assert!(s.contains("/data:/var/lib/data"));
    assert!(s.contains("postgres"));
}

#[test]
fn docker_run_json_mode_wraps() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "docker-run", "to-compose", "docker run nginx"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    let yaml = v["compose"].as_str().expect("compose field");
    assert!(yaml.contains("nginx"));
}

#[test]
fn docker_run_missing_image_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["docker-run", "to-compose", "docker run --name foo"])
        .assert()
        .failure()
        .code(3);
}
