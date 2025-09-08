use std::{process::exit, ptr::null_mut};

use anyhow::Result;
use cef_ui::{App, AppCallbacks, MainArgs, RenderProcessHandler};
use cef_ui_helper::ScopedSandbox;
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
    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        Some(self.render_process_handler.clone())
    }
}
