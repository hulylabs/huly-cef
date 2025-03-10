use anyhow::Result;
use cef_ui::{
    App, BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, Client, Context, LogSeverity,
    MainArgs, RenderHandler, Settings, WindowInfo,
};
use crossbeam_channel::Sender;
use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{level_filters::LevelFilter, subscriber::set_global_default, Level};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

mod application_callbacks;
mod client_callbacks;
mod context_menu_callbacks;
mod lifespan_callbacks;
mod render_callbacks;
mod render_process_callbacks;

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

/// Retrieves the root cache directory for CEF and ensures its existence.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
fn get_root_cache_dir() -> Result<PathBuf> {
    let path = PathBuf::from("/tmp/cef");
    if !path.exists() {
        create_dir_all(&path)?;
    }
    Ok(path)
}

pub struct Browser {
    pub inner: cef_ui::Browser,
    pub state: Arc<Mutex<BrowserState>>,
}

/// Maintains the state of a browser instance.
pub struct BrowserState {
    /// The URL of the browser.
    pub url: String,
    /// The width of the browser in pixels.
    pub width: u32,
    /// The height of the browser in pixels.
    pub height: u32,
    /// The sender for transmitting rendered frames.
    pub tx: UnboundedSender<Vec<u8>>,
    /// Whether the browser is active or not.
    pub active: bool,
}

/// Creates a browser in the UI thread.
///
/// # Parameters
///
/// - `width`: The width of the browser.
/// - `height`: The height of the browser.
/// - `url`: The URL to load in the browser.
/// - `tx`: A sender for frame buffers.
///
/// # Returns
///
/// A new instance of a CEF browser.
fn create_browser_in_ui_thread(
    width: u32,
    height: u32,
    url: &str,
    tx: UnboundedSender<Vec<u8>>,
) -> Browser {
    let window_info = WindowInfo::new().windowless_rendering_enabled(true);
    let settings = BrowserSettings::new().windowless_frame_rate(60);
    let state = Arc::new(Mutex::new(BrowserState {
        url: url.to_string(),
        width,
        height,
        tx,
        active: true,
    }));
    let render_handler = RenderHandler::new(render_callbacks::MyRenderHandlerCallbacks::new(
        state.clone(),
    ));
    let client = Client::new(client_callbacks::MyClientCallbacks::new(render_handler));
    let inner = BrowserHost::create_browser_sync(&window_info, client, url, &settings, None, None);

    Browser {
        inner,
        state: state.clone(),
    }
}

/// A task for creating a browser asynchronously.
struct CreateBrowserTaskCallback {
    tx: Sender<Browser>,
    width: u32,
    height: u32,
    url: String,
    sender: UnboundedSender<Vec<u8>>,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    /// Executes the task to create a browser and send it through the channel.
    fn execute(&mut self) {
        let browser =
            create_browser_in_ui_thread(self.width, self.height, &self.url, self.sender.clone());
        self.tx.send(browser).expect("failed to send a browser");
    }
}

/// Creates a new browser instance asynchronously.
///
/// # Parameters
///
/// - `width`: The width of the browser.
/// - `height`: The height of the browser.
/// - `url`: The URL to load in the browser.
/// - `sender`: A sender for frame buffers.
///
/// # Returns
///
/// A new instance of a CEF browser.
///
/// # Panics
///
/// This function will panic if it fails to create a browser in the UI thread.
pub fn create_browser(
    width: u32,
    height: u32,
    url: &str,
    sender: UnboundedSender<Vec<u8>>,
) -> Browser {
    let (tx, rx) = crossbeam_channel::unbounded::<Browser>();
    let result = cef_ui::post_task(
        cef_ui::ThreadId::UI,
        CefTask::new(CreateBrowserTaskCallback {
            tx,
            width,
            height,
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
