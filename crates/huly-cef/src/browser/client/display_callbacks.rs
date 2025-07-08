use cef_ui::{
    Browser, CursorHandle, CursorInfo, CursorType, DisplayHandlerCallbacks, Frame, LogSeverity,
    Size,
};
use log::{error, info};
use url::Url;

use crate::{browser::state::SharedBrowserState, messages::TabMessage};

pub struct HulyDisplayHandlerCallbacks {
    state: SharedBrowserState,
    hovered_url: Option<Url>,
}

impl HulyDisplayHandlerCallbacks {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            state,
            hovered_url: None,
        }
    }
}

impl DisplayHandlerCallbacks for HulyDisplayHandlerCallbacks {
    fn on_address_change(&mut self, _: Browser, _: Frame, url: &str) {
        self.state.update(|state| {
            state.url = url.to_string();
        });
        self.state.notify(TabMessage::Url(url.to_string()));
    }

    fn on_title_change(&mut self, _: Browser, title: Option<String>) {
        if let Some(title) = title {
            info!("Tab title changed: {}", title);
            self.state.update(|state| {
                state.title = title.clone();
            });
            self.state.notify(TabMessage::Title(title.to_string()));
        }
    }

    fn on_favicon_urlchange(&mut self, _: Browser, icon_urls: Vec<String>) {
        if !icon_urls.is_empty() {
            self.state.update(|state| {
                state.favicon = Some(icon_urls[0].to_string());
            });
            self.state
                .notify(TabMessage::Favicon(icon_urls[0].to_string()));
        }
    }

    fn on_fullscreen_mode_change(&mut self, _: Browser, _: bool) {}

    fn on_tooltip(&mut self, _: Browser, _: Option<String>) -> bool {
        false
    }

    fn on_status_message(&mut self, _: Browser, value: Option<String>) {
        if let Some(value) = value {
            let url = Url::parse(&value);
            if let Ok(url) = url {
                self.state.notify(TabMessage::UrlHovered {
                    url: url.to_string(),
                    hovered: true,
                });
                self.hovered_url = Some(url);
            }
        } else {
            self.state.notify(TabMessage::UrlHovered {
                url: "".to_string(),
                hovered: false,
            });
            self.hovered_url = None;
        }
    }

    fn on_console_message(
        &mut self,
        _: Browser,
        severity: LogSeverity,
        message: Option<String>,
        source: Option<String>,
        line: i32,
    ) -> bool {
        if severity == LogSeverity::Error {
            let message = message.unwrap_or_else(|| "No message".to_string());
            let source = source.unwrap_or_else(|| "Unknown source".to_string());
            error!("[{}: {}]: {}", source, line, message);
        }

        true
    }

    fn on_auto_resize(&mut self, _: Browser, _: &Size) -> bool {
        false
    }

    fn on_loading_progress_change(&mut self, _: Browser, _: f64) {}

    fn on_cursor_change(
        &mut self,
        _: Browser,
        _: CursorHandle,
        cursor_type: CursorType,
        _: Option<CursorInfo>,
    ) -> bool {
        self.state.update(|state| {
            state.cursor = format!("{:?}", cursor_type);
        });
        self.state
            .notify(TabMessage::Cursor(format!("{:?}", cursor_type)));
        true
    }

    fn on_media_access_change(&mut self, _: Browser, _: bool, _: bool) {}
}
