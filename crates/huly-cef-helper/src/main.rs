use std::process::exit;

use anyhow::Result;
use cef_ui_helper::{execute_process, App, AppCallbacks, MainArgs, ScopedSandbox};
use log::{error, info, SetLoggerError};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

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
    let app = App::new(HelperAppCallbacks::new());
    setup_logging()?;
    // Do not remove .clone() here, it is needed for correct reference counting
    let ret = match run(app.clone()) {
        Ok(code) => code,
        Err(e) => {
            error!("An error occurred: {}", e);
            1
        }
    };

    info!("The return code is: {}", ret);
    exit(ret);
}

fn run(app: App) -> Result<i32> {
    let _sandbox = ScopedSandbox::new()?;
    let main_args = MainArgs::new()?;
    Ok(execute_process(main_args, Some(app)))
}
struct HelperAppCallbacks {
    // render_process_handler: RenderProcessHandler,
}

impl HelperAppCallbacks {
    fn new() -> Self {
        Self {
            // render_process_handler: RenderProcessHandler::new(RenderProcessCallbacks),
        }
    }
}

impl AppCallbacks for HelperAppCallbacks {
    fn on_register_custom_schemes(&mut self, registrar: cef_ui_helper::SchemeRegistrar) {
        let _ = registrar.add_custom_scheme("huly", cef_ui_helper::SchemeOptions::Local.into());
    }

    // fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
    //     Some(self.render_process_handler.clone())
    // }
}
