mod handlers;

use cef_ui::{
    register_extension, Browser, DictionaryValue, Frame, ProcessId, ProcessMessage,
    RenderProcessHandlerCallbacks, V8Context, V8Handler, V8Value,
};

use crate::javascript::{
    INTERACTIVE_ELEMENT_FUNCTION, IS_ELEMENT_VISIBLE_FUNCTION, WALK_DOM_FUNCTION,
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

    fn on_context_created(&mut self, browser: Browser, _: Frame, context: V8Context) {
        let handler = V8Handler::new(handlers::GetClickableElementsHandler::new(browser));
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
