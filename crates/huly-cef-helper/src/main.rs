use std::{process::exit, ptr::null_mut, mem::zeroed, ffi::c_char};

use anyhow::Result;
use cef_ui::{App, AppCallbacks, MainArgs, RenderProcessHandler, SchemeOptions, SchemeRegistrar};
use cef_ui_helper::ScopedSandbox;
use cef_ui_sys::cef_string_t;
use log::{error, info, SetLoggerError};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::render_process::RenderProcessCallbacks;

mod cef_lib;
mod js;
mod render_process;

unsafe fn new_cef_string(s: &str) -> cef_string_t {
    let lib = &cef_lib::CEFLIB;
    let mut ret: cef_string_t = unsafe { zeroed() };

    if s.is_empty() {
        return ret;
    }

    match (lib.cef_string_utf8_to_utf16)(s.as_ptr() as *const c_char, s.len(), &mut ret) {
        0 => panic!("Failed to convert from utf8 to utf16!"),
        _ => ret
    }
}

trait SchemeRegistrarExt {
    fn add_custom_scheme_raw(&self, scheme_name: &str, options: i32) -> Result<bool>;
}

impl SchemeRegistrarExt for SchemeRegistrar {
    fn add_custom_scheme_raw(&self, scheme_name: &str, options: i32) -> Result<bool> {
        unsafe {
            if let Some(add_custom_scheme) = (*self.0).add_custom_scheme {
                Ok(add_custom_scheme(self.0, &new_cef_string(scheme_name), options) != 0)
            } else {
                Err(anyhow::anyhow!("add_custom_scheme is None"))
            }
        }
    }
}

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

fn main() -> Result<()> {
    setup_logging()?;
    let ret = match run() {
        Ok(code) => code,
        Err(e) => {
            error!("An error occurred: {}", e);
            1
        }
    };

    info!("The return code is: {}", ret);
    exit(ret);
}

fn run() -> Result<i32> {
    let _sandbox = ScopedSandbox::new()?;
    unsafe {
        let main_args = MainArgs::new()?;
        let app = App::new(HelperAppCallbacks::new());
        let lib = &cef_lib::CEFLIB;
        Ok((lib.cef_execute_process)(
            main_args.as_raw(),
            app.into_raw(),
            null_mut(),
        ))
    }
}

struct HelperAppCallbacks {
    render_process_handler: RenderProcessHandler,
}

impl HelperAppCallbacks {
    fn new() -> Self {
        Self {
            render_process_handler: RenderProcessHandler::new(RenderProcessCallbacks),
        }
    }
}



impl AppCallbacks for HelperAppCallbacks {
    fn on_register_custom_schemes(&mut self, registrar: SchemeRegistrar) {
        let _ = registrar.add_custom_scheme_raw("huly", SchemeOptions::Local.into());
    }

    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        Some(self.render_process_handler.clone())
    }
}
