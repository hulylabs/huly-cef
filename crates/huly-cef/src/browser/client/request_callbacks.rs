use cef_ui::{
    AuthCallback, Browser, Callback, ErrorCode, Frame, Request, RequestHandlerCallbacks,
    ResourceRequestHandler, SelectClientCertificateCallback, SslInfo, TerminationStatus,
    WindowOpenDisposition, X509Certificate,
};

use crate::browser::state::SharedBrowserState;

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
    fn on_before_browse(&mut self, _: Browser, _: Frame, _: Request, _: bool, _: bool) -> bool {
        false
    }

    fn on_open_urlfrom_tab(
        &mut self,
        _: Browser,
        _: Frame,
        _: &str,
        _: WindowOpenDisposition,
        _: bool,
    ) -> bool {
        true
    }

    fn get_resource_request_handler(
        &mut self,
        _: Browser,
        _: Frame,
        _: Request,
        _: bool,
        _: bool,
        _: &str,
        _: &mut bool,
    ) -> Option<ResourceRequestHandler> {
        None
    }

    fn get_auth_credentials(
        &mut self,
        _: Browser,
        _: &str,
        _: bool,
        _: &str,
        _: u16,
        _: Option<&str>,
        _: Option<&str>,
        _: AuthCallback,
    ) -> bool {
        true
    }

    fn on_certificate_error(
        &mut self,
        _browser: Browser,
        _cert_error: ErrorCode,
        _request_url: &str,
        _ssl_info: SslInfo,
        _callback: Callback,
    ) -> bool {
        true
    }

    fn on_select_client_certificate(
        &mut self,
        _browser: Browser,
        _is_proxy: bool,
        _host: &str,
        _port: u16,
        _certificates: &[X509Certificate],
        _callback: SelectClientCertificateCallback,
    ) -> bool {
        true
    }

    fn on_render_view_ready(&mut self, _browser: Browser) {}

    fn on_document_available_in_main_frame(&mut self, _browser: Browser) {}

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
