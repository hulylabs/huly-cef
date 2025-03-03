use cef_ui::{Browser, DictionaryValue, Frame, RenderProcessHandlerCallbacks, V8Context};

/// Render process handler.
pub struct MyRenderProcessHandler;

impl RenderProcessHandlerCallbacks for MyRenderProcessHandler {
    fn on_browser_created(&mut self, _browser: Browser, _extra_info: Option<DictionaryValue>) {}

    fn on_browser_destroyed(&mut self, _browser: Browser) {}

    fn on_web_kit_initialized(&mut self) {}

    fn on_context_created(&mut self, _browser: Browser, _frame: Frame, _context: V8Context) {}
}
