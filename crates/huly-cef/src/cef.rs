use anyhow::Result;
use cef_ui::{App, Context, LogSeverity, MainArgs, Settings};
use std::{fs::create_dir_all, path::PathBuf};

mod application;
mod client;

pub mod browser;
pub mod messages;

/// Represents the CEF context.
pub type CefContext = cef_ui::Context;

/// Initializes and returns a new CEF context.
///
/// # Errors
///
/// Returns an error if initialization fails.
pub fn new(port: u16, cache_path: String) -> Result<CefContext> {
    let cache_dir = PathBuf::from(cache_path.clone());
    if !cache_dir.exists() {
        create_dir_all(&cache_dir)?;
    }

    let main_args = MainArgs::new()?;
    let settings = Settings::new()
        .log_severity(LogSeverity::Verbose)
        .cache_path(&cache_dir)?
        .no_sandbox(false);

    let app = App::new(application::HulyAppCallbacks::new(port, cache_path));
    let context = Context::new(main_args, settings, Some(app));

    Ok(context)
}
