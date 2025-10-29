use anyhow::Result;

use crate::js::{
    GET_CLICKABLE_ELEMENTS_SCRIPT, INTERACTIVE_ELEMENT_FUNCTION, IS_ELEMENT_VISIBLE_FUNCTION,
    WALK_DOM_FUNCTION,
};
use cef_ui_helper::{
    register_extension, Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Value,
};

pub struct RenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        _ = register_extension("is_interactive_element", INTERACTIVE_ELEMENT_FUNCTION, None);
        _ = register_extension("is_element_visible", IS_ELEMENT_VISIBLE_FUNCTION, None);
        _ = register_extension("walk_dom", WALK_DOM_FUNCTION, None);
        _ = register_extension(
            "get_clickable_elements",
            GET_CLICKABLE_ELEMENTS_SCRIPT,
            None,
        );
    }

    fn on_process_message_received(
        &mut self,
        browser: Browser,
        frame: Frame,
        _source_process: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        let name = message
            .get_name()
            .expect("failed to get process message name");

        let args = message
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get message arguments");

        let id = args.get_string(0).ok().flatten().expect("no id");

        let context = frame
            .get_v8context()
            .expect("failed to get V8 context from the frame");

        if name == "getClickableElements" {
            let result = process_get_clickable_elements_message(&context);
            send_js_message(&browser, &id, &result)
                .expect("failed to send clickable elements message");
            return true;
        }

        false
    }
}

fn process_get_clickable_elements_message(context: &V8Context) -> String {
    context.enter().expect("failed to enter V8 context");

    let mut result = V8Value::create_string("");

    context
        .eval("getClickableElements();", "", 0, &mut result)
        .expect("failed to evaluate getClickableElements() function");

    let clickable_elements_json = result.get_string_value().expect("result must be a string");
    println!("Clickable elements JSON: {}", clickable_elements_json);

    context.exit().expect("failed to exit V8 context");

    clickable_elements_json
}

fn send_js_message(browser: &Browser, id: &str, message: &str) -> Result<()> {
    let ipc_message = ProcessMessage::new("javascript_message");
    let argument_list = ipc_message
        .get_argument_list()
        .ok()
        .flatten()
        .expect("failed to get argument list");
    _ = argument_list.set_string(0, id);
    _ = argument_list.set_string(1, message);

    _ = browser
        .get_main_frame()
        .unwrap()
        .unwrap()
        .send_process_message(ProcessId::Browser, ipc_message);

    Ok(())
}
