use cef_ui::{Browser, CursorHandle, CursorInfo, CursorType, DisplayHandlerCallbacks, Frame, Size};
use tokio::sync::mpsc::UnboundedSender;

use crate::cef::messages::CefMessage;

pub struct HulyDisplayHandlerCallbacks {
    cef_message_channel: UnboundedSender<CefMessage>,
}

impl HulyDisplayHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<CefMessage>) -> Self {
        Self {
            cef_message_channel,
        }
    }
}

impl DisplayHandlerCallbacks for HulyDisplayHandlerCallbacks {
    fn on_address_change(&mut self, _browser: Browser, _frame: Frame, url: &str) {
        _ = self
            .cef_message_channel
            .send(CefMessage::UrlChanged(url.to_string()));
    }

    fn on_title_change(&mut self, _browser: Browser, title: &str) {
        _ = self
            .cef_message_channel
            .send(CefMessage::TitleChanged(title.to_string()));
    }

    fn on_favicon_urlchange(&mut self, _browser: Browser, _icon_urls: Vec<String>) {}

    fn on_fullscreen_mode_change(&mut self, _browser: Browser, _fullscreen: bool) {}

    fn on_tooltip(&mut self, _browser: Browser, _text: Option<String>) -> bool {
        false
    }

    fn on_status_message(&mut self, _browser: Browser, _value: Option<String>) {}

    fn on_console_message(
        &mut self,
        _browser: Browser,
        _level: cef_ui::LogSeverity,
        _message: Option<String>,
        _source: Option<String>,
        _line: i32,
    ) -> bool {
        false
    }

    fn on_auto_resize(&mut self, _browser: Browser, _new_size: &Size) -> bool {
        false
    }

    fn on_loading_progress_change(&mut self, _browser: Browser, _progress: f64) {}

    fn on_cursor_change(
        &mut self,
        _browser: Browser,
        _cursor: CursorHandle,
        cursor_type: CursorType,
        _custom_cursor_info: Option<CursorInfo>,
    ) -> bool {
        _ = self
            .cef_message_channel
            .send(CefMessage::CursorChanged(format!("{:?}", cursor_type)));

        true
    }

    fn on_media_access_change(&mut self, _browser: Browser, _has_video: bool, _has_audio: bool) {}
}
