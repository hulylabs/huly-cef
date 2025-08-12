use cef_ui::{
    Browser, Client, ClientCallbacks, ContextMenuHandler, DisplayHandler, Frame, KeyboardHandler,
    LifeSpanHandler, LoadHandler, ProcessId, ProcessMessage, RenderHandler, RequestHandler,
};

use crate::browser::{automation::JSMessage, state::SharedBrowserState};

mod display_callbacks;
mod life_span_callbacks;
mod load_callbacks;
mod render_callbacks;
mod request_callbacks;

pub struct HulyClientCallbacks {
    state: SharedBrowserState,
    render_handler: RenderHandler,
    load_handler: LoadHandler,
    display_handler: DisplayHandler,
    life_span_handler: LifeSpanHandler,
    request_handler: RequestHandler,
}

impl HulyClientCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        let render_handler = RenderHandler::new(render_callbacks::HulyRenderHandlerCallbacks::new(
            state.clone(),
        ));
        let load_handler =
            LoadHandler::new(load_callbacks::HulyLoadHandlerCallbacks::new(state.clone()));
        let display_handler = DisplayHandler::new(
            display_callbacks::HulyDisplayHandlerCallbacks::new(state.clone()),
        );
        let life_span_handler = LifeSpanHandler::new(
            life_span_callbacks::HulyLifeSpanHandlerCallbacks::new(state.clone()),
        );
        let request_handler = RequestHandler::new(
            request_callbacks::HulyRequestHandlerCallbacks::new(state.clone()),
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
        ipc_msg: ProcessMessage,
    ) -> bool {
        let name = ipc_msg.get_name().unwrap_or_default();
        if name == "javascript_message" {
            let args = ipc_msg
                .get_argument_list()
                .unwrap()
                .expect("failed to get argument list");

            let id = args.get_string(0).ok().flatten().expect("no id");
            let msg = args.get_string(1).ok().flatten().expect("no message");

            let result = match serde_json::from_str::<JSMessage>(&msg) {
                Ok(value) => Ok(value),
                Err(e) => Err(anyhow::anyhow!("Failed to parse JSON message: {}", e)),
            };

            self.state.update(|state| {
                state
                    .javascript_messages
                    .remove(&id)
                    .and_then(|tx| Some(tx.send(result)));
            });
        }

        true
    }

    fn get_request_handler(&mut self) -> Option<RequestHandler> {
        Some(self.request_handler.clone())
    }
}

pub fn new(state: SharedBrowserState) -> Client {
    Client::new(HulyClientCallbacks::new(state))
}
