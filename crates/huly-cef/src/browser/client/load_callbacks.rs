use crate::{browser::state::SharedBrowserState, LoadState, LoadStatus, TabMessage};
use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};

pub struct HulyLoadHandlerCallbacks {
    state: SharedBrowserState,
    status: LoadStatus,
    error: Option<(i32, String)>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            status: LoadStatus::Loading,
            error: None,
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
        if is_loading {
            self.status = LoadStatus::Loading;
        }

        let mut load_state = LoadState::default();
        load_state.status = self.status.clone();
        load_state.can_go_back = can_go_back;
        load_state.can_go_forward = can_go_forward;
        if let Some((error_code, error_text)) = self.error.take() {
            load_state.error_code = error_code;
            load_state.error_text = error_text.clone();
        }

        self.state
            .update(|state| state.load_state = load_state.clone());
        self.state.notify(TabMessage::LoadState(load_state));
    }

    fn on_load_start(&mut self, _: Browser, _: Frame, _: TransitionType) {}

    fn on_load_end(&mut self, _browser: Browser, frame: Frame, http_status_code: i32) {
        if frame.is_main().unwrap() {
            if http_status_code == 200 || http_status_code == 0 {
                self.status = LoadStatus::Loaded;
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
            self.status = LoadStatus::LoadError;
            self.error = Some((error_code as i32, error_text.to_string()));
        }
    }
}
