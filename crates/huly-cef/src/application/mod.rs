use cef_ui::{
    AppCallbacks, BrowserProcessHandler, CommandLine, RenderProcessHandler, SchemeOptions,
    SchemeRegistrar,
};
use log::info;

mod browser_process;
mod render_process;

pub struct HulyAppCallbacks {
    browser_process_handler: BrowserProcessHandler,
    render_process_handler: RenderProcessHandler,
}

impl HulyAppCallbacks {
    pub fn new(port: u16, cache_path: String) -> Self {
        let browser_process_handler = BrowserProcessHandler::new(
            browser_process::BrowserProcessCallbacks::new(port, cache_path),
        );
        let render_process_handler =
            RenderProcessHandler::new(render_process::RenderProcessCallbacks {});
        Self {
            browser_process_handler,
            render_process_handler,
        }
    }
}

impl AppCallbacks for HulyAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        _: Option<&str>,
        command_line: Option<CommandLine>,
    ) {
        if let Some(command_line) = command_line {
            _ = command_line.append_switch("enable-media-stream");
        }
    }

    fn on_register_custom_schemes(&mut self, registrar: SchemeRegistrar) {
        info!("registering schemes");
        let options = SchemeOptions::Local;
        registrar
            .add_custom_scheme("huly", options.into())
            .expect("Failed to register custom scheme");
        info!("registering schemes");
    }

    fn get_browser_process_handler(&mut self) -> Option<BrowserProcessHandler> {
        Some(self.browser_process_handler.clone())
    }

    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        Some(self.render_process_handler.clone())
    }
}
