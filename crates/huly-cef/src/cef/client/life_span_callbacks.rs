use cef_ui::{
    Browser, BrowserSettings, Client, DictionaryValue, Frame, LifeSpanHandlerCallbacks,
    PopupFeatures, WindowInfo, WindowOpenDisposition,
};
use log::{self, error};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::TabMessage;

pub struct HulyLifeSpanHandlerCallbacks {
    cef_msg_channel: UnboundedSender<TabMessage>,
}

impl HulyLifeSpanHandlerCallbacks {
    pub fn new(cef_msg_channel: UnboundedSender<TabMessage>) -> Self {
        Self { cef_msg_channel }
    }

    fn send_message(&self, message: TabMessage) {
        if let Err(e) = self.cef_msg_channel.send(message) {
            error!("Failed to send message: {:?}", e);
        }
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
                self.send_message(TabMessage::NewTabRequested(target_url.unwrap()));
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
            "closing tab: {}",
            browser.get_identifier().expect("failed to get tab id")
        );

        self.send_message(TabMessage::Closed);
        false
    }

    fn on_before_close(&mut self, _browser: Browser) {}
}
