use std::{fs, path::PathBuf};

use anyhow::Result;
use cef_ui_util::{get_cef_artifacts_dir, get_cef_workspace_dir, AppBundleSettings, BuildCommand};
use clap::Parser;
use tracing::{level_filters::LevelFilter, subscriber::set_global_default, Level};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

#[cfg(target_os = "linux")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-linux-x86_64.zip";
#[cfg(target_os = "macos")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-arm64.zip";
#[cfg(target_os = "windows")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-windows-x86_64.zip";

#[derive(Parser, Default)]
struct BuildArgs {
    #[arg(long, default_value_t = String::from("dev"))]
    pub profile: String,
}

fn main() -> Result<()> {
    LogTracer::init()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::INFO))
        .finish();

    set_global_default(subscriber)?;

    let artifacts_dir = get_cef_artifacts_dir()?;
    let dir = PathBuf::from(artifacts_dir);
    if !dir.exists() {
        download_and_extract_cef(&dir)?;
    }

    let args = BuildArgs::parse();
    let workspace_dir = get_cef_workspace_dir()?;

    // Build the main executable.
    BuildCommand {
        binary: String::from("huly-cef-websockets"),
        profile: args.profile.to_string(),
    }
    .run()?;

    // If on macOS, we need to do some extra work.
    if cfg!(target_os = "macos") {
        // Build the helper executable.
        BuildCommand {
            binary: String::from("huly-cef-helper"),
            profile: args.profile.to_string(),
        }
        .run()?;

        // Build the app bundle.
        AppBundleSettings {
            profile: args.profile.to_string(),
            artifacts_dir: get_cef_artifacts_dir()?,
            app_name: String::from("huly-cef-websockets"),
            main_exe_name: String::from("huly-cef-websockets"),
            helper_exe_name: String::from("huly-cef-helper"),
            resources_dir: workspace_dir.join("cef-resources/macos"),
            org_name: String::from("huly"),
        }
        .run()?;
    }

    Ok(())
}

fn download_and_extract_cef(dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(dir)?;

    let result = download_cef(CEF_URL).and_then(|data| {
        zip::ZipArchive::new(std::io::Cursor::new(data))?.extract(&dir)?;
        Ok(())
    });

    if result.is_err() {
        _ = fs::remove_dir_all(dir);
        return Err(anyhow::anyhow!(
            "Failed to download CEF artifacts. Try again"
        ));
    }

    Ok(())
}

fn download_cef(url: &str) -> Result<Vec<u8>> {
    Ok(reqwest::blocking::get(url)?.bytes()?.to_vec())
}
