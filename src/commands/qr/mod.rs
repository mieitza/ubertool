//! QR code generation (SVG/PNG) including WiFi URIs.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::core::error::CliError;
use crate::core::output::Out;

pub mod generate;
pub mod wifi;

#[derive(Debug, Args)]
pub struct QrArgs {
    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Generate a QR code from arbitrary text.
    #[command(long_about = "Generate a QR code from arbitrary text.\n\nFormats:\n  svg (default) — emitted to stdout (or --out path)\n  png — requires --out path or non-TTY stdout (binary)\n\nExamples:\n  ubertool qr generate 'https://example.com'\n  ubertool qr generate 'hello' --format png --out qr.png\n  ubertool qr generate 'hi' --json")]
    Generate(generate::GenerateArgs),
    /// Generate a QR code encoding a Wi-Fi network URI.
    #[command(long_about = "Generate a QR code that, when scanned by a mobile camera, prompts to join a Wi-Fi network.\n\nEncodes WIFI:T:<security>;S:<ssid>;P:<password>;H:<hidden>;;\n\nExamples:\n  ubertool qr wifi --ssid 'MyNet' --password 'pw'\n  ubertool qr wifi --ssid 'Public' --security nopass\n  ubertool qr wifi --ssid 'Hidden' --password 'pw' --hidden --out wifi.png --format png")]
    Wifi(wifi::WifiArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Svg,
    Png,
}

pub fn dispatch(args: QrArgs, out: &Out) -> Result<(), CliError> {
    match args.verb {
        Verb::Generate(a) => generate::run(a, out),
        Verb::Wifi(a) => wifi::run(a, out),
    }
}

pub(super) fn render(
    data: &str,
    format: Format,
    out_path: Option<&PathBuf>,
    out: &Out,
) -> Result<(), CliError> {
    use crate::core::error::ErrorCode;
    use qrcode::QrCode;
    use serde::Serialize;

    let code = QrCode::new(data.as_bytes()).map_err(|e| {
        CliError::new(ErrorCode::Internal, format!("QR encode failed: {e}"))
            .with_hint("input may be too long or contain unencodable bytes")
    })?;

    match format {
        Format::Svg => {
            use qrcode::render::svg;
            let svg_str = code
                .render::<svg::Color>()
                .min_dimensions(200, 200)
                .build();
            if let Some(path) = out_path {
                std::fs::write(path, &svg_str).map_err(CliError::from)?;
                return Ok(());
            }
            #[derive(Serialize)]
            struct Out0 {
                svg: String,
            }
            out.emit_value(&Out0 { svg: svg_str })
        }
        Format::Png => {
            let width = code.width();
            // Build a bool matrix from the QR code.
            let pixels: Vec<Vec<bool>> = (0..width)
                .map(|y| (0..width).map(|x| code[(x, y)] == qrcode::Color::Dark).collect())
                .collect();
            let scale: u32 = 8;
            let border: u32 = 4;
            let inner = width as u32;
            let total = (inner + border * 2) * scale;
            let mut img = image::GrayImage::from_pixel(total, total, image::Luma([255u8]));
            for (y, row) in pixels.iter().enumerate() {
                for (x, &dark) in row.iter().enumerate() {
                    if dark {
                        let px = (x as u32 + border) * scale;
                        let py = (y as u32 + border) * scale;
                        for dy in 0..scale {
                            for dx in 0..scale {
                                img.put_pixel(px + dx, py + dy, image::Luma([0u8]));
                            }
                        }
                    }
                }
            }
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
                .map_err(|e| CliError::new(ErrorCode::Internal, format!("PNG encode: {e}")))?;
            out.emit_binary(&bytes, out_path.map(|p| p.as_path()))
        }
    }
}
