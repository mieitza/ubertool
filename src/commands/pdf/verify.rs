//! Cryptographic verification of PKCS#7/CMS detached PDF signatures.
//!
//! # What is verified
//! For each RSA-signed signature dict:
//!   1. The ByteRange digest is recomputed and compared to the `messageDigest`
//!      signed attribute (integrity — was the PDF modified after signing?).
//!   2. The RSA signature over the DER-encoded signed attributes is verified
//!      against the signer certificate's public key.
//!
//! # What is NOT verified
//! - Certificate chain / trust anchor validation.  A self-signed cert that
//!   correctly signs the PDF reports `verified: true`.  This is an integrity
//!   check ("unmodified since signing"), not a trust check.
//! - ECDSA / DSA signatures — RSA only.  Non-RSA → `verified: null`.
//! - Timestamp / revocation (OCSP/CRL) checking.

use cms::cert::CertificateChoices;
use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerIdentifier};
use der::asn1::ObjectIdentifier;
use der::{Decode, Encode};
use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Sign, RsaPublicKey};
use sha2::{Digest, Sha256, Sha384, Sha512};
use x509_cert::Certificate;

// Well-known OIDs
const OID_SHA256: &str = "2.16.840.1.101.3.4.2.1";
const OID_SHA384: &str = "2.16.840.1.101.3.4.2.2";
const OID_SHA512: &str = "2.16.840.1.101.3.4.2.3";
const OID_SHA1: &str = "1.3.14.3.2.26";
const OID_RSA_ENCRYPTION: &str = "1.2.840.113549.1.1.1";
const OID_SHA256_WITH_RSA: &str = "1.2.840.113549.1.1.11";
const OID_SHA384_WITH_RSA: &str = "1.2.840.113549.1.1.12";
const OID_SHA512_WITH_RSA: &str = "1.2.840.113549.1.1.13";
const OID_SHA1_WITH_RSA: &str = "1.2.840.113549.1.1.5";
// messageDigest attribute OID
const OID_MESSAGE_DIGEST: &str = "1.2.840.113549.1.9.4";

/// Result of verifying one signature.
pub struct VerifyResult {
    /// `Some(true)` — valid; `Some(false)` — invalid; `None` — algorithm not supported.
    pub verified: Option<bool>,
    /// Human-readable digest algorithm name.
    pub digest_algorithm: String,
    /// Signer certificate subject, if available.
    pub signer_subject: Option<String>,
}

/// Verify the cryptographic integrity of one PDF signature.
///
/// `pdf_bytes` — the full raw file bytes.
/// `contents_der` — the `/Contents` value as decoded DER bytes (lopdf gives
/// this directly via `Object::as_str()` on a hex-encoded string).
/// `byte_range` — the four integers `[a, b, c, d]` from `/ByteRange`.
pub fn verify_signature(
    pdf_bytes: &[u8],
    contents_der: &[u8],
    byte_range: &[i64; 4],
) -> VerifyResult {
    let [a, b, c, d] = *byte_range;
    let a = a as usize;
    let b = b as usize;
    let c = c as usize;
    let d = d as usize;

    // Bounds-check ByteRange against the actual file length.
    if a + b > pdf_bytes.len() || c + d > pdf_bytes.len() {
        return VerifyResult {
            verified: Some(false),
            digest_algorithm: "unknown".to_string(),
            signer_subject: None,
        };
    }

    // The /Contents value in a PDF is padded with trailing zero bytes to
    // reserve space.  Trim to the actual DER content length before parsing.
    let contents_der = trim_der(contents_der);

    // Parse the CMS ContentInfo.
    let ci = match ContentInfo::from_der(contents_der) {
        Ok(ci) => ci,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: "unknown".to_string(),
                signer_subject: None,
            }
        }
    };

    // Extract SignedData from ContentInfo.content (wrapped in explicit [0]).
    let sd_bytes = match ci.content.to_der() {
        Ok(b) => b,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: "unknown".to_string(),
                signer_subject: None,
            }
        }
    };
    let sd = match SignedData::from_der(&sd_bytes) {
        Ok(sd) => sd,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: "unknown".to_string(),
                signer_subject: None,
            }
        }
    };

    // Take the first SignerInfo (PDF signatures have exactly one).
    let si = match sd.signer_infos.0.iter().next() {
        Some(si) => si.clone(),
        None => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: "unknown".to_string(),
                signer_subject: None,
            }
        }
    };

    let digest_alg_oid = si.digest_alg.oid.to_string();
    let sig_alg_oid = si.signature_algorithm.oid.to_string();

    let digest_algorithm_name = match digest_alg_oid.as_str() {
        OID_SHA256 => "SHA-256",
        OID_SHA384 => "SHA-384",
        OID_SHA512 => "SHA-512",
        OID_SHA1 => "SHA-1",
        other => other,
    }
    .to_string();

    // Check whether this is an RSA signature.  We accept rsaEncryption (bare
    // RSA) and the sha*WithRSAEncryption OIDs.
    let is_rsa = matches!(
        sig_alg_oid.as_str(),
        OID_RSA_ENCRYPTION
            | OID_SHA256_WITH_RSA
            | OID_SHA384_WITH_RSA
            | OID_SHA512_WITH_RSA
            | OID_SHA1_WITH_RSA
    );
    if !is_rsa {
        // Non-RSA (ECDSA, DSA, …) — not supported, report null.
        let signer_subject = extract_signer_subject(&sd, &si);
        return VerifyResult {
            verified: None,
            digest_algorithm: digest_algorithm_name,
            signer_subject,
        };
    }

    // ------------------------------------------------------------------
    // Step 1: Recompute the ByteRange digest and compare to messageDigest.
    // ------------------------------------------------------------------
    let signed_bytes: Vec<u8> = pdf_bytes[a..a + b]
        .iter()
        .chain(pdf_bytes[c..c + d].iter())
        .copied()
        .collect();

    let computed_digest = match digest_alg_oid.as_str() {
        OID_SHA256 => Sha256::digest(&signed_bytes).to_vec(),
        OID_SHA384 => Sha384::digest(&signed_bytes).to_vec(),
        OID_SHA512 => Sha512::digest(&signed_bytes).to_vec(),
        OID_SHA1 => {
            use sha1::Digest as _;
            sha1::Sha1::digest(&signed_bytes).to_vec()
        }
        _ => {
            return VerifyResult {
                verified: None,
                digest_algorithm: digest_algorithm_name,
                signer_subject: extract_signer_subject(&sd, &si),
            }
        }
    };

    // Extract messageDigest from signed attributes.
    let signed_attrs = match &si.signed_attrs {
        Some(sa) => sa,
        None => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: digest_algorithm_name,
                signer_subject: extract_signer_subject(&sd, &si),
            }
        }
    };

    let message_digest_oid: ObjectIdentifier = OID_MESSAGE_DIGEST.parse().unwrap();
    let expected_digest: Vec<u8> = signed_attrs
        .iter()
        .find(|attr| attr.oid == message_digest_oid)
        .and_then(|attr| attr.values.iter().next())
        .and_then(|any| {
            // The value is an OCTET STRING stored as Any.  decode_as() uses
            // the tag stored in Any to re-decode the value correctly.
            any.decode_as::<der::asn1::OctetString>().ok()
        })
        .map(|os| os.as_bytes().to_vec())
        .unwrap_or_default();

    if computed_digest != expected_digest {
        return VerifyResult {
            verified: Some(false),
            digest_algorithm: digest_algorithm_name,
            signer_subject: extract_signer_subject(&sd, &si),
        };
    }

    // ------------------------------------------------------------------
    // Step 2: Verify the RSA signature over the DER-encoded signed attrs.
    //
    // The signed_attrs field is stored in SignerInfo with an IMPLICIT [0]
    // context tag (0xA0).  The signature covers the same content re-encoded
    // with the universal SET OF tag (0x31).  We get the DER with the context
    // tag from to_der(), then swap byte 0 from 0xA0 to 0x31.
    // ------------------------------------------------------------------
    let sa_der_context_tagged = match signed_attrs.to_der() {
        Ok(bytes) => bytes,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: digest_algorithm_name,
                signer_subject: extract_signer_subject(&sd, &si),
            }
        }
    };

    // Swap tag: 0xA0 (context [0] constructed) → 0x31 (SET OF)
    let mut sa_der_set = sa_der_context_tagged;
    if sa_der_set.first() == Some(&0xA0) {
        sa_der_set[0] = 0x31;
    }
    // (If it already is 0x31 — i.e. the crate re-encodes correctly — we're fine.)

    // Hash the signed-attributes DER.
    let sa_digest = match digest_alg_oid.as_str() {
        OID_SHA256 => Sha256::digest(&sa_der_set).to_vec(),
        OID_SHA384 => Sha384::digest(&sa_der_set).to_vec(),
        OID_SHA512 => Sha512::digest(&sa_der_set).to_vec(),
        OID_SHA1 => {
            use sha1::Digest as _;
            sha1::Sha1::digest(&sa_der_set).to_vec()
        }
        _ => {
            return VerifyResult {
                verified: None,
                digest_algorithm: digest_algorithm_name,
                signer_subject: extract_signer_subject(&sd, &si),
            }
        }
    };

    // Locate the signer certificate.
    let signer_cert = find_signer_cert(&sd, &si);
    let signer_subject = signer_cert
        .as_ref()
        .map(|c| format_name(&c.tbs_certificate.subject));

    let cert = match signer_cert {
        Some(c) => c,
        None => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: digest_algorithm_name,
                signer_subject,
            }
        }
    };

    // Extract RSA public key from SubjectPublicKeyInfo.
    let spki_der = match cert.tbs_certificate.subject_public_key_info.to_der() {
        Ok(d) => d,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: digest_algorithm_name,
                signer_subject,
            }
        }
    };
    let pub_key = match RsaPublicKey::from_public_key_der(&spki_der) {
        Ok(k) => k,
        Err(_) => {
            return VerifyResult {
                verified: Some(false),
                digest_algorithm: digest_algorithm_name,
                signer_subject,
            }
        }
    };

    let sig_bytes = si.signature.as_bytes();
    let scheme = match digest_alg_oid.as_str() {
        OID_SHA256 => Pkcs1v15Sign::new::<Sha256>(),
        OID_SHA384 => Pkcs1v15Sign::new::<Sha384>(),
        OID_SHA512 => Pkcs1v15Sign::new::<Sha512>(),
        OID_SHA1 => {
            use sha1::Sha1;
            Pkcs1v15Sign::new::<Sha1>()
        }
        _ => {
            return VerifyResult {
                verified: None,
                digest_algorithm: digest_algorithm_name,
                signer_subject,
            }
        }
    };

    let verified = pub_key.verify(scheme, &sa_digest, sig_bytes).is_ok();

    VerifyResult {
        verified: Some(verified),
        digest_algorithm: digest_algorithm_name,
        signer_subject,
    }
}

/// Find the signer's X.509 certificate embedded in the SignedData.
fn find_signer_cert(sd: &SignedData, si: &cms::signed_data::SignerInfo) -> Option<Certificate> {
    let certs = sd.certificates.as_ref()?;
    match &si.sid {
        SignerIdentifier::IssuerAndSerialNumber(isn) => {
            for choice in certs.0.iter() {
                if let CertificateChoices::Certificate(cert) = choice {
                    if cert.tbs_certificate.issuer == isn.issuer
                        && cert.tbs_certificate.serial_number == isn.serial_number
                    {
                        return Some(cert.clone());
                    }
                }
            }
        }
        SignerIdentifier::SubjectKeyIdentifier(_ski) => {
            // SubjectKeyIdentifier matching is less common; return first cert as fallback.
            for choice in certs.0.iter() {
                if let CertificateChoices::Certificate(cert) = choice {
                    return Some(cert.clone());
                }
            }
        }
    }
    None
}

/// Extract the signer subject string without requiring a certificate lookup.
fn extract_signer_subject(sd: &SignedData, si: &cms::signed_data::SignerInfo) -> Option<String> {
    find_signer_cert(sd, si).map(|c| format_name(&c.tbs_certificate.subject))
}

/// Format an x509-cert `Name` as a compact RFC 4514-style string.
fn format_name(name: &x509_cert::name::Name) -> String {
    name.to_string()
}

/// Trim a DER-encoded byte slice to the length declared in its outer TLV
/// header, discarding any trailing zero-padding added by PDF signers.
fn trim_der(data: &[u8]) -> &[u8] {
    if data.len() < 2 {
        return data;
    }
    // DER length encoding:
    //   If byte 1 < 0x80: short form — length is byte 1 itself.
    //   If byte 1 == 0x81: one subsequent byte gives the length.
    //   If byte 1 == 0x82: two subsequent bytes give the length.
    //   etc.
    let (header_len, content_len) = if data[1] < 0x80 {
        (2usize, data[1] as usize)
    } else {
        let num_len_bytes = (data[1] & 0x7f) as usize;
        if data.len() < 2 + num_len_bytes {
            return data;
        }
        let mut len = 0usize;
        for &b in &data[2..2 + num_len_bytes] {
            len = (len << 8) | (b as usize);
        }
        (2 + num_len_bytes, len)
    };
    let total = header_len + content_len;
    if total <= data.len() {
        &data[..total]
    } else {
        data
    }
}
