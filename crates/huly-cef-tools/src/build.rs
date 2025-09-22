use std::{fs, path::PathBuf};

use anyhow::Result;
use cef_ui_util::{get_cef_artifacts_dir, get_cef_workspace_dir, AppBundleSettings, BuildCommand};
use clap::Parser;

#[cfg(target_os = "linux")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-linux-x86_64.zip";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-arm64.zip";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-x86_64.zip";
#[cfg(target_os = "windows")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-windows-x86_64.zip";

#[derive(Parser, Default)]
struct BuildArgs {
    #[arg(long, default_value_t = String::from("dev"))]
    pub profile: String,

    #[arg(long, default_value_t = String::from(""))]
    pub target: String,
}

fn main() -> Result<()> {
    let artifacts_dir = get_cef_artifacts_dir()?;
    let dir = PathBuf::from(artifacts_dir);
    if !dir.exists() {
        download_and_extract_cef(&dir)?;
    }

    let args = BuildArgs::parse();
    let workspace_dir = get_cef_workspace_dir()?;

    let target = if args.target.is_empty() {
        None
    } else {
        Some(args.target.clone())
    };

    // Build the main executable.
    BuildCommand {
        binary: String::from("huly-cef-websockets"),
        profile: args.profile.to_string(),
        target: target.clone(),
    }
    .run()?;

    if let Some(target) = target.clone() {
        let from = get_exe_path(&args.profile, &target, "huly-cef-websockets")?;
        let to = workspace_dir.join(format!("target/{}/huly-cef-websockets", args.profile));
        println!("Copying executable from {:?} to {:?}", from, to);
        fs::copy(from, to)?;
    }

    // If on macOS, we need to do some extra work.
    if cfg!(target_os = "macos") {
        // Build the helper executable.
        BuildCommand {
            binary: String::from("huly-cef-helper"),
            profile: args.profile.to_string(),
            target: target.clone(),
        }
        .run()?;

        if let Some(target) = target.clone() {
            let from = get_exe_path(&args.profile, &target, "huly-cef-helper")?;
            let to = workspace_dir.join(format!("target/{}/huly-cef-helper", args.profile));
            println!("Copying executable from {:?} to {:?}", from, to);
            fs::copy(from, to)?;
        }

        // Build the app bundle.
        AppBundleSettings {
            profile: args.profile.to_string(),
            artifacts_dir: get_cef_artifacts_dir()?,
            app_name: String::from("huly-cef-websockets"),
            main_exe_name: String::from("huly-cef-websockets"),
            helper_exe_name: String::from("huly-cef-helper"),
            resources_dir: workspace_dir.join("resources/macos"),
            org_name: String::from("huly"),
        }
        .run()?;
    }

    copy_resources(&args.profile)?;

    Ok(())
}

fn download_and_extract_cef(dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(dir)?;

    println!("Downloading CEF from {}", CEF_URL);

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

fn copy_resources(profile: &str) -> Result<()> {
    let profile = if profile == "release" {
        "release"
    } else {
        "debug"
    };

    let workspace_dir = get_cef_workspace_dir()?;
    let resources_dir = workspace_dir.join("resources/pages");

    #[cfg(target_os = "linux")]
    let target_dir = workspace_dir.join(format!("target/{}/cef/huly-cef-resources", profile));
    #[cfg(target_os = "macos")]
    let target_dir = workspace_dir.join(format!(
        "target/{}/huly-cef-websockets.app/Contents/Resources",
        profile
    ));
    #[cfg(target_os = "windows")]
    let target_dir = workspace_dir.join(format!("target/{}/huly-cef-resources", profile));

    dircpy::copy_dir(resources_dir, target_dir)?;

    Ok(())
}

fn get_exe_path(profile: &str, target: &str, name: &str) -> Result<PathBuf> {
    let workspace_dir = get_cef_workspace_dir()?;
    let profile = if profile == "release" {
        "release"
    } else {
        "debug"
    };
    Ok(workspace_dir.join(format!("target/{}/{}/{}", target, profile, name)))
}
