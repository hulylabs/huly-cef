use anyhow::Result;

use crate::js::{
    GET_CLICKABLE_ELEMENTS, GET_ELEMENT_CENTER, IS_ELEMENT_CLICKED, IS_ELEMENT_VISIBLE,
    IS_INTERACTIVE_ELEMENT, WALK_DOM,
};
use cef_ui_helper::{
    register_extension, Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Handler, V8HandlerCallbacks, V8Value,
};

pub struct RenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        _ = register_extension("is_interactive_element", IS_INTERACTIVE_ELEMENT, None);
        _ = register_extension("get_clickable_elements", GET_CLICKABLE_ELEMENTS, None);
        _ = register_extension("is_element_visible", IS_ELEMENT_VISIBLE, None);
        _ = register_extension("get_element_center", GET_ELEMENT_CENTER, None);
        _ = register_extension("is_element_clicked", IS_ELEMENT_CLICKED, None);
        _ = register_extension("walk_dom", WALK_DOM, None);
    }

    fn on_context_created(&mut self, browser: Browser, frame: Frame, context: V8Context) {
        if !frame.is_main().unwrap() {
            return;
        }

        let func = V8Value::create_function(
            "sendMessage",
            V8Handler::new(SendMessageHandler::new(browser)),
        )
        .expect("failed to create func sendMessage");

        context
            .get_global()
            .expect("failed to get global context object")
            .set_value_by_key("sendMessage", func)
            .expect("failed to set sendMessage function");
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
