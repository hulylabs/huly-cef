use cef_ui::{AppCallbacks, BrowserProcessHandler, CommandLine, RenderProcessHandler};
use tracing::info;

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
        process_type: Option<&str>,
        command_line: Option<CommandLine>,
    ) {
        info!("Setting CEF command line switches.");

        // This is to disable scary warnings on macOS.
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        if let Some(command_line) = command_line {
            if process_type.is_none() {
                if let Err(e) = command_line.append_switch("--use-mock-keychain") {
                    println!("{}", e);
                }
            }
        }
    }

    fn get_browser_process_handler(&mut self) -> Option<BrowserProcessHandler> {
        None
    }

    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        Some(self.render_process_handler.clone())
    }
}
