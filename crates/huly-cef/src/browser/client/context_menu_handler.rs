use cef_ui::{Browser, ContextMenuHandlerCallbacks, ContextMenuParams, Frame, MenuModel};

pub struct ContextMenuCallbacks;

impl ContextMenuHandlerCallbacks for ContextMenuCallbacks {
    fn on_before_context_menu(
        &mut self,
        _: Browser,
        _: Frame,
        _: ContextMenuParams,
        model: MenuModel,
    ) {
        _ = model.clear();
    }
}
