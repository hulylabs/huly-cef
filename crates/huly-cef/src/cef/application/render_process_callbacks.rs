use anyhow::Result;

use cef_ui::{
    Browser, DictionaryValue, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks,
    V8Context, V8Handler, V8HandlerCallbacks, V8Value,
};
use log::error;

pub struct HulyRenderProcessHandlerCallbacks;

impl RenderProcessHandlerCallbacks for HulyRenderProcessHandlerCallbacks {
    fn on_web_kit_initialized(&mut self) {}

    fn on_browser_created(&mut self, _: Browser, _: Option<DictionaryValue>) {}

    fn on_browser_destroyed(&mut self, _: Browser) {}

    fn on_context_created(&mut self, browser: Browser, _: Frame, context: V8Context) {
        let handler = V8Handler::new(GetClickableElementsCallback::new(browser));
        let func = V8Value::create_function("getClickableElements", handler)
            .expect("failed to create func getClickableElements");

        context
            .get_global()
            .expect("failed to get global context object")
            .set_value_by_key("getClickableElements", func)
            .expect("failed to set getClickableElements function");
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

macro_rules! try_get_string {
    ($element:expr, $key:expr) => {{
        let value = $element.get_value_by_key($key).unwrap();
        if !value.is_string().unwrap() {
            Err(anyhow::anyhow!("Invalid {} type: expected a string", $key))
        } else {
            value.get_string_value()
        }
    }};
}

macro_rules! try_get_double {
    ($element:expr, $key:expr) => {{
        let value = $element.get_value_by_key($key).unwrap();
        if !value.is_double().unwrap() {
            Err(anyhow::anyhow!("Invalid {} type: expected a double", $key))
        } else {
            value.get_double_value()
        }
    }};
}

struct GetClickableElementsCallback {
    browser: Browser,
    function_name: String,
}
impl GetClickableElementsCallback {
    pub fn new(browser: Browser) -> Self {
        Self {
            function_name: "getClickableElements".to_string(),
            browser,
        }
    }
}
impl V8HandlerCallbacks for GetClickableElementsCallback {
    fn execute(
        &mut self,
        name: String,
        _: V8Value,
        arguments_count: usize,
        arguments: Vec<V8Value>,
    ) -> Result<i32> {
        if name != self.function_name {
            error!("Invalid function name: {}. Expected: function_name", name);
            return Ok(1);
        }
        if arguments_count != 1 {
            error!(
                "Invalid number of arguments: expected 1, got {}",
                arguments_count
            );
            return Ok(1);
        }

        let first = arguments.get(0).unwrap();
        if !first.is_array().unwrap() {
            error!("Invalid argument type: expected an array");
            return Ok(1);
        }

        let length = first.get_array_length().unwrap();
        let message = ProcessMessage::new("clickable_elements");
        let argument_list = message.get_argument_list().unwrap().unwrap();

        for i in 0..length {
            let element = first.get_value_by_index(i).unwrap();
            if !element.is_object().unwrap() {
                error!("Invalid element type at index {}: expected an object", i);
                return Err(anyhow::anyhow!(
                    "Invalid element type at index {}: expected an object",
                    i
                ));
            }

            let tag = try_get_string!(element, "tag").unwrap_or_default();
            let text = try_get_string!(element, "text").unwrap_or_default();
            let x = try_get_double!(element, "x").unwrap_or_default();
            let y = try_get_double!(element, "y").unwrap_or_default();
            let offset = (i * 4) as usize;

            _ = argument_list.set_string(offset + 0, &tag);
            _ = argument_list.set_string(offset + 1, &text);
            _ = argument_list.set_int(offset + 2, x as i32);
            _ = argument_list.set_int(offset + 3, y as i32);
        }

        _ = self
            .browser
            .get_main_frame()
            .unwrap()
            .unwrap()
            .send_process_message(ProcessId::Browser, message);

        Ok(1)
    }
}
