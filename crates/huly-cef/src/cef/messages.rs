use serde::{Deserialize, Serialize};
use serde_repr::*;

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
    /// Message containin the queried elemnt's center coordinates.
    ElementCenter { x: i32, y: i32 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserMessage {
    pub id: String,
    pub tab_id: i32,
    pub body: BrowserMessageType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserMessageType {
    Close,
    RestoreSession,
    OpenTab(String),
    CloseTab,
    GetTabs,
    Resize {
        width: u32,
        height: u32,
    },
    TakeScreenshot,
    GoTo {
        url: String,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
    MouseClick {
        x: i32,
        y: i32,
        button: MouseType,
        down: bool,
    },
    MouseWheel {
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    },
    KeyPress {
        character: u16,
        code: i32,
        windowscode: i32,
        down: bool,
        ctrl: bool,
        shift: bool,
    },
    StopVideo,
    StartVideo,
    Reload,
    GoBack,
    GoForward,
    SetFocus(bool),
    GetDOM,
    GetElementCenter {
        selector: String,
    },
    SetText {
        selector: String,
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub id: String,
    pub tab_id: i32,
    pub body: ServerMessageType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessageType {
    Session(Vec<String>),
    Tab(i32),
    Tabs(Vec<String>),
    Screenshot(String),
    DOM(String),
    ElementCenter(i32, i32),
}
