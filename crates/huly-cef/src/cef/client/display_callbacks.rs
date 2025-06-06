use cef_ui::{Browser, CursorHandle, CursorInfo, CursorType, DisplayHandlerCallbacks, Frame, Size};
use log::error;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;

use crate::cef::messages::TabMessage;

pub struct HulyDisplayHandlerCallbacks {
    cef_message_channel: UnboundedSender<TabMessage>,
    hovered_url: Option<Url>,
}

impl HulyDisplayHandlerCallbacks {
    pub fn new(cef_message_channel: UnboundedSender<TabMessage>) -> Self {
        Self {
            cef_message_channel,
            hovered_url: None,
        }
    }

    fn send_message(&self, message: TabMessage) {
        if let Err(e) = self.cef_message_channel.send(message) {
            error!("failed to send message: {}", e);
        }
    }
}

impl DisplayHandlerCallbacks for HulyDisplayHandlerCallbacks {
    fn on_address_change(&mut self, _browser: Browser, _frame: Frame, url: &str) {
        self.send_message(TabMessage::UrlChanged(url.to_string()));
    }

    fn on_title_change(&mut self, _browser: Browser, title: Option<String>) {
        if let Some(title) = title {
            self.send_message(TabMessage::TitleChanged(title.to_string()));
        }
    }

    fn on_favicon_urlchange(&mut self, _browser: Browser, icon_urls: Vec<String>) {
        if !icon_urls.is_empty() {
            self.send_message(TabMessage::FaviconUrlChanged(icon_urls[0].to_string()));
        }
    }

    fn on_fullscreen_mode_change(&mut self, _browser: Browser, _fullscreen: bool) {}

    fn on_tooltip(&mut self, _browser: Browser, _text: Option<String>) -> bool {
        false
    }

    fn on_status_message(&mut self, _browser: Browser, value: Option<String>) {
        if let Some(value) = value {
            let url = Url::parse(&value);
            if let Ok(url) = url {
                self.send_message(TabMessage::UrlHovered {
                    url: url.to_string(),
                    hovered: true,
                });
                self.hovered_url = Some(url);
            }
        } else {
            self.send_message(TabMessage::UrlHovered {
                url: "".to_string(),
                hovered: false,
            });
            self.hovered_url = None;
        }
    }

    fn on_console_message(
        &mut self,
        _browser: Browser,
        _level: cef_ui::LogSeverity,
        _message: Option<String>,
        _source: Option<String>,
        _line: i32,
    ) -> bool {
        true
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
        self.send_message(TabMessage::CursorChanged(format!("{:?}", cursor_type)));
        true
    }

    fn on_media_access_change(&mut self, _browser: Browser, _has_video: bool, _has_audio: bool) {}
}
