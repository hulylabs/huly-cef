use cef_ui::RenderProcessHandlerCallbacks;

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
        _browser: cef_ui::Browser,
        _frame: cef_ui::Frame,
        _context: cef_ui::V8Context,
    ) {
    }
}
