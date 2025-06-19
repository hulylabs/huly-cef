use cef_ui::{
    Browser, BrowserSettings, Client, DictionaryValue, Frame, LifeSpanHandlerCallbacks,
    PopupFeatures, WindowInfo, WindowOpenDisposition,
};
use log::{self, error};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::TabMessage;

pub struct HulyLifeSpanHandlerCallbacks {
    event_channel: UnboundedSender<TabMessage>,
}

impl HulyLifeSpanHandlerCallbacks {
    pub fn new(event_channel: UnboundedSender<TabMessage>) -> Self {
        Self { event_channel }
    }

    fn send_message(&self, message: TabMessage) {
        if let Err(e) = self.event_channel.send(message) {
            error!("Failed to send message: {:?}", e);
        }
    }
}

impl LifeSpanHandlerCallbacks for HulyLifeSpanHandlerCallbacks {
    unsafe fn on_before_popup(
        &mut self,
        _: Browser,
        _: Frame,
        _: i32,
        target_url: Option<String>,
        _: Option<String>,
        target_disposition: WindowOpenDisposition,
        _: bool,
        _: PopupFeatures,
        _: &mut WindowInfo,
        _: &mut Option<Client>,
        _: &mut BrowserSettings,
        _: &mut Option<DictionaryValue>,
        _: &mut bool,
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
        _: Browser,
        _: &mut WindowInfo,
        _: &mut Option<Client>,
        _: &mut BrowserSettings,
        _: &mut Option<DictionaryValue>,
        _: &mut bool,
    ) {
    }

    fn on_after_created(&mut self, _: Browser) {}

    fn do_close(&mut self, browser: Browser) -> bool {
        log::info!(
            "closing tab: {}",
            browser.get_identifier().expect("failed to get tab id")
        );

        self.send_message(TabMessage::Closed);
        false
    }

    fn on_before_close(&mut self, _: Browser) {}
}
