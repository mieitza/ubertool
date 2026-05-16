//! Token-level parser for `docker run` argument vectors.

use crate::core::error::{CliError, ErrorCode};

#[derive(Debug, Default)]
pub struct DockerRun {
    pub name: Option<String>,
    pub image: Option<String>,
    pub command: Vec<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
    pub restart: Option<String>,
    pub network: Option<String>,
}

pub fn parse(input: &str) -> Result<DockerRun, CliError> {
    let mut tokens = shlex_split(input).ok_or_else(|| {
        CliError::new(
            ErrorCode::InvalidDockerRun,
            "could not tokenize input (unclosed quote?)",
        )
    })?;
    if tokens.first().map(|s| s.as_str()) == Some("docker") {
        tokens.remove(0);
    }
    if tokens.first().map(|s| s.as_str()) == Some("run") {
        tokens.remove(0);
    }

    let mut dr = DockerRun::default();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "--name" => {
                i += 1;
                dr.name = Some(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("--name"))?,
                );
            }
            "-p" | "--publish" => {
                i += 1;
                dr.ports.push(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("-p"))?,
                );
            }
            "-v" | "--volume" => {
                i += 1;
                dr.volumes.push(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("-v"))?,
                );
            }
            "-e" | "--env" => {
                i += 1;
                dr.env.push(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("-e"))?,
                );
            }
            "--restart" => {
                i += 1;
                dr.restart = Some(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("--restart"))?,
                );
            }
            "--network" => {
                i += 1;
                dr.network = Some(
                    tokens
                        .get(i)
                        .cloned()
                        .ok_or_else(|| flag_needs_value("--network"))?,
                );
            }
            "-d" | "--detach" | "-i" | "-t" | "-it" | "--interactive" | "--tty" | "--rm" => {
                // Ignored — compose handles these declaratively.
            }
            _ if t.starts_with("--") || (t.starts_with('-') && t.len() == 2) => {
                // Unknown flag — skip it and its value (if next token isn't another flag).
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    i += 1;
                }
            }
            _ => {
                if dr.image.is_none() {
                    dr.image = Some(t.clone());
                } else {
                    dr.command.push(t.clone());
                }
            }
        }
        i += 1;
    }

    if dr.image.is_none() {
        return Err(CliError::new(
            ErrorCode::InvalidDockerRun,
            "missing image — `docker run` requires an image positional",
        )
        .with_hint("docker run [flags] <image> [command]"));
    }
    Ok(dr)
}

fn flag_needs_value(flag: &str) -> CliError {
    CliError::new(
        ErrorCode::InvalidDockerRun,
        format!("flag '{flag}' requires a value"),
    )
}

fn shlex_split(s: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut chars = s.chars();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !buf.is_empty() {
                    out.push(std::mem::take(&mut buf));
                }
            }
            c => buf.push(c),
        }
    }
    if in_single || in_double {
        return None;
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal() {
        let dr = parse("docker run nginx").unwrap();
        assert_eq!(dr.image.as_deref(), Some("nginx"));
    }

    #[test]
    fn parses_name_and_ports() {
        let dr = parse("docker run --name web -p 8080:80 nginx").unwrap();
        assert_eq!(dr.name.as_deref(), Some("web"));
        assert_eq!(dr.ports, vec!["8080:80".to_string()]);
        assert_eq!(dr.image.as_deref(), Some("nginx"));
    }

    #[test]
    fn parses_env_and_volume() {
        let dr = parse("docker run -e A=1 -e B=2 -v /a:/b nginx").unwrap();
        assert_eq!(dr.env, vec!["A=1".to_string(), "B=2".to_string()]);
        assert_eq!(dr.volumes, vec!["/a:/b".to_string()]);
    }

    #[test]
    fn rejects_missing_image() {
        let err = parse("docker run --name foo").unwrap_err();
        assert!(err.message.contains("missing image"));
    }
}
