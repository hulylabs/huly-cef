use cef_ui::{
    Browser, ContextMenuHandlerCallbacks, ContextMenuParams, EventFlags, Frame, MenuCommandId,
    MenuModel, Point, QuickMenuEditStateFlags, RunContextMenuCallback, RunQuickMenuCallback, Size,
};
use tracing::error;

pub struct MyContextMenuHandler;

#[allow(unused_variables)]
impl ContextMenuHandlerCallbacks for MyContextMenuHandler {
    fn on_before_context_menu(
        &mut self,
        _browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel,
    ) {
        // Prevent popups from spawning.
        if let Err(e) = model.clear() {
            error!("{}", e);
        }
    }

    fn run_context_menu(
        &mut self,
        _browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel,
        callback: RunContextMenuCallback,
    ) -> bool {
        false
    }

    fn on_context_menu_command(
        &mut self,
        _browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        command_id: MenuCommandId,
        event_flags: EventFlags,
    ) -> bool {
        false
    }

    fn on_context_menu_dismissed(&mut self, _browser: Browser, frame: Frame) {}

    fn run_quick_menu(
        &mut self,
        _browser: Browser,
        frame: Frame,
        location: &Point,
        size: &Size,
        edit_state_flags: QuickMenuEditStateFlags,
        callback: RunQuickMenuCallback,
    ) -> bool {
        false
    }

    fn on_quick_menu_command(
        &mut self,
        _browser: Browser,
        frame: Frame,
        command_id: MenuCommandId,
        event_flags: EventFlags,
    ) -> bool {
        false
    }

    fn on_quick_menu_dismissed(&mut self, _browser: Browser, frame: Frame) {}
}
