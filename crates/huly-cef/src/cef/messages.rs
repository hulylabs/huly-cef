/// Represents different types of messages that can be sent from CEF to the browser.
#[derive(Debug, Serialize, Deserialize)]
pub enum CefMessage {
    /// Message to render a frame.
    Frame(Vec<u8>),
    /// Message indicating that the browser is loading a page.
    Loading,
    /// Message indicating that the browser has finished loading a page.
    Loaded,
    /// Message indicating that the browser has failed to load a page.
    LoadError {
        error_code: i32,
        error_text: String,
        failed_url: String,
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
}

use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseType {
    Left = 0,
    Middle = 1,
    Right = 2,
}

/// Represents different types of messages that can be sent from the browser to CEF.
#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserMessage {
    /// Message to load a new url.
    GoTo { url: String },
    /// Message to resize the browser.
    Resize { width: u32, height: u32 },
    /// Message indicating a mouse movement event.
    MouseMove { x: i32, y: i32 },
    /// Message indicating a mouse click event.
    MouseClick {
        x: i32,
        y: i32,
        button: MouseType,
        down: bool,
    },
    /// Message indicating a mouse scroll event.
    MouseWheel { x: i32, y: i32, dx: i32, dy: i32 },
    /// Message indicating a key press event.
    KeyPress {
        character: u16,
        code: i32,
        down: bool,
        ctrl: bool,
        shift: bool,
    },
    /// Message indicating that the browser is idle and should not be rendered.
    StopVideo,
    /// Message indicating that the browser is active and should be rendered.
    StartVideo,
    /// Message indicating that the browser is closing.
    Close,
    /// Message indicating that the browser is reloading.
    Reload,
    /// Message indicating that the browser should go back to the previous page.
    GoBack,
    /// Message indicating that the browser should go forward to the next page.
    GoForward,
}
