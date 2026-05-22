//! HMAC — multiplexed verb mirroring `hash`. Requires a `--key`.
//! Output: `{"hmac": "<hex>"}`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use hmac::{Hmac, Mac};
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::hex;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HmacArgs {
    #[command(subcommand)]
    pub verb: HmacVerb,
}

#[derive(Debug, Subcommand)]
pub enum HmacVerb {
    #[command(
        name = "md5",
        long_about = "Compute HMAC-MD5 (legacy / not collision-resistant).\n\nExamples:\n  ubertool hmac md5 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac md5 --key \"secret\" --batch"
    )]
    Md5(HmacRunArgs),
    #[command(
        name = "sha1",
        long_about = "Compute HMAC-SHA1 (legacy).\n\nExamples:\n  ubertool hmac sha1 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha1 --key \"secret\" --batch"
    )]
    Sha1(HmacRunArgs),
    #[command(
        name = "sha224",
        long_about = "Compute HMAC-SHA224.\n\nExamples:\n  ubertool hmac sha224 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha224 --key \"secret\" --batch"
    )]
    Sha224(HmacRunArgs),
    #[command(
        name = "sha256",
        long_about = "Compute HMAC-SHA256.\n\nExamples:\n  ubertool hmac sha256 \"msg\" --key \"secret\"\n  ubertool hmac sha256 --in ./file --key \"secret\" --json\n  printf 'alice\\nbob\\n' | ubertool hmac sha256 --key \"secret\" --batch"
    )]
    Sha256(HmacRunArgs),
    #[command(
        name = "sha384",
        long_about = "Compute HMAC-SHA384.\n\nExamples:\n  ubertool hmac sha384 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha384 --key \"secret\" --batch"
    )]
    Sha384(HmacRunArgs),
    #[command(
        name = "sha512",
        long_about = "Compute HMAC-SHA512.\n\nExamples:\n  ubertool hmac sha512 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha512 --key \"secret\" --batch"
    )]
    Sha512(HmacRunArgs),
    #[command(
        name = "sha3-256",
        long_about = "Compute HMAC-SHA3-256.\n\nExamples:\n  ubertool hmac sha3-256 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha3-256 --key \"secret\" --batch"
    )]
    Sha3_256(HmacRunArgs),
    #[command(
        name = "sha3-512",
        long_about = "Compute HMAC-SHA3-512.\n\nExamples:\n  ubertool hmac sha3-512 \"msg\" --key \"secret\"\n  printf 'alice\\nbob\\n' | ubertool hmac sha3-512 --key \"secret\" --batch"
    )]
    Sha3_512(HmacRunArgs),
}

#[derive(Debug, Args)]
pub struct HmacRunArgs {
    /// Literal input to MAC (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// HMAC secret key (required).
    #[arg(long)]
    pub key: String,
    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

#[derive(Serialize)]
struct HmacOutput {
    hmac: String,
}

#[derive(Debug, Clone, Copy)]
enum Algo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_512,
}

pub fn dispatch(args: HmacArgs, out: &Out) -> Result<(), CliError> {
    let (run_args, algo) = match args.verb {
        HmacVerb::Md5(a) => (a, Algo::Md5),
        HmacVerb::Sha1(a) => (a, Algo::Sha1),
        HmacVerb::Sha224(a) => (a, Algo::Sha224),
        HmacVerb::Sha256(a) => (a, Algo::Sha256),
        HmacVerb::Sha384(a) => (a, Algo::Sha384),
        HmacVerb::Sha512(a) => (a, Algo::Sha512),
        HmacVerb::Sha3_256(a) => (a, Algo::Sha3_256),
        HmacVerb::Sha3_512(a) => (a, Algo::Sha3_512),
    };
    run(run_args, algo, out)
}

fn run(args: HmacRunArgs, algo: Algo, out: &Out) -> Result<(), CliError> {
    if args.batch {
        let key = args.key.clone();
        return crate::core::batch::run_jsonl("hmac", |line| {
            compute(algo, key.as_bytes(), line.as_bytes())
        });
    }
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let mac_hex = compute(algo, args.key.as_bytes(), input.as_bytes())?;
    out.emit_value(&HmacOutput { hmac: mac_hex })
}

fn compute(algo: Algo, key: &[u8], data: &[u8]) -> Result<String, CliError> {
    fn mac<M: Mac + digest::KeyInit>(key: &[u8], data: &[u8]) -> Result<String, CliError> {
        let mut m = <M as digest::KeyInit>::new_from_slice(key)
            .map_err(|_| CliError::new(ErrorCode::UsageError, "HMAC key length is invalid"))?;
        m.update(data);
        Ok(hex::encode(&m.finalize().into_bytes()))
    }
    match algo {
        Algo::Md5 => mac::<Hmac<md5::Md5>>(key, data),
        Algo::Sha1 => mac::<Hmac<sha1::Sha1>>(key, data),
        Algo::Sha224 => mac::<Hmac<sha2::Sha224>>(key, data),
        Algo::Sha256 => mac::<Hmac<sha2::Sha256>>(key, data),
        Algo::Sha384 => mac::<Hmac<sha2::Sha384>>(key, data),
        Algo::Sha512 => mac::<Hmac<sha2::Sha512>>(key, data),
        Algo::Sha3_256 => mac::<Hmac<sha3::Sha3_256>>(key, data),
        Algo::Sha3_512 => mac::<Hmac<sha3::Sha3_512>>(key, data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_sha256_rfc_vector() {
        // HMAC-SHA256("key", "The quick brown fox jumps over the lazy dog")
        let r = compute(
            Algo::Sha256,
            b"key",
            b"The quick brown fox jumps over the lazy dog",
        )
        .unwrap();
        assert_eq!(
            r,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn hmac_sha1_rfc_vector() {
        let r = compute(
            Algo::Sha1,
            b"key",
            b"The quick brown fox jumps over the lazy dog",
        )
        .unwrap();
        assert_eq!(r, "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9");
    }
}
