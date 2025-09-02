use anyhow::Result;

use crate::javascript::{
    INTERACTIVE_ELEMENT_FUNCTION, IS_ELEMENT_VISIBLE_FUNCTION, WALK_DOM_FUNCTION,
};
use cef_ui::{
    register_extension, Browser, DictionaryValue, Frame, ProcessId, ProcessMessage,
    RenderProcessHandlerCallbacks, V8Context, V8Handler, V8HandlerCallbacks, V8Value,
};

pub struct RenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        _ = register_extension("is_interactive_element", INTERACTIVE_ELEMENT_FUNCTION, None);
        _ = register_extension("is_element_visible", IS_ELEMENT_VISIBLE_FUNCTION, None);
        _ = register_extension("walk_dom", WALK_DOM_FUNCTION, None);
    }

    fn on_browser_created(&mut self, _: Browser, _: Option<DictionaryValue>) {}

    fn on_browser_destroyed(&mut self, _: Browser) {}

    fn on_context_created(&mut self, _browser: Browser, frame: Frame, _context: V8Context) {
        if !frame.is_main().unwrap() {
            return;
        }

        // let func = V8Value::create_function(
        //     "sendMessage",
        //     V8Handler::new(SendMessageHandler::new(browser)),
        // )
        // .expect("failed to create func sendMessage");

        // context
        //     .get_global()
        //     .expect("failed to get global context object")
        //     .set_value_by_key("sendMessage", func)
        //     .expect("failed to set sendMessage function");
    }

    fn on_process_message_received(
        &mut self,
        _: Browser,
        _: Frame,
        _: ProcessId,
        _: &mut ProcessMessage,
    ) -> bool {
        true
    }
}

pub struct SendMessageHandler {
    browser: Browser,
}

impl SendMessageHandler {
    pub fn new(browser: Browser) -> Self {
        Self { browser }
    }
}

impl V8HandlerCallbacks for SendMessageHandler {
    fn execute(&mut self, _: String, _: V8Value, _: usize, arguments: Vec<V8Value>) -> Result<i32> {
        let first_arg = arguments.get(0).expect("first argument is required");

        let id = first_arg
            .get_value_by_key("id")
            .expect("failed to get id")
            .get_string_value()
            .expect("id must be a string");

        let message = first_arg
            .get_value_by_key("message")
            .expect("failed to get message")
            .get_string_value()
            .expect("message must be a string");

        let ipc_message = ProcessMessage::new("javascript_message");
        let argument_list = ipc_message
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get argument list");
        _ = argument_list.set_string(0, &id);
        _ = argument_list.set_string(1, &message);

        _ = self
            .browser
            .get_main_frame()
            .unwrap()
            .unwrap()
            .send_process_message(ProcessId::Browser, ipc_message);

        Ok(1)
    }
}
