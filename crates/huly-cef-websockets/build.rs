use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Result;
use cef_ui_util::link_cef;

const CEF_ARTIFACTS_DIR: &str = "CEF_ARTIFACTS_DIR";
#[cfg(target_os = "linux")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-linux-x86_64.zip";
#[cfg(target_os = "macos")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-arm64.zip";
#[cfg(target_os = "windows")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-windows-x86_64.zip";

fn main() -> Result<()> {
    let dir = PathBuf::from(std::env::var(CEF_ARTIFACTS_DIR)?);
    if !dir.exists() {
        download_and_extract_cef(&dir)?;
    }
    link_cef()?;

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
