use crate::{browser::state::SharedBrowserState, TabMessage};
use cef_ui::{Browser, ErrorCode, Frame, LoadHandlerCallbacks, TransitionType};

#[derive(Debug)]
struct LoadState {
    status: crate::messages::LoadStatus,
    error_code: i32,
    error_text: String,
}

pub struct HulyLoadHandlerCallbacks {
    state: SharedBrowserState,
    load_state: Option<LoadState>,
}

impl HulyLoadHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            load_state: None,
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
            TabMessage::LoadState {
                status: load_state.status,
                can_go_back,
                can_go_forward,
                error_code: load_state.error_code,
                error_text: load_state.error_text,
            }
        } else {
            TabMessage::LoadState {
                status: crate::LoadStatus::Loading,
                can_go_back,
                can_go_forward,
                error_code: 0,
                error_text: String::new(),
            }
        };

        // TODO: Use a LoadStateStructure insterad of a message
        self.state.update(|state| {
            // state.load_status = message.status.clone();
            state.can_go_back = can_go_back;
            state.can_go_forward = can_go_forward;
        });

        self.state.notify(message);
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
