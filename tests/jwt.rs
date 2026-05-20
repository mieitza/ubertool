use assert_cmd::Command;
use predicates::prelude::*;

// Valid HS256 token: header={"alg":"HS256","typ":"JWT"}, claims={"sub":"1234567890","name":"John Doe","iat":1516239022}
// Signed with secret="secret". (The jwt.io example token uses a different signing key.)
const HS256_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.XbPfbIHMI6arZ3Y922BhjWgQzWXcXNrz0ogtVhfEd2o";

// Test-only RSA 2048-bit keypair in PKCS#8/SPKI PEM format.
// These keys are used ONLY for unit tests and carry no security value.
const TEST_RSA_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDTz4BREg/+3PEL
/F+X+E+g95wrMgskCqO0f9iaiLppheFeKAwakv1VrdJz6X80ig6mRSPlXfWgR/Hf
QNwj3d0HuZEb6jEfLDR/mZaxE62EfCimT82RAwMRT4/LK36N7rj1KdT+TRdpu/5d
Mg9ajO73S0V4xJ6/jrDaoyGeqCozA57JVITc99mjGqJZnFkWE3oa+IzZCl64GD1O
k8Y3/JHFAQzTB29KS79crpYBg6Rxj8171AcgelHpDFzA5DC+8lm5idtIUoDgiK2Q
jV5XJx/a5GerpZxCjFGOaZJ5622Y4QVZoMaLP9Yd/33CHb3U+S6jEPdpGIDPgWoX
rWA8D9gpAgMBAAECggEAAkqbY8UjQSmYvf1z90vraJ1lJh57el9XGAvOBxX1llCp
eIBMaTkv3m5r+W8MPkBEGk5jLgcPMjW1CypDd2veCUhbpoapE2JCCyNZOv8wgF9r
fSkd2zTCIrzOnG8TBmwPui9Cq6Gf4Df1b4KsIdmmeCxrSvwtj3r2tNbQ9UtycWzS
jsRuH2N2PDGxj9bJObu8C9ca7kkzVlVAXy//bQJi8ubl1OKCRaaLOhRQQ6RwpEaH
YCA+yUSLBfVA+TlU8npUGLUEf6i8JYHj1B4W/zDegtJOdR+y4pFdfRWaYCMCsv6j
KsjRkJlFZ+1FpAaG4d8xLc3fSgtZNTSi24iOSTKA1QKBgQDzxA1BpBumhNqDjfGr
jJrgCAjcv9neIALuEDagwoiYhTX0JPxq7TzJNlK0NRDDAp/gOLxB2c94PD6ig2Y8
5JV7z/JhFLKeO1Rsdy5o15VRLrRV3MEIgUHAclleANLTUUMSsNAlA7FPB0A35HdV
vOFE1yuhSVD2gV3FD2ZOKqeHYwKBgQDecOHXU7rBgTW8wLV8mtEk4Q29URuyOrbQ
tb7GfowuOP0FQq6GpyUH+nldqYgUzuu4TyUpeAPuEz99aFlsKJ2nBEUUIpHfq1RE
tQW7oB1J6OnrvYFHePyODwgYZ0OhI2NlU/fcmDmskaC86YE/3wRlpT4gWsPWpwTu
RMnYBkpWAwKBgQDYpZe6jSXd+xlR7mepNc+36KwntYLmfcDc4CANBJfuJgZrtCt6
xNpPYi5i5v68sqpw1zvJstFHZROtz+afm/CXF5utWH3fT4+aztm6aE/W1RGFditB
ac+MQJTYjRbSpNhd1jNHkl6VhDY49p5w3uu/CRVHxSdlzlgOeb9NHW/DTQKBgFRb
jupwUIUOEWkR1EH7HJHV37YKSmyfI7GnG0B/wX3OEvapwTJDtSF/dgN1RaxOhMqT
CU7euu7Q0eUna7I+/PrX/bKTIasneH/N/uyW3kSnVf8XES4fzfe576IBl35Gw20s
GPXJrSWq3MCH9YJZe3to3VS710VSdeY+pCWOzSYtAoGBAIUr3dfAbR/OoDnAiHyH
ZvgnnvyXFWub2K54WRDxoK6vnGPiZHYhow2kh1g4EYYHZY+OgNVhacybX3/gejsJ
nkdXvR6fjJUCb/vcbchdx2Ps+OP/DREuImvn6oCa4/KVKrFwYyAKFZaLg0KdY7mJ
hl7gnuSlshUWyZs0BgatskXH
-----END PRIVATE KEY-----";

const TEST_RSA_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA08+AURIP/tzxC/xfl/hP
oPecKzILJAqjtH/Ymoi6aYXhXigMGpL9Va3Sc+l/NIoOpkUj5V31oEfx30DcI93d
B7mRG+oxHyw0f5mWsROthHwopk/NkQMDEU+Pyyt+je649SnU/k0Xabv+XTIPWozu
90tFeMSev46w2qMhnqgqMwOeyVSE3PfZoxqiWZxZFhN6GviM2QpeuBg9TpPGN/yR
xQEM0wdvSku/XK6WAYOkcY/Ne9QHIHpR6QxcwOQwvvJZuYnbSFKA4IitkI1eVycf
2uRnq6WcQoxRjmmSeettmOEFWaDGiz/WHf99wh291PkuoxD3aRiAz4FqF61gPA/Y
KQIDAQAB
-----END PUBLIC KEY-----";

// A second RSA public key (different from the one above) used to verify wrong-key rejection.
// Generated with a separate openssl invocation; never corresponds to TEST_RSA_PRIVATE_PEM.
const TEST_RSA_OTHER_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsnTu3EOdGHAN8FWxbu36
30SW6AnNfONqztfNIRxXGa7fHYS44YaYieMuAGMyavR4IJPwqhiSPawvvfK1Jz2z
/bm0l2QqcyzpzJynFlG5q2W8v9T8RSJ4z079Yt/+dGanWw7iZmQgAEo+F5rDlhCx
NBG/cjvyieq2XbykFLcQsWIsWsimpFI4oKz8cQRsJgy3El7geiQPQEx60+QfkO2R
p9/j0HruhtYq64vl/Qs7oqqW8i3X1DnOQTD4T8GwK59YliEkBNLZYQ9ELbUoIC9R
fq20ld2PWL0ObEsKXA1jofTrlXTtX/afJkguoASeA7UZBlu0krkT4PSZnefndD/p
iQIDAQAB
-----END PUBLIC KEY-----";

#[test]
fn jwt_decode_emits_header_and_claims() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "jwt", "decode", HS256_TOKEN])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["header"]["alg"], "HS256");
    assert_eq!(v["header"]["typ"], "JWT");
    assert_eq!(v["claims"]["sub"], "1234567890");
    assert_eq!(v["claims"]["name"], "John Doe");
}

#[test]
fn jwt_decode_malformed_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "decode", "not.a.jwt"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}

#[test]
fn jwt_verify_correct_secret_succeeds() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success();
}

#[test]
fn jwt_verify_correct_secret_emits_claims_json() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
    assert_eq!(v["claims"]["sub"], "1234567890");
}

#[test]
fn jwt_verify_wrong_secret_exits_5() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "wrong"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn jwt_verify_secret_is_redacted_from_json_error_input() {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "jwt",
            "verify",
            HS256_TOKEN,
            "--secret",
            "wrong-secret-do-not-leak",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(
        !s.contains("wrong-secret-do-not-leak"),
        "secret leaked in JSON error: {s}"
    );
}

#[test]
fn jwt_verify_malformed_token_exits_3() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", "garbage", "--secret", "secret"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid_jwt"));
}

// ── RS256 tests ──────────────────────────────────────────────────────────────

/// Sign a JWT with the test RSA private key (via jsonwebtoken) and return the token string.
fn make_rs256_token() -> String {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    let claims = serde_json::json!({"sub": "test-rs256", "iat": 1700000000_i64});
    encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_PEM.as_bytes())
            .expect("test RSA private key must be valid"),
    )
    .expect("encoding must succeed")
}

#[test]
fn jwt_verify_rs256_with_key_file() {
    let token = make_rs256_token();
    let pub_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(pub_file.path(), TEST_RSA_PUBLIC_PEM).unwrap();

    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "--json",
            "jwt",
            "verify",
            &token,
            "--algo",
            "rs256",
            "--key-file",
            pub_file.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["verified"], true);
    assert_eq!(v["claims"]["sub"], "test-rs256");
}

#[test]
fn jwt_verify_rs256_wrong_key_exits_5() {
    let token = make_rs256_token();
    let other_pub_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(other_pub_file.path(), TEST_RSA_OTHER_PUBLIC_PEM).unwrap();

    Command::cargo_bin("ubertool")
        .unwrap()
        .args([
            "jwt",
            "verify",
            &token,
            "--algo",
            "rs256",
            "--key-file",
            other_pub_file.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("signature_mismatch"));
}

#[test]
fn jwt_verify_hs256_still_works() {
    // Regression: make sure the HS256 path is unchanged after adding RS/ES support.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--secret", "secret"])
        .assert()
        .success();
}

#[test]
fn jwt_verify_rs256_missing_key_file_exits_2() {
    let token = make_rs256_token();
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", &token, "--algo", "rs256"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("usage_error"));
}

#[test]
fn jwt_verify_hs256_missing_secret_exits_2() {
    // --secret is now optional, so omitting it with an HS algo must be a usage error.
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["jwt", "verify", HS256_TOKEN, "--algo", "hs256"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("usage_error"));
}
