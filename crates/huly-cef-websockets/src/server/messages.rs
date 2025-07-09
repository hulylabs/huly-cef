use huly_cef::{ClickableElement, MouseButton};
use serde::{Deserialize, Serialize};

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
    // Browser control messages
    Close,
    RestoreSession,
    OpenTab {
        options: Option<OpenTabOptions>,
    },
    GetTabs,
    Resize {
        width: u32,
        height: u32,
    },

    // Tab control messages
    CloseTab,
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
        button: MouseButton,
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
    Char {
        unicode: u16,
    },
    StopVideo,
    StartVideo,
    Reload,
    GoBack,
    GoForward,
    SetFocus(bool),
    GetDOM,
    GetClickableElements,
    ClickElement {
        id: i32,
    },
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
