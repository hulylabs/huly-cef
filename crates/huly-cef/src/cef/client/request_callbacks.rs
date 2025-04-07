use cef_ui::{Browser, Frame, RequestHandlerCallbacks, TerminationStatus};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

pub struct HulyRequestHandlerCallbacks {
    cef_message_channel: UnboundedSender<CefMessage>,
}

impl HulyRequestHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<CefMessage>) -> Self {
        Self {
            cef_message_channel,
        }
    }
}

impl RequestHandlerCallbacks for HulyRequestHandlerCallbacks {
    fn on_before_browse(
        &mut self,
        _browser: cef_ui::Browser,
        _frame: cef_ui::Frame,
        _request: cef_ui::Request,
        _user_gesture: bool,
        _is_redirect: bool,
    ) -> bool {
        false
    }

    fn on_open_urlfrom_tab(
        &mut self,
        _browser: cef_ui::Browser,
        _frame: cef_ui::Frame,
        _target_url: &str,
        _target_disposition: cef_ui::WindowOpenDisposition,
        _user_gesture: bool,
    ) -> bool {
        true
    }

    fn get_resource_request_handler(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _request: cef_ui::Request,
        _is_navigation: bool,
        _is_download: bool,
        _request_initiator: &str,
        _disable_default_handling: &mut bool,
    ) -> Option<cef_ui::ResourceRequestHandler> {
        None
    }

    fn get_auth_credentials(
        &mut self,
        _browser: cef_ui::Browser,
        _origin_url: &str,
        _is_proxy: bool,
        _host: &str,
        _port: u16,
        _realm: Option<&str>,
        _scheme: Option<&str>,
        _callback: cef_ui::AuthCallback,
    ) -> bool {
        true
    }

    fn on_certificate_error(
        &mut self,
        _browser: cef_ui::Browser,
        _cert_error: cef_ui::ErrorCode,
        _request_url: &str,
        _ssl_info: cef_ui::SslInfo,
        _callback: cef_ui::Callback,
    ) -> bool {
        true
    }

    fn on_select_client_certificate(
        &mut self,
        _browser: cef_ui::Browser,
        _is_proxy: bool,
        _host: &str,
        _port: u16,
        _certificates: &[cef_ui::X509Certificate],
        _callback: cef_ui::SelectClientCertificateCallback,
    ) -> bool {
        true
    }

    fn on_render_view_ready(&mut self, _browser: Browser) {}

    fn on_document_available_in_main_frame(&mut self, _browser: Browser) {}

    fn on_render_process_terminated(
        &mut self,
        _browser: Browser,
        _status: TerminationStatus,
        _error_code: i32,
        _error_string: Option<String>,
    ) {
    }
}
