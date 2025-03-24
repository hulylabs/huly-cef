use std::sync::Arc;

use anyhow::Result;
use cef_ui::{
    Browser, ProcessMessage, RenderProcessHandlerCallbacks, V8Handler, V8HandlerCallbacks, V8Value,
};

struct UrlHoveredV8HandlerCallbacks {
    browser: Arc<Browser>,
}

impl V8HandlerCallbacks for UrlHoveredV8HandlerCallbacks {
    fn execute(
        &mut self,
        name: String,
        _object: V8Value,
        arguments_count: usize,
        arguments: Vec<V8Value>,
    ) -> Result<i32> {
        if arguments_count < 2 {
            panic!("not enough arguments for urlHovered function");
        }

        let url = arguments[0]
            .get_string_value()
            .expect("failed to get url from urlHovered function");
        let hovered = arguments[1]
            .get_bool_value()
            .expect("failed to get hovered from urlHovered function");

        let msg = ProcessMessage::new("UrlHoveredMessage");

        let args = msg.get_argument_list().unwrap().unwrap();
        args.set_string(0, &url)
            .expect("failed to set url in UrlHoveredMessage");
        args.set_bool(1, hovered)
            .expect("failed to set hovered in UrlHoveredMessage");

        _ = self
            .browser
            .get_main_frame()
            .unwrap()
            .unwrap()
            .send_process_message(cef_ui::ProcessId::Browser, msg);
        Ok(1)
    }
}

pub struct HulyRenderProcessHandlerCallbacks;

impl RenderProcessHandlerCallbacks for HulyRenderProcessHandlerCallbacks {
    fn on_web_kit_initialized(&mut self) {}

    fn on_browser_created(
        &mut self,
        _browser: cef_ui::Browser,
        _extra_info: Option<cef_ui::DictionaryValue>,
    ) {
    }

    fn on_browser_destroyed(&mut self, _browser: cef_ui::Browser) {}

    fn on_context_created(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        context: cef_ui::V8Context,
    ) {
        let mouseover_listener = r#"document.addEventListener('mouseover', event => {
                let target = event.target.closest('a');
                if (target) {
                    console.log('Unhovered URL');
                    window.urlHovered(target.href, true);
                }
            });"#;

        let mouseout_listener = r#"document.addEventListener('mouseout', event => {
                let target = event.target.closest('a');
                if (target) {
                    console.log('Hovered URL:', target.href);
                    window.urlHovered(target.href, false);
                }
            });"#;

        frame
            .execute_java_script(mouseover_listener, "", 0)
            .expect("failed to execute script");

        frame
            .execute_java_script(mouseout_listener, "", 0)
            .expect("failed to execute script");

        let handler = V8Handler::new(UrlHoveredV8HandlerCallbacks {
            browser: Arc::new(browser),
        });
        let func = V8Value::create_function("urlHovered", handler)
            .expect("failed to create func urlHovered");

        context
            .get_global()
            .expect("failed to get global context object")
            .set_value_by_key("urlHovered", func)
            .expect("failed to set urlHovered function");
    }
}
