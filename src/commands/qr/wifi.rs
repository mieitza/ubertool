use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

use super::{render, Format};

#[derive(Debug, Args)]
pub struct WifiArgs {
    #[arg(long)]
    pub ssid: String,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long, value_enum, default_value = "wpa")]
    pub security: Security,
    #[arg(long, default_value_t = false)]
    pub hidden: bool,
    #[arg(long, value_enum, default_value = "svg")]
    pub format: Format,
    #[arg(long = "out")]
    pub out_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Security {
    #[value(name = "WPA")]
    Wpa,
    #[value(name = "WEP")]
    Wep,
    #[value(name = "nopass")]
    Nopass,
}

impl Security {
    fn label(self) -> &'static str {
        match self {
            Security::Wpa => "WPA",
            Security::Wep => "WEP",
            Security::Nopass => "nopass",
        }
    }
}

pub fn run(args: WifiArgs, out: &Out) -> Result<(), CliError> {
    let security = args.security.label();
    let password = args.password.unwrap_or_default();
    let hidden = if args.hidden { "true" } else { "false" };
    let escaped_ssid = escape_wifi(&args.ssid);
    let escaped_pw = escape_wifi(&password);
    let uri = format!("WIFI:T:{security};S:{escaped_ssid};P:{escaped_pw};H:{hidden};;");
    render(&uri, args.format, args.out_path.as_ref(), out)
}

fn escape_wifi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | ';' | ',' | '"' | ':' => {
                out.push('\\');
                out.push(c);
            }
            other => out.push(other),
        }
    }
    out
}
