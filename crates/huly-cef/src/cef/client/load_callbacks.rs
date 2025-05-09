use cef_ui::{Browser, Frame, LoadHandlerCallbacks};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

pub struct HulyLoadHandlerCallbacks {
    cef_message_channel: UnboundedSender<CefMessage>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<CefMessage>) -> Self {
        Self {
            cef_message_channel,
        }
    }
}

impl LoadHandlerCallbacks for HulyLoadHandlerCallbacks {
    fn on_loading_state_change(
        &mut self,
        _browser: Browser,
        _is_loading: bool,
        _can_go_back: bool,
        _can_go_forward: bool,
    ) {
    }

    fn on_load_start(
        &mut self,
        _browser: Browser,
        frame: Frame,
        _transition_type: cef_ui::TransitionType,
    ) {
        if frame.is_main().unwrap() {
            _ = self.cef_message_channel.send(CefMessage::Loading);
        }
    }

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code == 200 {
                _ = self.cef_message_channel.send(CefMessage::Loaded);
            }
        }
    }

    fn on_load_error(
        &mut self,
        _browser: Browser,
        frame: Frame,
        _error_code: cef_ui::ErrorCode,
        error_text: &str,
        failed_url: &str,
    ) {
        if frame.is_main().unwrap() {
            _ = self.cef_message_channel.send(CefMessage::LoadError {
                error_code: 0,
                error_text: error_text.to_string(),
                failed_url: failed_url.to_string(),
            });
        }
    }
}
