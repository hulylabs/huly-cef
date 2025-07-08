use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_repr::*;

use crate::browser::ClickableElement;

// TODO: This file looks a bit messy, consider refactoring it later.

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseType {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum LoadStatus {
    Loading,
    Loaded,
    LoadError,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum TabEventType {
    Frame,
    Popup,
    CursorChanged,
    TitleChanged,
    UrlChanged,
    FaviconUrlChanged,
    Closed,
    UrlHovered,
    NewTabRequested,
    LoadStateChanged,
}

impl From<&TabMessage> for TabEventType {
    fn from(message: &TabMessage) -> Self {
        match message {
            TabMessage::Frame(_) => TabEventType::Frame,
            TabMessage::Popup { .. } => TabEventType::Popup,
            TabMessage::CursorChanged(_) => TabEventType::CursorChanged,
            TabMessage::TitleChanged(_) => TabEventType::TitleChanged,
            TabMessage::UrlChanged(_) => TabEventType::UrlChanged,
            TabMessage::FaviconUrlChanged(_) => TabEventType::FaviconUrlChanged,
            TabMessage::Closed => TabEventType::Closed,
            TabMessage::UrlHovered { .. } => TabEventType::UrlHovered,
            TabMessage::NewTabRequested(_) => TabEventType::NewTabRequested,
            TabMessage::LoadStateChanged { .. } => TabEventType::LoadStateChanged,
        }
    }
}

/// Represents different types of messages that can be sent from CEF to the browser.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub enum TabMessage {
    /// Message to render a frame.
    Frame(Vec<u8>),
    ///Message to render a popup frame.
    Popup {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
    /// Message indicating that cursor has changed.
    CursorChanged(String),
    /// Message indicating that title has changed.
    TitleChanged(String),
    /// Message indicating that URL has changed.
    UrlChanged(String),
    /// Message indicating that favicon has changed.
    FaviconUrlChanged(String),
    /// Message indicating that CEF has closed the browser.
    Closed,
    /// Message indicating that the mouse has hovered over a URL.
    UrlHovered { url: String, hovered: bool },
    /// Message indicating that a new tab has been requested.
    NewTabRequested(String),
    /// Message indicating that load state has changed.
    LoadStateChanged {
        status: LoadStatus,
        can_go_back: bool,
        can_go_forward: bool,
        error_code: i32,
        error_text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub size: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenTabOptions {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserMessage {
    pub id: String,
    pub tab_id: i32,
    pub body: BrowserMessageType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BrowserMessageType {
    Close,
    RestoreSession,
    OpenTab {
        options: Option<OpenTabOptions>,
    },
    CloseTab,
    GetTabs,
    Resize {
        width: u32,
        height: u32,
    },
    GetTitle,
    GetUrl,
    Screenshot {
        options: Option<ScreenshotOptions>,
    },
    Navigate {
        url: String,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
    Click {
        x: i32,
        y: i32,
        button: MouseType,
        down: bool,
    },
    Wheel {
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    },
    Key {
        character: u16,
        code: i32,
        windowscode: i32,
        down: bool,
        ctrl: bool,
        shift: bool,
    },
    Char(u16),
    StopVideo,
    StartVideo,
    Reload,
    GoBack,
    GoForward,
    SetFocus(bool),
    GetDOM,
    GetClickableElements,
    ClickElement(i32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub id: String,
    pub tab_id: i32,
    pub body: ServerMessageType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessageType {
    Session(Vec<String>),
    Tab(i32),
    Tabs(Vec<i32>),
    Title(String),
    Url(String),
    Screenshot(String),
    DOM(String),
    ClickableElements(Vec<ClickableElement>),
}
