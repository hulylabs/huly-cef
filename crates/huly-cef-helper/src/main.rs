use std::{env::current_exe, ffi::{c_int, c_void}, fs::canonicalize, path::PathBuf, process::exit, ptr::null_mut};

use anyhow::Result;
use anyhow::anyhow;
use cef_ui::{AppCallbacks, MainArgs, RenderProcessHandler, RenderProcessHandlerCallbacks};
use cef_ui_helper::ScopedSandbox;
use cef_ui_sys::{cef_app_t, cef_main_args_t};
use libloading::{Library, Symbol};
use tracing::{error, info, level_filters::LevelFilter, subscriber::set_global_default, Level};
use tracing_log::{ LogTracer};
use tracing_subscriber::FmtSubscriber;

fn main() {
    run(true);
}

/// The relative path to the CEF framework library within the app bundle on macOS.
const CEF_PATH: &str = "../../../Chromium Embedded Framework.framework/Chromium Embedded Framework";

/// Returns the CEF error code or 1 if an error occurred.
pub fn run(sandbox: bool) {
    let ret = try_run(sandbox).unwrap_or_else(|e| {
        error!("An error occurred: {}", e);

        1
    });

    info!("The return code is: {}", ret);

    exit(ret);
}

/// Try and run the helper, returning the CEF error code if successful.
fn try_run(sandbox: bool) -> Result<i32> {
    // This routes log macros through tracing.
    LogTracer::init()?;

    // Setup the tracing subscriber globally.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::DEBUG))
        .finish();

    set_global_default(subscriber)?;

    // Setup the sandbox if enabled.
    let _sandbox = match sandbox {
        true => Some(ScopedSandbox::new()?),
        false => None
    };

    // Manually load CEF and execute the subprocess.
    let ret = unsafe {
        // Load our main args.
        let main_args = MainArgs::new()?;

        info!("Main args: {:?}", main_args);

        // Manually load the CEF framework.
        let cef_path = get_cef_path(CEF_PATH)?;
        let lib = Library::new(cef_path)?;


        let cef_execute_process: Symbol<
            unsafe extern "C" fn(args: *const cef_main_args_t, *mut cef_app_t, *mut c_void) -> c_int
        > = lib.get(b"cef_execute_process")?;

        info!("Executing CEF subprocess ..");

        let app = cef_ui::App::new(HelperAppCallbacks);

        // Execute the CEF subprocess.
        let ret = cef_execute_process(main_args.as_raw(), app.into_raw(), null_mut()) as i32;

        info!("CEF exited with code: {}", ret);

        // Close the CEF framework.
        lib.close()?;

        info!("Closed CEF library.");

        ret
    };

    Ok(ret)
}

/// Get the cef library path.
fn get_cef_path(relative_path: &str) -> Result<PathBuf> {
    let cef_path = current_exe()?
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("Could not get parent directory"))?;
    let cef_path = cef_path.join(relative_path);
    let cef_path = canonicalize(cef_path)?;

    Ok(cef_path)
}

struct HelperAppCallbacks;

impl AppCallbacks for HelperAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        _: Option<&str>,
        _: Option<cef_ui::CommandLine>
    ) {
        
    }

    fn get_browser_process_handler(&mut self) -> Option<cef_ui::BrowserProcessHandler> {
        None
    }

    fn get_render_process_handler(&mut self) -> Option<cef_ui::RenderProcessHandler> {
        Some(RenderProcessHandler::new(RenderProcessCallbacks))
    }
}

struct RenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        info!("[on_web_kit_initialized]");
    }

    fn on_browser_created(&mut self, browser: cef_ui::Browser, extra_info: Option<cef_ui::DictionaryValue>) {
    }

    fn on_browser_destroyed(&mut self, browser: cef_ui::Browser) {
    }

    fn on_context_created(&mut self, browser: cef_ui::Browser, frame: cef_ui::Frame, context: cef_ui::V8Context) {
    }

    fn on_process_message_received(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        source_process: cef_ui::ProcessId,
        message: &mut cef_ui::ProcessMessage
    ) -> bool {
        true
    }
}