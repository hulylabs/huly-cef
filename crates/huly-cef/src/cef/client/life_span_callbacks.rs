use cef_ui::{
    Browser, BrowserSettings, Client, DictionaryValue, Frame, LifeSpanHandlerCallbacks,
    PopupFeatures, WindowInfo, WindowOpenDisposition,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

pub struct HulyLifeSpanHandlerCallbacks {
    cef_message_channel: UnboundedSender<CefMessage>,
}

impl HulyLifeSpanHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<CefMessage>) -> Self {
        Self {
            cef_message_channel,
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
            WindowOpenDisposition::NewForegroundTab => {
                _ = self
                    .cef_message_channel
                    .send(CefMessage::NewTabRequested(target_url.unwrap()));
            }
            WindowOpenDisposition::NewBackgroundTab => {
                _ = self
                    .cef_message_channel
                    .send(CefMessage::NewTabRequested(target_url.unwrap()));
            }
            WindowOpenDisposition::NewWindow => {
                _ = self
                    .cef_message_channel
                    .send(CefMessage::NewTabRequested(target_url.unwrap()));
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

    fn do_close(&mut self, _browser: Browser) -> bool {
        println!("do_close is called");
        let result = self.cef_message_channel.send(CefMessage::Closed);

        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send message: {:?}", e);
            }
        }
        true
    }

    fn on_before_close(&mut self, _browser: Browser) {}
}
