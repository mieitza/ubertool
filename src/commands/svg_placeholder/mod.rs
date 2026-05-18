//! SVG placeholder image generator.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::core::error::CliError;
use crate::core::output::Out;

#[derive(Debug, Args)]
pub struct SvgPlaceholderArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a placeholder SVG image.
    #[command(
        long_about = "Generate a placeholder SVG image with a centered label.\n\nUseful for prototyping when you need a sized image but no real content.\n\nExamples:\n  ubertool svg-placeholder generate --width 800 --height 600\n  ubertool svg-placeholder generate --text 'Logo'\n  ubertool svg-placeholder generate --width 100 --height 100 --bg '#333' --fg '#fff' --json"
    )]
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(long, default_value_t = 400)]
    pub width: u32,
    #[arg(long, default_value_t = 300)]
    pub height: u32,
    /// Centered text label (default: "<W>x<H>").
    #[arg(long)]
    pub text: Option<String>,
    /// Background color (any CSS color).
    #[arg(long, default_value = "#cccccc")]
    pub bg: String,
    /// Foreground (text) color.
    #[arg(long, default_value = "#333333")]
    pub fg: String,
}

#[derive(Serialize)]
struct Out0 {
    svg: String,
}

pub fn dispatch(args: SvgPlaceholderArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => run(a, out),
    }
}

fn run(args: GenerateArgs, out: &Out) -> Result<(), CliError> {
    let label = args
        .text
        .unwrap_or_else(|| format!("{}x{}", args.width, args.height));
    let font_size = (args.width.min(args.height) / 8).max(12);
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <text x="50%" y="50%" font-family="sans-serif" font-size="{fs}" fill="{fg}" text-anchor="middle" dominant-baseline="middle">{label}</text>
</svg>"#,
        w = args.width,
        h = args.height,
        bg = args.bg,
        fg = args.fg,
        fs = font_size,
        label = escape_xml(&label),
    );
    out.emit_value(&Out0 { svg })
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
