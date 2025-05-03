use cef_ui::{AppCallbacks, BrowserProcessHandler, CommandLine, RenderProcessHandler};

mod render_process_callbacks;

pub struct HulyAppCallbacks {
    render_process_handler: RenderProcessHandler,
}

impl HulyAppCallbacks {
    pub fn new() -> Self {
        let render_process_handler = RenderProcessHandler::new(
            render_process_callbacks::HulyRenderProcessHandlerCallbacks {},
        );
        Self {
            render_process_handler,
        }
    }
}

impl AppCallbacks for HulyAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        _process_type: Option<&str>,
        command_line: Option<CommandLine>,
    ) {
        if let Some(command_line) = command_line {
            _ = command_line.append_switch("enable-media-stream");
            _ = command_line
                .append_switch_with_value("load-extension", Some("/home/nikita/Downloads/unhook"));
        }
    }

    fn get_browser_process_handler(&mut self) -> Option<BrowserProcessHandler> {
        None
    }

    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        Some(self.render_process_handler.clone())
    }
}
