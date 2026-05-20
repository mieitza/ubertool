use clap::Args;
use serde::Serialize;

use crate::core::error::{CliError, ErrorCode};
use crate::core::output::{Out, OutputMode};

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Only report whether an update is available; do not install.
    #[arg(long, default_value_t = false)]
    pub check: bool,
}

#[derive(Serialize)]
struct CheckOutput {
    current: String,
    latest: String,
    update_available: bool,
}

#[derive(Serialize)]
struct UpdateOutput {
    previous: String,
    current: String,
    updated: bool,
}

pub fn run(args: UpdateArgs, out: &Out) -> Result<(), CliError> {
    use self_update::backends::github;

    let current = env!("CARGO_PKG_VERSION").to_string();

    if args.check {
        let releases = github::ReleaseList::configure()
            .repo_owner("mieitza")
            .repo_name("ubertool")
            .build()
            .map_err(|e| {
                CliError::new(
                    ErrorCode::IoError,
                    format!("could not query releases: {e}"),
                )
                .retriable()
            })?
            .fetch()
            .map_err(|e| {
                CliError::new(
                    ErrorCode::IoError,
                    format!("could not fetch releases: {e}"),
                )
                .retriable()
            })?;
        let latest = releases
            .first()
            .map(|r| r.version.clone())
            .unwrap_or_else(|| current.clone());
        let update_available = current != latest && !latest.is_empty();
        out.emit_value(&CheckOutput {
            current,
            latest,
            update_available,
        })
    } else {
        let status = github::Update::configure()
            .repo_owner("mieitza")
            .repo_name("ubertool")
            .bin_name("ubertool")
            .current_version(&current)
            .show_download_progress(out.mode != OutputMode::Json)
            .show_output(out.mode != OutputMode::Json)
            .no_confirm(true)
            .build()
            .map_err(|e| {
                CliError::new(
                    ErrorCode::IoError,
                    format!("self_update build failed: {e}"),
                )
                .retriable()
            })?
            .update()
            .map_err(|e| {
                CliError::new(ErrorCode::IoError, format!("update failed: {e}")).retriable()
            })?;
        let new_version = status.version().to_string();
        let updated = !status.uptodate();
        out.emit_value(&UpdateOutput {
            previous: current,
            current: new_version,
            updated,
        })
    }
}
