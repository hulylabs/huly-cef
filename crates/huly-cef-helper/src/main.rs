
use anyhow::{anyhow, Result};
use cef_ui_helper::{cef_main_args_t, MainArgs, ScopedSandbox};
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
            unsafe extern "C" fn(args: *const cef_main_args_t, *mut c_void, *mut c_void) -> c_int
        > = lib.get(b"cef_execute_process")?;

        info!("Executing CEF subprocess ..");

        // Execute the CEF subprocess.
        let ret = cef_execute_process(main_args.as_raw(), null_mut(), null_mut()) as i32;

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
