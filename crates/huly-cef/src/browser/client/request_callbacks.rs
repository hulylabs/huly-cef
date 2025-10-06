use cef_ui::{
    Browser, Frame, Request, RequestHandlerCallbacks, ResourceRequestHandler,
    ResourceRequestHandlerCallbacks, TerminationStatus,
};
use log::info;

use crate::browser::state::SharedBrowserState;

pub struct HulyRequestHandlerCallbacks {
    #[allow(unused)]
    state: SharedBrowserState,
    resource_request_handler: ResourceRequestHandler,
}

impl HulyRequestHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            resource_request_handler: ResourceRequestHandler::new(
                HulyResourceRequestHandlerCallbacks {},
            ),
        }
    }
}

impl RequestHandlerCallbacks for HulyRequestHandlerCallbacks {
    fn get_resource_request_handler(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _request: Request,
        _is_navigation: bool,
        _is_download: bool,
        _request_initiator: &str,
        _disable_default_handling: &mut bool,
    ) -> Option<ResourceRequestHandler> {
        // Some(self.resource_request_handler.clone())
        None
    }
    fn on_render_process_terminated(
        &mut self,
        _browser: Browser,
        status: TerminationStatus,
        error_code: i32,
        _error_string: Option<String>,
    ) {
        println!("Render process terminated: {:?} - {}", status, error_code);
    }
}

struct HulyResourceRequestHandlerCallbacks;

impl ResourceRequestHandlerCallbacks for HulyResourceRequestHandlerCallbacks {
    fn on_protocol_execution(
        &mut self,
        _browser: Option<Browser>,
        _frame: Option<Frame>,
        _request: Request,
    ) -> bool {
        info!("on_protocol_execution");
        true
    }
}
