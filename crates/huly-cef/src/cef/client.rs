use std::sync::{Arc, Mutex};

use cef_ui::{
    Browser, Client, ClientCallbacks, ContextMenuHandler, DisplayHandler, Frame, KeyboardHandler,
    LifeSpanHandler, LoadHandler, ProcessId, ProcessMessage, RenderHandler,
};
use tokio::sync::mpsc::UnboundedSender;

use super::{browser::BrowserState, messages::CefMessage};

mod display_callbacks;
mod life_span_callbacks;
mod load_callbacks;
mod render_callbacks;

pub struct HulyClientCallbacks {
    render_handler: RenderHandler,
    load_handler: LoadHandler,
    display_handler: DisplayHandler,
    life_span_handler: LifeSpanHandler,
}

impl HulyClientCallbacks {
    pub fn new(
        render_handler: RenderHandler,
        load_handler: LoadHandler,
        display_handler: DisplayHandler,
        life_span_handler: LifeSpanHandler,
    ) -> Self {
        Self {
            render_handler,
            load_handler,
            display_handler,
            life_span_handler,
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
        Some(self.life_span_handler.clone())
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

    fn get_display_handler(&mut self) -> Option<cef_ui::DisplayHandler> {
        Some(self.display_handler.clone())
    }
}

pub fn new(state: Arc<Mutex<BrowserState>>, sender: UnboundedSender<CefMessage>) -> cef_ui::Client {
    let render_handler = RenderHandler::new(render_callbacks::HulyRenderHandlerCallbacks::new(
        state.clone(),
    ));
    let load_handler = LoadHandler::new(load_callbacks::HulyLoadHandlerCallbacks::new(
        sender.clone(),
    ));
    let display_handler = DisplayHandler::new(display_callbacks::HulyDisplayHandlerCallbacks::new(
        sender.clone(),
    ));
    let life_span_handler = LifeSpanHandler::new(
        life_span_callbacks::HulyLifeSpanHandlerCallbacks::new(sender.clone()),
    );

    Client::new(HulyClientCallbacks::new(
        render_handler,
        load_handler,
        display_handler,
        life_span_handler,
    ))
}
