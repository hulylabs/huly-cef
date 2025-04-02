use anyhow::Result;
use cef_ui::{App, Context, LogSeverity, MainArgs, Settings};
use std::{fs::create_dir_all, path::PathBuf};
use tracing::{level_filters::LevelFilter, subscriber::set_global_default, Level};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

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
pub fn new() -> Result<CefContext, anyhow::Error> {
    LogTracer::init()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::ERROR))
        .finish();

    set_global_default(subscriber)?;

    let root_cache_dir = get_root_cache_dir()?;

    let main_args = MainArgs::new()?;
    let settings = Settings::new()
        .log_severity(LogSeverity::Error)
        .root_cache_path(&root_cache_dir)?
        .no_sandbox(false);

    let app = App::new(application::HulyAppCallbacks::new());
    let context = Context::new(main_args, settings, Some(app));

    Ok(context)
}

/// Retrieves the root cache directory for CEF and ensures its existence.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
fn get_root_cache_dir() -> Result<PathBuf> {
    let path = PathBuf::from("/home/nikita/repos/cefcache");
    if !path.exists() {
        create_dir_all(&path)?;
    }
    Ok(path)
}
