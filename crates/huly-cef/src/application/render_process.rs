use anyhow::Result;
use serde_json::json;

use crate::{
    application::ipc, GET_CLICKABLE_ELEMENTS, GET_ELEMENT_CENTER, IS_ELEMENT_CLICKED,
    IS_ELEMENT_VISIBLE, IS_INTERACTIVE_ELEMENT, WALK_DOM,
};
use cef_ui::{
    register_extension, Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Value,
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

    fn on_process_message_received(
        &mut self,
        browser: Browser,
        frame: Frame,
        _: ProcessId,
        message: &mut ProcessMessage,
    ) -> bool {
        let request: ipc::Request = message.clone().into();

        let context = frame
            .get_v8context()
            .expect("failed to get V8 context from the frame");

        let result = match request.body {
            ipc::RequestBody::GetClickableElements => {
                let result = execute_javascript(&context, "getClickableElements();");
                json!({ "clickable_elements": result })
            }
            ipc::RequestBody::GetElementCenter { selector } => {
                let result =
                    execute_javascript(&context, &format!("getElementCenter('{}');", selector));
                json!({ "element_center": result })
            }
            ipc::RequestBody::CheckElementClicked { selector } => {
                let result =
                    execute_javascript(&context, &format!("isElementClicked('{}');", selector));
                json!({ "clicked": result })
            }
        };

        let response = json!({
            "id": request.id,
            "body": result,
        })
        .to_string();

        send_message(&browser, &response).expect("failed to send IPC response message");

        false
    }
}

fn execute_javascript(context: &V8Context, script: &str) -> String {
    context.enter().expect("failed to enter V8 context");

    let mut result = V8Value::create_string("");
    context
        .eval(script, "", 0, &mut result)
        .expect("failed to evaluate the script");
    let result = result
        .get_string_value()
        .expect("return value should be string");

    context.exit().expect("failed to exit V8 context");

    result
}

fn send_message(browser: &Browser, message: &str) -> Result<()> {
    let ipc_message = ProcessMessage::new("response");
    let argument_list = ipc_message
        .get_argument_list()?
        .expect("failed to get argument list");
    argument_list.set_string(0, &message)?;

    browser
        .get_main_frame()?
        .unwrap()
        .send_process_message(ProcessId::Browser, ipc_message)?;

    Ok(())
}
