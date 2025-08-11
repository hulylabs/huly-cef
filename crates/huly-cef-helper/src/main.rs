
use anyhow::{anyhow, Result};
use cef_ui::{AppCallbacks, RenderProcessHandlerCallbacks};
use cef_ui_helper::{cef_main_args_t, MainArgs, ScopedSandbox};
use cef_ui_sys::cef_app_t;
use libloading::{Library, Symbol};
use log::{error, info, SetLoggerError};
use log4rs::{append::{console::ConsoleAppender, file::FileAppender}, config::{Appender, Root}, encode::pattern::PatternEncoder, Config};
use std::{
    env::current_exe,
    ffi::{c_int, c_void},
    fs::canonicalize,
    path::PathBuf,
    process::exit,
    ptr::null_mut
};


/// The relative path to the CEF framework library within the app bundle on macOS.
const CEF_PATH: &str = "../../../Chromium Embedded Framework.framework/Chromium Embedded Framework";

fn setup_logging() -> Result<log4rs::Handle, SetLoggerError> {
    let stdout_pattern = "\x1b[90m{d(%H:%M:%S%.3f)} \x1b[0m{h({l})} \x1b[90m{f}:{L} \x1b[0m{m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(stdout_pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config)
}

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
    setup_logging()?;

    // Setup the sandbox if enabled.
    let _sandbox = match sandbox {
        true => Some(ScopedSandbox::new()?),
        false => None
    };

    let app = cef_ui::App::new(HelperAppCallbacks {});

    // Manually load CEF and execute the subprocess.
    let ret = unsafe {
        // Load our main args.
        let main_args = MainArgs::new()?;

        info!("Main args: {:?}", main_args);

        // Manually load the CEF framework.
        let cef_path = get_cef_path(CEF_PATH)?;
        let lib = Library::new(cef_path)?;

        // Manually load the cef_execute_process function.
        let cef_execute_process: Symbol<
            unsafe extern "C" fn(args: *const cef_main_args_t, *mut cef_app_t, *mut c_void) -> c_int
        > = lib.get(b"cef_execute_process")?;

        info!("Executing CEF subprocess ..");

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


fn main() {
    run(true);
}


struct HelperRenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for HelperRenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        info!("on_web_kit_initialized");
    }

    fn on_browser_created(&mut self, browser: cef_ui::Browser, extra_info: Option<cef_ui::DictionaryValue>) {
        info!("on_browser_created");
    }

    fn on_browser_destroyed(&mut self, browser: cef_ui::Browser) {
        info!("on_browser_destroyed");
    }

    fn on_context_created(&mut self, browser: cef_ui::Browser, frame: cef_ui::Frame, context: cef_ui::V8Context) {
        info!("on_context_created");
    }

    fn on_process_message_received(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        source_process: cef_ui::ProcessId,
        message: &mut cef_ui::ProcessMessage
    ) -> bool {
        info!("on_process_message_received");
        // Handle the message here if needed.
        false // Return true if the message was handled, false otherwise.
    }
}
struct HelperAppCallbacks;

impl AppCallbacks for HelperAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        process_type: Option<&str>,
        command_line: Option<cef_ui::CommandLine>
    ) {
        info!("on_before_command_line_processing");
    }

    fn get_browser_process_handler(&mut self) -> Option<cef_ui::BrowserProcessHandler> {
        info!("get_browser_process_handler");
        None
    }

    fn get_render_process_handler(&mut self) -> Option<cef_ui::RenderProcessHandler> {
        Some(cef_ui::RenderProcessHandler::new(HelperRenderProcessCallbacks {}))
    }
}