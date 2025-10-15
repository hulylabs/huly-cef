use anyhow::Result;
use cef_ui::{App, CefTask, Context, LogSeverity, MainArgs, Settings};
use cef_ui_sys::{cef_api_hash, cef_quit_message_loop, CEF_API_VERSION_13800};
use std::{fs::create_dir_all, path::PathBuf};

pub type CefContext = cef_ui::Context;

pub fn new(port: u16, cache_path: String) -> Result<CefContext> {
    unsafe {
        cef_api_hash(CEF_API_VERSION_13800 as i32, 0);
    }

    let cache_dir = PathBuf::from(cache_path.clone());
    if !cache_dir.exists() {
        create_dir_all(&cache_dir)?;
    }

    let log_file = cache_dir.join("cef.log");
    if !log_file.exists() {
        std::fs::File::create(&log_file)?;
    }

    let main_args = MainArgs::new()?;
    let settings = Settings::new()
        .log_severity(LogSeverity::Verbose)
        .log_file(&log_file)?
        .cache_path(&cache_dir)?
        .no_sandbox(true)
        .windowless_rendering_enabled(true);

    let app = App::new(crate::application::HulyAppCallbacks::new(port, cache_path));
    let context = Context::new(main_args, settings, Some(app));

    Ok(context)
}

pub fn close() {
    cef_ui::post_task(
        cef_ui::ThreadId::UI,
        CefTask::new(GenericTaskCallbacks::new(|| {
            unsafe { cef_quit_message_loop() };
        })),
    );
}

struct GenericTaskCallbacks<F>(Option<F>);
impl<F: FnOnce() + Send + Sync + 'static> cef_ui::CefTaskCallbacks for GenericTaskCallbacks<F> {
    fn execute(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

impl<F: FnOnce() + Send + Sync + 'static> GenericTaskCallbacks<F> {
    fn new(f: F) -> Self {
        GenericTaskCallbacks(Some(f))
    }
}
