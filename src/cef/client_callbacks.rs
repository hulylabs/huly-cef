use cef_ui::Browser;
use cef_ui::ClientCallbacks;
use cef_ui::ContextMenuHandler;
use cef_ui::Frame;
use cef_ui::KeyboardHandler;
use cef_ui::LifeSpanHandler;
use cef_ui::ProcessId;
use cef_ui::ProcessMessage;
use cef_ui::RenderHandler;

pub struct MyClientCallbacks {
    render_handler: RenderHandler,
}

impl MyClientCallbacks {
    pub fn new(render_handler: RenderHandler) -> Self {
        Self {
            render_handler: render_handler,
        }
    }
}

impl ClientCallbacks for MyClientCallbacks {
    fn get_context_menu_handler(&mut self) -> Option<ContextMenuHandler> {
        None
    }

    fn get_keyboard_handler(&mut self) -> Option<KeyboardHandler> {
        None
    }

    fn get_life_span_handler(&mut self) -> Option<LifeSpanHandler> {
        None
    }

    fn get_render_handler(&mut self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }

    fn on_process_message_received(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _source_process: ProcessId,
        _message: ProcessMessage,
    ) -> bool {
        true
    }
}
