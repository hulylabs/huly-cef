use std::sync::Arc;

use anyhow::Result;

use cef_ui::{Browser, DictionaryValue, Frame, ProcessMessage, RenderProcessHandlerCallbacks, V8Context, V8Handler, V8HandlerCallbacks, V8Value};

pub struct MyV8Handler {
    browser: Arc<Browser>
}

impl V8HandlerCallbacks for MyV8Handler {
    fn execute(&mut self, name: String, _object: V8Value, arguments_count: usize, arguments: Vec<V8Value>) -> Result<i32> {
        println!("executed func {} with {} arguments", name, arguments_count);
        println!("std::process::id(): {}", std::process::id());

        if arguments_count < 3 {
            return Ok(0)
        }

        let url = arguments[0].get_string_value().unwrap();
        let pos_x = arguments[1].get_value_by_key("x").unwrap().get_int_value().unwrap();
        let pos_y = arguments[1].get_value_by_key("y").unwrap().get_int_value().unwrap();
        let w = arguments[2].get_value_by_key("w").unwrap().get_int_value().unwrap();
        let h = arguments[2].get_value_by_key("h").unwrap().get_int_value().unwrap();

        let msg = ProcessMessage::new("message from render process");

        let args = msg.get_argument_list().unwrap().unwrap();
        let _ = args.set_string(0, &url);
        let _ = args.set_int(1, pos_x);
        let _ = args.set_int(2, pos_y);
        let _ = args.set_int(3, w);
        let _ = args.set_int(4, h);

        _ = self.browser.get_main_frame().unwrap().unwrap().send_process_message(cef_ui::ProcessId::Browser, msg);
        Ok(1)
    }
}
/// Render process handler.
pub struct MyRenderProcessHandler;

impl RenderProcessHandlerCallbacks for MyRenderProcessHandler {
    fn on_browser_created(
        &mut self,
        _browser: Browser,
        _extra_info: Option<DictionaryValue>
    ) {
    }

    fn on_browser_destroyed(
        &mut self,
        _browser: Browser
    ) {
    }
    
    fn on_web_kit_initialized(
        &mut self
    ) {
    }
    
    fn on_context_created(
        &mut self,
        browser: Browser,
        _frame: Frame,
        context: V8Context
    ) {
        let handler = V8Handler::new(MyV8Handler { browser: Arc::new(browser) });
        let func = V8Value::create_function("create_new_window", handler).expect("failed to create func");

        let object = context.get_global().unwrap();
        _ = object.set_value_by_key("create_new_window", func);
    }
}