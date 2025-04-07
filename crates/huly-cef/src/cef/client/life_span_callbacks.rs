use cef_ui::{
    Browser, BrowserSettings, Client, DictionaryValue, Frame, LifeSpanHandlerCallbacks,
    PopupFeatures, WindowInfo, WindowOpenDisposition,
};
use log;
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

pub struct HulyLifeSpanHandlerCallbacks {
    cef_msg_channel: UnboundedSender<CefMessage>,
}

impl HulyLifeSpanHandlerCallbacks {
    pub fn new(cef_msg_channel: UnboundedSender<CefMessage>) -> Self {
        Self { cef_msg_channel }
    }
}

impl LifeSpanHandlerCallbacks for HulyLifeSpanHandlerCallbacks {
    unsafe fn on_before_popup(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _popup_id: i32,
        target_url: Option<String>,
        _target_frame_name: Option<String>,
        target_disposition: WindowOpenDisposition,
        _user_gesture: bool,
        _popup_features: PopupFeatures,
        _window_info: &mut WindowInfo,
        _client: &mut Option<Client>,
        _settings: &mut BrowserSettings,
        _extra_info: &mut Option<DictionaryValue>,
        _no_javascript_access: &mut bool,
    ) -> bool {
        match target_disposition {
            WindowOpenDisposition::NewForegroundTab
            | WindowOpenDisposition::NewBackgroundTab
            | WindowOpenDisposition::NewWindow => {
                if let Err(error) = self
                    .cef_msg_channel
                    .send(CefMessage::NewTabRequested(target_url.unwrap()))
                {
                    log::error!("Failed to send message: {:?}", error);
                }
            }
            _ => {}
        };
        true
    }

    fn on_before_dev_tools_popup(
        &mut self,
        _browser: Browser,
        _window_info: &mut WindowInfo,
        _client: &mut Option<Client>,
        _settings: &mut BrowserSettings,
        _extra_info: &mut Option<DictionaryValue>,
        _use_default_window: &mut bool,
    ) {
    }

    fn on_after_created(&mut self, _browser: Browser) {}

    fn do_close(&mut self, browser: Browser) -> bool {
        log::info!(
            "closing browser: {}",
            browser.get_identifier().expect("failed to get browser id")
        );

        if let Err(error) = self.cef_msg_channel.send(CefMessage::Closed) {
            log::error!("Failed to send message: {:?}", error);
        }
        true
    }

    fn on_before_close(&mut self, _browser: Browser) {}
}
