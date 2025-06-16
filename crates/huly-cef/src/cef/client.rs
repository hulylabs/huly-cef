use std::sync::{Arc, Mutex};

use cef_ui::{
    Browser, Client, ClientCallbacks, ContextMenuHandler, DisplayHandler, Frame, KeyboardHandler,
    LifeSpanHandler, LoadHandler, ProcessId, ProcessMessage, RenderHandler, RequestHandler,
};
use log::error;
use tokio::sync::mpsc::UnboundedSender;

use super::{browser::BrowserState, messages::TabMessage};

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
        _browser: Browser,
        _frame: Frame,
        _source_process: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        let message_name = message.get_name().unwrap_or_default();
        if message_name == "getElementCenterResponse" {
            let args = message.get_argument_list().unwrap_or_default().unwrap();
            let (x, y) = (args.get_int(0).unwrap(), args.get_int(1).unwrap());

            let mut state = self.state.lock().unwrap();
            if let Some(tx) = state.get_center_oneshot_channel.take() {
                if tx.send(Ok((x, y))).is_err() {
                    error!("Failed to send getElementCenter response");
                }
            } else {
                error!("No channel to send getElementCenter response");
            }
        }

        true
    }

    fn get_request_handler(&mut self) -> Option<cef_ui::RequestHandler> {
        Some(self.request_handler.clone())
    }
}

pub fn new(state: Arc<Mutex<BrowserState>>, sender: UnboundedSender<TabMessage>) -> cef_ui::Client {
    Client::new(HulyClientCallbacks::new(sender, state))
}
