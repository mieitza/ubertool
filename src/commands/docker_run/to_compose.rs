use serde::Serialize;

use crate::core::error::CliError;
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::parser::{parse, DockerRun};
use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    compose: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let dr = parse(input.as_str()?)?;
    let yaml = render(&dr);
    out.emit_value(&Out0 { compose: yaml })
}

fn render(dr: &DockerRun) -> String {
    let service_name = dr.name.clone().unwrap_or_else(|| {
        dr.image
            .as_deref()
            .unwrap_or("app")
            .rsplit('/')
            .next()
            .unwrap_or("app")
            .split(':')
            .next()
            .unwrap_or("app")
            .to_string()
    });
    let mut s = String::new();
    s.push_str("services:\n");
    s.push_str(&format!("  {service_name}:\n"));
    if let Some(image) = &dr.image {
        s.push_str(&format!("    image: {image}\n"));
    }
    if let Some(name) = &dr.name {
        s.push_str(&format!("    container_name: {name}\n"));
    }
    if let Some(restart) = &dr.restart {
        s.push_str(&format!("    restart: {restart}\n"));
    }
    if !dr.ports.is_empty() {
        s.push_str("    ports:\n");
        for p in &dr.ports {
            s.push_str(&format!("      - \"{p}\"\n"));
        }
    }
    if !dr.volumes.is_empty() {
        s.push_str("    volumes:\n");
        for v in &dr.volumes {
            s.push_str(&format!("      - {v}\n"));
        }
    }
    if !dr.env.is_empty() {
        s.push_str("    environment:\n");
        for e in &dr.env {
            s.push_str(&format!("      - {e}\n"));
        }
    }
    if let Some(net) = &dr.network {
        s.push_str("    networks:\n");
        s.push_str(&format!("      - {net}\n"));
    }
    if !dr.command.is_empty() {
        let cmd = dr.command.join(" ");
        s.push_str(&format!("    command: {cmd}\n"));
    }
    s
}
