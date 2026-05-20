use lopdf::{Document, Object};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::verify::verify_signature;
use super::SignatureArgs;

#[derive(Serialize)]
struct SignatureInfo {
    signer: Option<String>,
    signing_time: Option<String>,
    location: Option<String>,
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest_algorithm: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    signature_count: usize,
    signatures: Vec<SignatureInfo>,
}

pub fn run(args: SignatureArgs, out: &Out) -> Result<(), CliError> {
    // Read raw bytes first if verification is needed (before lopdf parsing,
    // which doesn't preserve the exact original byte layout required for
    // ByteRange-based digest verification).
    let raw_bytes: Option<Vec<u8>> = if args.verify {
        match std::fs::read(&args.in_path) {
            Ok(b) => Some(b),
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                if msg.contains("not found") || msg.contains("no such file") {
                    return Err(CliError::new(ErrorCode::FileNotFound, format!("{e}")));
                } else {
                    return Err(CliError::new(
                        ErrorCode::InvalidPdf,
                        format!("could not read PDF: {e}"),
                    )
                    .with_hint("ensure the file is a valid PDF"));
                }
            }
        }
    } else {
        None
    };

    let doc = Document::load(&args.in_path).map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("not found") || msg.contains("no such file") {
            CliError::new(ErrorCode::FileNotFound, format!("{e}"))
        } else {
            CliError::new(ErrorCode::InvalidPdf, format!("could not load PDF: {e}"))
                .with_hint("ensure the file is a valid PDF")
        }
    })?;

    let mut signatures = Vec::new();
    for (_, obj) in doc.objects.iter() {
        if let Object::Dictionary(d) = obj {
            let is_sig = d
                .get(b"Type")
                .ok()
                .and_then(|v| v.as_name().ok())
                .map(|n| n == b"Sig")
                .unwrap_or(false);
            if !is_sig {
                continue;
            }
            let signer = d
                .get(b"Name")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let signing_time = d
                .get(b"M")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let location = d
                .get(b"Location")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());
            let reason = d
                .get(b"Reason")
                .ok()
                .and_then(|v| v.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned());

            let (verified, digest_algorithm, cert_signer) = if args.verify {
                // Extract /ByteRange — array of 4 integers.
                let byte_range: Option<[i64; 4]> = d
                    .get(b"ByteRange")
                    .ok()
                    .and_then(|v| v.as_array().ok())
                    .and_then(|arr| {
                        if arr.len() >= 4 {
                            Some([
                                arr[0].as_i64().ok()?,
                                arr[1].as_i64().ok()?,
                                arr[2].as_i64().ok()?,
                                arr[3].as_i64().ok()?,
                            ])
                        } else {
                            None
                        }
                    });

                // Extract /Contents — lopdf decodes hex strings to raw bytes.
                let contents: Option<Vec<u8>> = d
                    .get(b"Contents")
                    .ok()
                    .and_then(|v| v.as_str().ok())
                    .map(|b| b.to_vec());

                match (byte_range, contents, raw_bytes.as_deref()) {
                    (Some(br), Some(cms), Some(pdf)) => {
                        let result = verify_signature(pdf, &cms, &br);
                        (
                            result.verified,
                            Some(result.digest_algorithm),
                            result.signer_subject,
                        )
                    }
                    _ => (Some(false), None, None),
                }
            } else {
                (None, None, None)
            };

            // When --verify is on and we got a signer from the cert, prefer it
            // over the /Name field (cert subject is more authoritative).
            let effective_signer = if args.verify {
                cert_signer.or(signer)
            } else {
                signer
            };

            signatures.push(SignatureInfo {
                signer: effective_signer,
                signing_time,
                location,
                reason,
                verified,
                digest_algorithm,
            });
        }
    }
    out.emit_value(&Out0 {
        signature_count: signatures.len(),
        signatures,
    })
}
