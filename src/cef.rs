use anyhow::Result;

use cef_ui::{
    App, BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, Client, Context, LogSeverity,
    MainArgs, RenderHandler, Settings, WindowInfo,
};
use crossbeam_channel::Sender;

use core::panic;
use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::{level_filters::LevelFilter, subscriber::set_global_default, Level};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

pub mod application_callbacks;
pub mod client_callbacks;
mod context_menu_callbacks;
mod lifespan_callbacks;
mod render_callbacks;
mod render_process_callbacks;

pub type CefContext = cef_ui::Context;

pub struct BrowserState {
    pub width: u32,
    pub height: u32,
    pub tx: Sender<Buffer>,
}

pub struct Buffer {
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

pub fn new() -> Result<CefContext, anyhow::Error> {
    LogTracer::init()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::DEBUG))
        .finish();

    set_global_default(subscriber)?;

    let root_cache_dir = get_root_cache_dir()?;

    let main_args = MainArgs::new()?;
    let settings = Settings::new()
        .log_severity(LogSeverity::Error)
        .root_cache_path(&root_cache_dir)?
        .no_sandbox(false);

    let app = App::new(application_callbacks::MyAppCallbacks {});
    let context = Context::new(main_args, settings, Some(app));

    Ok(context)
}

fn get_root_cache_dir() -> Result<PathBuf> {
    let path = PathBuf::from("/tmp/cef-ui-simple");
    if !path.exists() {
        create_dir_all(&path)?;
    }

    Ok(path)
}

fn create_browser_in_ui_thread(
    width: u32,
    height: u32,
    url: &str,
    tx: Sender<Buffer>,
) -> cef_ui::Browser {
    let window_info = WindowInfo::new().windowless_rendering_enabled(true);
    let browser_settings = BrowserSettings::new().windowless_frame_rate(30);

    let browser_state = Arc::new(Mutex::new(BrowserState {
        //browser: None,
        width: width,
        height: height,
        tx: tx,
    }));

    let render_handler = RenderHandler::new(render_callbacks::MyRenderHandlerCallbacks::new(
        browser_state.clone(),
    ));
    let client = Client::new(client_callbacks::MyClientCallbacks::new(render_handler));

    let browser =
        BrowserHost::create_browser_sync(&window_info, client, url, &browser_settings, None, None);

    return browser;
}

struct CreateBrowserTaskCallback {
    tx: Sender<cef_ui::Browser>,
    width: u32,
    height: u32,
    url: String,
    sender: Sender<Buffer>,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    fn execute(&mut self) {
        let browser =
            create_browser_in_ui_thread(self.width, self.height, &self.url, self.sender.clone());
        self.tx.send(browser).expect("failed to send a browser");
    }
}

pub fn create_browser(
    width: u32,
    height: u32,
    url: &str,
    sender: Sender<Buffer>,
) -> cef_ui::Browser {
    let (tx, rx) = crossbeam_channel::unbounded::<cef_ui::Browser>();
    let result = cef_ui::post_task(
        cef_ui::ThreadId::UI,
        CefTask::new(CreateBrowserTaskCallback {
            tx,
            width: width,
            height: height,
            url: url.to_string(),
            sender,
        }),
    );

    if !result {
        panic!("failed to create a browser in the UI thread");
    }

    rx.recv()
        .expect("failed to receive a CEF browser, created in the UI thread")
}
