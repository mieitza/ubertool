use lopdf::{Document, Object};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::Out;

use super::SignatureArgs;

#[derive(Serialize)]
struct SignatureInfo {
    signer: Option<String>,
    signing_time: Option<String>,
    location: Option<String>,
    reason: Option<String>,
}

#[derive(Serialize)]
struct Out0 {
    signature_count: usize,
    signatures: Vec<SignatureInfo>,
}

pub fn run(args: SignatureArgs, out: &Out) -> Result<(), CliError> {
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
            signatures.push(SignatureInfo {
                signer,
                signing_time,
                location,
                reason,
            });
        }
    }
    out.emit_value(&Out0 {
        signature_count: signatures.len(),
        signatures,
    })
}
