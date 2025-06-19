use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};
use log::error;
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::TabMessage;

#[derive(Debug)]
struct LoadState {
    status: crate::messages::LoadStatus,
    error_code: i32,
    error_text: String,
}

pub struct HulyLoadHandlerCallbacks {
    event_channel: UnboundedSender<TabMessage>,
    load_state: Option<LoadState>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(event_channel: UnboundedSender<TabMessage>) -> Self {
        Self {
            event_channel,
            load_state: None,
        }
    }

    fn send_message(&self, message: TabMessage) {
        if let Err(e) = self.event_channel.send(message) {
            error!("Failed to send message: {:?}", e);
        }
    }
}

impl LoadHandlerCallbacks for HulyLoadHandlerCallbacks {
    fn on_loading_state_change(
        &mut self,
        _: Browser,
        is_loading: bool,
        can_go_back: bool,
        can_go_forward: bool,
    ) {
        // TODO: Why do we need it here?
        if !is_loading && self.load_state.is_none() {
            return;
        }

        let message = if !is_loading {
            let load_state = self.load_state.take().expect("load state can't be None");
            TabMessage::LoadStateChanged {
                status: load_state.status,
                can_go_back,
                can_go_forward,
                error_code: load_state.error_code,
                error_text: load_state.error_text,
            }
        } else {
            TabMessage::LoadStateChanged {
                status: crate::cef::messages::LoadStatus::Loading,
                can_go_back,
                can_go_forward,
                error_code: 0,
                error_text: String::new(),
            }
        };

        self.send_message(message);
    }

    fn on_load_start(&mut self, _: Browser, frame: Frame, _: TransitionType) {
        if frame.is_main().unwrap() {
            self.load_state = Some(LoadState {
                status: crate::messages::LoadStatus::Loading,
                error_code: 0,
                error_text: String::new(),
            });
        }
    }

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code == 200 || http_status_code == 0 {
                self.load_state = Some(LoadState {
                    status: crate::messages::LoadStatus::Loaded,
                    error_code: 0,
                    error_text: String::new(),
                });
            }
        }
    }

    fn on_load_error(
        &mut self,
        _: Browser,
        frame: Frame,
        error_code: ErrorCode,
        error_text: &str,
        _: &str,
    ) {
        if frame.is_main().unwrap() {
            self.load_state = Some(LoadState {
                status: crate::messages::LoadStatus::LoadError,
                error_code: error_code as i32,
                error_text: error_text.to_string(),
            });
        }
    }
}
