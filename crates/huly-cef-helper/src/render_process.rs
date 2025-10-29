use cef_ui_helper::{
    register_extension, Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Value,
};

use crate::{
    ipc,
    js::{
        GET_CLICKABLE_ELEMENTS, GET_ELEMENT_CENTER, IS_ELEMENT_CLICKED, IS_ELEMENT_VISIBLE,
        IS_INTERACTIVE_ELEMENT, WALK_DOM,
    },
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
        _: Browser,
        frame: Frame,
        _: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        let request: ipc::Request = message.clone().into();

        let context = frame
            .get_v8context()
            .expect("failed to get V8 context from the frame");

        let result = match request.body {
            ipc::RequestBody::GetClickableElements => {
                let result = execute_javascript(&context, "getClickableElements();");
                let elements: Vec<ipc::ClickableElement> = serde_json::from_str(&result)
                    .expect("failed to deserialize clickable elements");

                ipc::ResponseBody::ClickableElements(elements)
            }
            ipc::RequestBody::GetElementCenter { selector } => {
                let result =
                    execute_javascript(&context, &format!("getElementCenter('{}');", selector));
                let center: (i32, i32) =
                    serde_json::from_str(&result).expect("failed to deserialize center");
                ipc::ResponseBody::ElementCenter {
                    x: center.0,
                    y: center.1,
                }
            }
            ipc::RequestBody::CheckElementClicked { selector } => {
                let result =
                    execute_javascript(&context, &format!("isElementClicked('{}');", selector));
                let clicked: bool =
                    serde_json::from_str(&result).expect("failed to deserialize clicked");
                ipc::ResponseBody::Clicked(clicked)
            }
        };

        let response = ipc::Response {
            id: request.id,
            body: result,
        };

        frame
            .send_process_message(ProcessId::Browser, response.into())
            .expect("failed to send IPC response message");

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
