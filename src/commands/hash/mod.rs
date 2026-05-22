//! Hashing — multiplexed verb. The algorithm name is the subcommand
//! (e.g. `ubertool hash sha256`). Output: `{"hash": "<hex>"}`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::hex;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

#[derive(Debug, Args)]
pub struct HashArgs {
    #[command(subcommand)]
    pub verb: HashVerb,
}

#[derive(Debug, Subcommand)]
pub enum HashVerb {
    #[command(
        name = "md5",
        long_about = "Compute MD5 hash (legacy / not collision-resistant).\n\nExamples:\n  ubertool hash md5 \"hello\"\n  ubertool hash md5 --in ./file.bin\n  echo -n hello | ubertool hash md5\n  printf 'alice\\nbob\\n' | ubertool hash md5 --batch"
    )]
    Md5(HashRunArgs),
    #[command(
        name = "sha1",
        long_about = "Compute SHA-1 hash (legacy / not collision-resistant).\n\nExamples:\n  ubertool hash sha1 \"hello\"\n  ubertool hash sha1 --in ./file.bin\n  printf 'alice\\nbob\\n' | ubertool hash sha1 --batch"
    )]
    Sha1(HashRunArgs),
    #[command(
        name = "sha224",
        long_about = "Compute SHA-224 hash.\n\nExamples:\n  ubertool hash sha224 \"hello\"\n  printf 'alice\\nbob\\n' | ubertool hash sha224 --batch"
    )]
    Sha224(HashRunArgs),
    #[command(
        name = "sha256",
        long_about = "Compute SHA-256 hash.\n\nExamples:\n  ubertool hash sha256 \"hello\"\n  ubertool hash sha256 --in ./file.bin --json\n  printf 'alice\\nbob\\n' | ubertool hash sha256 --batch"
    )]
    Sha256(HashRunArgs),
    #[command(
        name = "sha384",
        long_about = "Compute SHA-384 hash.\n\nExamples:\n  ubertool hash sha384 \"hello\"\n  printf 'alice\\nbob\\n' | ubertool hash sha384 --batch"
    )]
    Sha384(HashRunArgs),
    #[command(
        name = "sha512",
        long_about = "Compute SHA-512 hash.\n\nExamples:\n  ubertool hash sha512 \"hello\"\n  printf 'alice\\nbob\\n' | ubertool hash sha512 --batch"
    )]
    Sha512(HashRunArgs),
    #[command(
        name = "sha3-256",
        long_about = "Compute SHA3-256 hash (Keccak).\n\nExamples:\n  ubertool hash sha3-256 \"hello\"\n  printf 'alice\\nbob\\n' | ubertool hash sha3-256 --batch"
    )]
    Sha3_256(HashRunArgs),
    #[command(
        name = "sha3-512",
        long_about = "Compute SHA3-512 hash (Keccak).\n\nExamples:\n  ubertool hash sha3-512 \"hello\"\n  printf 'alice\\nbob\\n' | ubertool hash sha3-512 --batch"
    )]
    Sha3_512(HashRunArgs),
}

#[derive(Debug, Args)]
pub struct HashRunArgs {
    /// Literal input to hash (omit to read from --in or stdin).
    pub input: Option<String>,
    /// Read input from file.
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
    /// Read one item per line from stdin and emit JSONL (one record per line).
    #[arg(long)]
    pub batch: bool,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
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

pub fn dispatch(args: HashArgs, out: &Out) -> Result<(), CliError> {
    let (run_args, algo) = match args.verb {
        HashVerb::Md5(a) => (a, Algo::Md5),
        HashVerb::Sha1(a) => (a, Algo::Sha1),
        HashVerb::Sha224(a) => (a, Algo::Sha224),
        HashVerb::Sha256(a) => (a, Algo::Sha256),
        HashVerb::Sha384(a) => (a, Algo::Sha384),
        HashVerb::Sha512(a) => (a, Algo::Sha512),
        HashVerb::Sha3_256(a) => (a, Algo::Sha3_256),
        HashVerb::Sha3_512(a) => (a, Algo::Sha3_512),
    };
    run(run_args, algo, out)
}

fn run(args: HashRunArgs, algo: Algo, out: &Out) -> Result<(), CliError> {
    if args.batch {
        return crate::core::batch::run_jsonl("hash", |line| Ok(compute(algo, line.as_bytes())));
    }
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let digest = compute(algo, input.as_bytes());
    out.emit_value(&HashOutput { hash: digest })
}

fn compute(algo: Algo, data: &[u8]) -> String {
    use digest::Digest;
    match algo {
        Algo::Md5 => hex::encode(&md5::Md5::digest(data)),
        Algo::Sha1 => hex::encode(&sha1::Sha1::digest(data)),
        Algo::Sha224 => hex::encode(&sha2::Sha224::digest(data)),
        Algo::Sha256 => hex::encode(&sha2::Sha256::digest(data)),
        Algo::Sha384 => hex::encode(&sha2::Sha384::digest(data)),
        Algo::Sha512 => hex::encode(&sha2::Sha512::digest(data)),
        Algo::Sha3_256 => hex::encode(&sha3::Sha3_256::digest(data)),
        Algo::Sha3_512 => hex::encode(&sha3::Sha3_512::digest(data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_hello() {
        assert_eq!(
            compute(Algo::Sha256, b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn md5_of_empty() {
        assert_eq!(compute(Algo::Md5, b""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn sha1_of_hello() {
        assert_eq!(
            compute(Algo::Sha1, b"hello"),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
    }
}
