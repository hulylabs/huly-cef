use std::sync::{Arc, Mutex};

use cef_ui::{
    Browser, Client, ClientCallbacks, ContextMenuHandler, DisplayHandler, Frame, KeyboardHandler,
    LifeSpanHandler, LoadHandler, ProcessId, ProcessMessage, RenderHandler, RequestHandler,
};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    browser::{BrowserState, ClickableElement},
    messages::TabMessage,
};

mod display_callbacks;
mod life_span_callbacks;
mod load_callbacks;
mod render_callbacks;
mod request_callbacks;

pub struct HulyClientCallbacks {
    state: Arc<Mutex<BrowserState>>,
    render_handler: RenderHandler,
    load_handler: LoadHandler,
    display_handler: DisplayHandler,
    life_span_handler: LifeSpanHandler,
    request_handler: RequestHandler,
}

impl HulyClientCallbacks {
    pub fn new(sender: UnboundedSender<TabMessage>, state: Arc<Mutex<BrowserState>>) -> Self {
        let render_handler = RenderHandler::new(render_callbacks::HulyRenderHandlerCallbacks::new(
            sender.clone(),
            state.clone(),
        ));
        let load_handler = LoadHandler::new(load_callbacks::HulyLoadHandlerCallbacks::new(
            sender.clone(),
        ));
        let display_handler = DisplayHandler::new(
            display_callbacks::HulyDisplayHandlerCallbacks::new(sender.clone()),
        );
        let life_span_handler = LifeSpanHandler::new(
            life_span_callbacks::HulyLifeSpanHandlerCallbacks::new(sender.clone()),
        );

        let request_handler = RequestHandler::new(
            request_callbacks::HulyRequestHandlerCallbacks::new(sender.clone()),
        );

        Self {
            state: state,
            render_handler,
            load_handler,
            display_handler,
            life_span_handler,
            request_handler,
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

    fn get_display_handler(&mut self) -> Option<cef_ui::DisplayHandler> {
        Some(self.display_handler.clone())
    }

    fn on_process_message_received(
        &mut self,
        _: Browser,
        _: Frame,
        _: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        let message_name = message.get_name().unwrap_or_default();
        if message_name == "clickable_elements" {
            let mut elements = Vec::new();
            let args = message.get_argument_list().unwrap_or_default().unwrap();
            let len = args.len().unwrap();
            for i in 0..len / 4 {
                let tag = args.get_string(i * 4 + 0).unwrap().unwrap();
                let text = args
                    .get_string(i * 4 + 1)
                    .unwrap_or_default()
                    .unwrap_or_default();
                let x = args.get_int(i * 4 + 2).unwrap();
                let y = args.get_int(i * 4 + 3).unwrap();

                elements.push(ClickableElement { tag, text, x, y });
            }

            _ = self
                .state
                .lock()
                .unwrap()
                .clickable_elements_channel
                .take()
                .unwrap()
                .send(elements);
        }

        true
    }

    fn get_request_handler(&mut self) -> Option<RequestHandler> {
        Some(self.request_handler.clone())
    }
}

pub fn new(state: Arc<Mutex<BrowserState>>, sender: UnboundedSender<TabMessage>) -> Client {
    Client::new(HulyClientCallbacks::new(sender, state))
}
