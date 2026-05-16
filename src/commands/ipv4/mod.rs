//! IPv4 utilities.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod parse;
pub mod range_expand;
pub mod subnet;
pub mod to_ipv6;

#[derive(Debug, Args)]
pub struct Ipv4Args {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Parse an IPv4 address and emit its representations.
    #[command(long_about = "Parse an IPv4 address and emit decimal/hex/binary forms, plus private/loopback/multicast classification.\n\nExamples:\n  ubertool ipv4 parse 192.168.1.1\n  ubertool ipv4 parse 8.8.8.8 --json\n\nExit codes:\n  3   invalid IPv4 address (invalid_ip)")]
    Parse(RunArgs),
    /// Compute subnet info for a CIDR block.
    #[command(long_about = "Compute subnet info for a CIDR block.\n\nExamples:\n  ubertool ipv4 subnet 10.0.0.0/24\n\nExit codes:\n  3   invalid CIDR (invalid_ip)")]
    Subnet(RunArgs),
    /// Expand a CIDR block into a list of addresses (capped by --max).
    #[command(name = "range-expand", long_about = "Expand a CIDR block into a list of addresses, capped by --max (default 1024).\n\nExamples:\n  ubertool ipv4 range-expand 10.0.0.0/30\n  ubertool ipv4 range-expand 192.168.1.0/24 --max 100 --json")]
    RangeExpand(RangeExpandArgs),
    /// Convert IPv4 to IPv4-mapped IPv6 form (::ffff:a.b.c.d).
    #[command(name = "to-ipv6", long_about = "Convert IPv4 to IPv4-mapped IPv6 (::ffff:a.b.c.d) and compatible form.\n\nExamples:\n  ubertool ipv4 to-ipv6 192.168.1.1")]
    ToIpv6(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    pub input: String,
    #[arg(long = "in")]
    pub in_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RangeExpandArgs {
    pub input: String,
    #[arg(long, default_value_t = 1024)]
    pub max: usize,
}

pub fn dispatch(args: Ipv4Args, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Parse(a) => parse::run(a, out),
        Verb::Subnet(a) => subnet::run(a, out),
        Verb::RangeExpand(a) => range_expand::run(a, out),
        Verb::ToIpv6(a) => to_ipv6::run(a, out),
    }
}
