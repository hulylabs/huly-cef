use cef_ui::{Context, ContextMenuHandlerCallbacks};

pub struct ContextMenuCallbacks;

impl ContextMenuHandlerCallbacks for ContextMenuCallbacks {
    fn on_before_context_menu(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        params: cef_ui::ContextMenuParams,
        model: cef_ui::MenuModel
    ) {
        model.clear();
    }

    fn run_context_menu(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        params: cef_ui::ContextMenuParams,
        model: cef_ui::MenuModel,
        callback: cef_ui::RunContextMenuCallback
    ) -> bool {
        false
    }

    fn on_context_menu_command(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        params: cef_ui::ContextMenuParams,
        command_id: cef_ui::MenuCommandId,
        event_flags: cef_ui::EventFlags
    ) -> bool {
        true
    }

    fn on_context_menu_dismissed(&mut self, browser: cef_ui::Browser, frame: cef_ui::Frame) {
        
    }

    fn run_quick_menu(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        location: &cef_ui::Point,
        size: &cef_ui::Size,
        edit_state_flags: cef_ui::QuickMenuEditStateFlags,
        callback: cef_ui::RunQuickMenuCallback
    ) -> bool {
        true
    }
    
    fn on_quick_menu_command(
        &mut self,
        browser: cef_ui::Browser,
        frame: cef_ui::Frame,
        command_id: cef_ui::MenuCommandId,
        event_flags: cef_ui::EventFlags
    ) -> bool {
        true
    }
    
    fn on_quick_menu_dismissed(&mut self, browser: cef_ui::Browser, frame: cef_ui::Frame) {
        
    }

}