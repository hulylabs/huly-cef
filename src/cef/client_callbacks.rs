use cef_ui::{
    Browser, ClientCallbacks, ContextMenuHandler, Frame, KeyboardHandler, LifeSpanHandler,
    LoadHandler, ProcessId, ProcessMessage, RenderHandler,
};

pub struct HulyClientCallbacks {
    render_handler: RenderHandler,
    load_handler: LoadHandler,
}

impl HulyClientCallbacks {
    pub fn new(render_handler: RenderHandler, load_handler: LoadHandler) -> Self {
        Self {
            render_handler,
            load_handler,
        }
    }
}

impl ClientCallbacks for HulyClientCallbacks {
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

    fn get_load_handler(&mut self) -> Option<cef_ui::LoadHandler> {
        Some(self.load_handler.clone())
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
