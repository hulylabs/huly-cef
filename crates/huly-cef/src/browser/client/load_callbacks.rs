use std::sync::Mutex;

use crate::{browser::state::SharedBrowserState, LoadState, LoadStatus, TabMessage};
use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};

pub struct HulyLoadHandlerCallbacks {
    state: SharedBrowserState,
    load_state: Mutex<LoadState>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            load_state: Mutex::default(),
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
        let mut load_state = self.load_state.lock().unwrap();
        (*load_state).can_go_back = can_go_back;
        (*load_state).can_go_forward = can_go_forward;

        if self.state.read(|s| s.navigation_started) {
            if is_loading {
                (*load_state).status = LoadStatus::Loading;
            } else {
                (*load_state).status = LoadStatus::Loaded;
                self.state.update(|s| s.navigation_started = false);
            }
        }

        self.state.update(|s| s.load_state = load_state.clone());
        self.state.notify(TabMessage::LoadState(load_state.clone()));
    }

    fn on_load_start(&mut self, _: Browser, frame: Frame, _: TransitionType) {
        if frame.is_main().unwrap() {
            let mut load_state = self.load_state.lock().unwrap();
            (*load_state).status = LoadStatus::Loading;
            (*load_state).error_code = 0;
            (*load_state).error_message.clear();

            self.state.update(|s| s.load_state = load_state.clone());
            self.state.update(|s| s.navigation_started = false);
            self.state.notify(TabMessage::LoadState(load_state.clone()));
        }
    }

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code != 0 && (http_status_code < 200 || http_status_code > 299) {
                return;
            }

            let mut load_state = self.load_state.lock().unwrap();
            (*load_state).status = LoadStatus::Loaded;

            self.state.update(|s| s.load_state = load_state.clone());
            self.state.notify(TabMessage::LoadState(load_state.clone()));
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
            let mut load_state = self.load_state.lock().unwrap();
            (*load_state).status = LoadStatus::LoadError;
            (*load_state).error_code = error_code as i32;
            (*load_state).error_message = error_text.to_string();

            self.state.update(|s| s.load_state = load_state.clone());
            self.state.notify(TabMessage::LoadState(load_state.clone()));
        }
    }
}
