//! MAC address utilities.

use clap::{Args, Subcommand};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod lookup;
pub mod new;

#[derive(Debug, Args)]
pub struct MacArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a random locally-administered MAC address.
    #[command(
        long_about = "Generate a random 48-bit MAC address with the locally-administered bit set (so it can't collide with real vendor MACs).\n\nExamples:\n  ubertool mac new\n  ubertool mac new --json"
    )]
    New,
    /// Look up the vendor for a MAC address from the bundled OUI table.
    #[command(
        long_about = "Look up the vendor for a MAC address from the bundled OUI table. The OUI is the first 24 bits.\n\nThe bundled table covers ~50 common vendors. For full IEEE OUI coverage, future builds will ship a complete CSV.\n\nExamples:\n  ubertool mac lookup F0:F6:1C:00:11:22\n  ubertool mac lookup 00:1c:42:aa:bb:cc --json\n\nExit codes:\n  3   invalid MAC address (invalid_mac)"
    )]
    Lookup(LookupArgs),
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    /// MAC address (colon, hyphen, or dot-separated; case-insensitive).
    pub input: String,
}

pub fn dispatch(args: MacArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::New => new::run(out),
        Verb::Lookup(a) => lookup::run(a, out),
    }
}
