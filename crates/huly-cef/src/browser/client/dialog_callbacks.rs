use cef_ui::{Browser, DialogHandlerCallbacks, FileDialogCallback, FileDialogMode};

use crate::{state::SharedBrowserState, TabMessage};

pub struct HulyDialogHandlerCallbacks {
    state: SharedBrowserState,
}

impl HulyDialogHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self { state }
    }
}

impl DialogHandlerCallbacks for HulyDialogHandlerCallbacks {
    fn on_file_dialog(
        &mut self,
        _: Browser,
        mode: FileDialogMode,
        title: Option<String>,
        default_file_path: Option<String>,
        accept_filters: Vec<String>,
        accept_extensions: Vec<String>,
        accept_descriptions: Vec<String>,
        callback: FileDialogCallback,
    ) -> bool {
        let msg = TabMessage::FileDialog {
            mode: mode as i32,
            title: title.unwrap_or_default(),
            default_file_path: default_file_path.unwrap_or_default(),
            accept_types: accept_filters,
            accept_extensions,
            accept_descriptions,
        };
        self.state.notify(msg);
        self.state
            .update(|s| s.file_dialog_callback = Some(callback));
        true
    }
}
