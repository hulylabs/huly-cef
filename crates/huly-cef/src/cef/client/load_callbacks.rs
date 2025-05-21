use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

#[derive(Debug)]
struct LoadState {
    state: crate::messages::LoadState,
    error_code: i32,
    error_text: String,
}

pub struct HulyLoadHandlerCallbacks {
    cef_message_channel: UnboundedSender<CefMessage>,
    load_state: Option<LoadState>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<CefMessage>) -> Self {
        Self {
            cef_message_channel,
            load_state: None,
        }
    }
}

impl LoadHandlerCallbacks for HulyLoadHandlerCallbacks {
    fn on_loading_state_change(
        &mut self,
        _browser: Browser,
        is_loading: bool,
        can_go_back: bool,
        can_go_forward: bool,
    ) {
        if !is_loading && self.load_state.is_none() {
            return;
        }

        let mut message = CefMessage::LoadStateChanged {
            state: crate::messages::LoadState::Loading,
            can_go_back,
            can_go_forward,
            error_code: 0,
            error_text: String::new(),
        };

        if !is_loading {
            let load_state = self.load_state.take().expect("load state can't be None");
            message = CefMessage::LoadStateChanged {
                state: load_state.state,
                can_go_back,
                can_go_forward,
                error_code: load_state.error_code,
                error_text: load_state.error_text,
            };
        }

        _ = self.cef_message_channel.send(message);
    }

    fn on_load_start(&mut self, _browser: Browser, frame: Frame, _transition_type: TransitionType) {
        if frame.is_main().unwrap() {
            self.load_state = Some(LoadState {
                state: crate::messages::LoadState::Loading,
                error_code: 0,
                error_text: String::new(),
            });
        }
    }

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code == 200 || http_status_code == 0 {
                self.load_state = Some(LoadState {
                    state: crate::messages::LoadState::Loaded,
                    error_code: 0,
                    error_text: String::new(),
                });
            }
        }
    }

    fn on_load_error(
        &mut self,
        _browser: Browser,
        frame: Frame,
        error_code: ErrorCode,
        error_text: &str,
        _failed_url: &str,
    ) {
        if frame.is_main().unwrap() {
            self.load_state = Some(LoadState {
                state: crate::messages::LoadState::LoadError,
                error_code: error_code as i32,
                error_text: error_text.to_string(),
            });
        }
    }
}
