use cef_ui::{
    Browser, Frame, Request, RequestHandlerCallbacks, TerminationStatus, WindowOpenDisposition,
};
use log::warn;

use crate::{browser::state::SharedBrowserState, TabMessage};

static PROTOCOLS: &[&str] = &["http", "https", "file"];
pub struct HulyRequestHandlerCallbacks {
    #[allow(unused)]
    state: SharedBrowserState,
}

impl HulyRequestHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self { state }
    }
}

impl RequestHandlerCallbacks for HulyRequestHandlerCallbacks {
    fn on_before_browse(
        &mut self,
        _: Browser,
        _: Frame,
        request: Request,
        _: bool,
        _: bool,
    ) -> bool {
        let url = request.get_url().unwrap_or_default();
        let custom = PROTOCOLS.iter().any(|proto| url.starts_with(proto));
        let external = if custom { url } else { "".into() };
        self.state
            .notify(TabMessage::ExternalLink(external.clone()));
        self.state.update(|s| s.external_link = external);
        false
    }

    fn on_open_urlfrom_tab(
        &mut self,
        _: Browser,
        _: Frame,
        target_url: &str,
        target_disposition: WindowOpenDisposition,
        _: bool,
    ) -> bool {
        match target_disposition {
            WindowOpenDisposition::NewForegroundTab
            | WindowOpenDisposition::NewBackgroundTab
            | WindowOpenDisposition::NewWindow => {
                self.state
                    .notify(TabMessage::NewTab(target_url.to_string()));
            }
            _ => {}
        };
        true
    }
    fn on_render_process_terminated(
        &mut self,
        _browser: Browser,
        status: TerminationStatus,
        error_code: i32,
        _error_string: Option<String>,
    ) {
        warn!("Render process terminated: {:?} - {}", status, error_code);
    }
}
