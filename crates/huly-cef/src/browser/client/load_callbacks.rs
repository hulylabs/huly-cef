use std::sync::Mutex;

use crate::{browser::state::SharedBrowserState, LoadState, LoadStatus, TabMessage};
use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};
use log::info;

struct Flags {
    is_loading: bool,
    can_go_back: bool,
    can_go_forward: bool,
}
pub struct HulyLoadHandlerCallbacks {
    state: SharedBrowserState,
    flags: Mutex<Flags>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            flags: Mutex::new(Flags {
                is_loading: false,
                can_go_back: false,
                can_go_forward: false,
            }),
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
        let mut flags = self.flags.lock().unwrap();
        flags.is_loading = is_loading;
        flags.can_go_back = can_go_back;
        flags.can_go_forward = can_go_forward;
    }

    fn on_load_start(&mut self, _: Browser, frame: Frame, _: TransitionType) {
        if frame.is_main().unwrap() {
            let flags = self.flags.lock().unwrap();
            let load_state = LoadState {
                status: LoadStatus::Loading,
                can_go_back: flags.can_go_back,
                can_go_forward: flags.can_go_forward,
                ..Default::default()
            };

            self.state.update(|s| s.load_state = load_state.clone());
            self.state.notify(TabMessage::LoadState(load_state.clone()));
        }
    }

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code == 200 {
                let flags = self.flags.lock().unwrap();
                let load_state = LoadState {
                    status: LoadStatus::Loaded,
                    can_go_back: flags.can_go_back,
                    can_go_forward: flags.can_go_forward,
                    ..Default::default()
                };
                self.state.update(|s| s.load_state = load_state.clone());
                self.state.notify(TabMessage::LoadState(load_state.clone()));
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
            let flags = self.flags.lock().unwrap();
            let load_state = LoadState {
                status: LoadStatus::LoadError,
                can_go_back: flags.can_go_back,
                can_go_forward: flags.can_go_forward,
                error_code: error_code as i32,
                error_message: error_text.to_string(),
            };
            self.state.update(|s| s.load_state = load_state.clone());
            self.state.notify(TabMessage::LoadState(load_state.clone()));
        }
    }
}
