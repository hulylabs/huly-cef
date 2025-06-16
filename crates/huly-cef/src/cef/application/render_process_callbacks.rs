use cef_ui::{
    Browser, DictionaryValue, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Value,
};

pub struct HulyRenderProcessHandlerCallbacks;

impl HulyRenderProcessHandlerCallbacks {
    fn construct_error_message(error: &str) -> ProcessMessage {
        let message = ProcessMessage::new("error");
        _ = message
            .get_argument_list()
            .unwrap()
            .unwrap()
            .set_string(0, error);
        message
    }

    pub fn process_get_center_message(frame: &Frame, message: &ProcessMessage) -> ProcessMessage {
        let args = message.get_argument_list().unwrap_or_default();
        let Some(selector) = args
            .and_then(|args| args.get_string(0).ok().flatten())
            .map(|s| s.to_string())
        else {
            return Self::construct_error_message(
                "selector is missing in getElementCenter message",
            );
        };

        let context = frame.get_v8context().expect("failed to get V8 context");
        context.enter().expect("failed to enter V8 context");
        let mut retval = V8Value::create_object();
        _ = context.eval(&format!("getCenter('{selector}')"), "", 0, &mut retval);
        let (Ok(x), Ok(y)) = (
            retval
                .get_value_by_key("x")
                .and_then(|v| v.get_double_value()),
            retval
                .get_value_by_key("y")
                .and_then(|v| v.get_double_value()),
        ) else {
            return Self::construct_error_message("failed to get x or y from getCenter result");
        };
        context.exit().expect("failed to exit V8 context");

        let response = ProcessMessage::new("getElementCenterResponse");
        let args = response.get_argument_list().unwrap().unwrap();
        _ = args.set_int(0, x as i32);
        _ = args.set_int(1, y as i32);
        response
    }
}

impl RenderProcessHandlerCallbacks for HulyRenderProcessHandlerCallbacks {
    fn on_web_kit_initialized(&mut self) {
        _ = cef_ui::register_extension(
            "utility",
            "let getCenter = function(selector) {
                let element = document.querySelector(selector);
                if (!element) {
                    return null;
                }
                let rect = element.getBoundingClientRect();
                return {
                    x: rect.left + (rect.width / 2),
                    y: rect.top + (rect.height / 2)
                };
            };",
            None,
        );
    }

    fn on_browser_created(&mut self, _: Browser, _: Option<DictionaryValue>) {}

    fn on_browser_destroyed(&mut self, _: Browser) {}

    fn on_context_created(&mut self, _: Browser, _: Frame, _: V8Context) {}

    fn on_process_message_received(
        &mut self,
        _: Browser,
        frame: Frame,
        _: ProcessId,
        message: &mut ProcessMessage,
    ) -> bool {
        let message_name = message.get_name().unwrap_or_default();
        if message_name == "getElementCenter" {
            _ = frame.send_process_message(
                ProcessId::Browser,
                Self::process_get_center_message(&frame, &message),
            );
        }

        true
    }
}
