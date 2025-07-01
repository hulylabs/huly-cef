use anyhow::Result;
use cef_ui::{Browser, ProcessId, ProcessMessage, V8HandlerCallbacks, V8Value};
use log::error;

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

pub struct GetClickableElementsHandler {
    browser: Browser,
    function_name: String,
}
impl GetClickableElementsHandler {
    pub fn new(browser: Browser) -> Self {
        Self {
            function_name: "getClickableElements".to_string(),
            browser,
        }
    }
}
impl V8HandlerCallbacks for GetClickableElementsHandler {
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

            let id = try_get_double!(element, "id").unwrap_or_default();
            let tag = try_get_string!(element, "tag").unwrap_or_default();
            let text = try_get_string!(element, "text").unwrap_or_default();

            let stride = 3;
            let offset = (i * stride) as usize;

            _ = argument_list.set_int(offset + 0, id as i32);
            _ = argument_list.set_string(offset + 1, &tag);
            _ = argument_list.set_string(offset + 2, &text);
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
