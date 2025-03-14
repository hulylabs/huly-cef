/// Represents different types of messages that can be sent from CEF to the browser.
#[derive(Debug, Serialize, Deserialize)]
pub enum CefMessage {
    /// Message to render a frame.
    Render(Vec<u8>),
    /// Message to indicate that the browser is loading a page.
    Loading,
    /// Message to indicate that the browser has finished loading a page.
    Loaded,
    /// Message to indicate that the browser has failed to load a page.
    LoadError,
    /// Message to indicate that cursor has changed.
    CursorChanged(String),
    /// Message to indicate that title has changed.
    TitleChanged(String),
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
    /// Message to create a new browser instance.
    CreateBrowser {
        url: String,
        width: u32,
        height: u32,
    },
    /// Message to resize the browser.
    Resize { width: u32, height: u32 },
    /// Message to indicate a mouse movement event.
    MouseMove { x: i32, y: i32 },
    /// Message to indicate a mouse click event.
    MouseClick {
        x: i32,
        y: i32,
        button: MouseType,
        down: bool,
    },
    /// Message to indicate a mouse scroll event.
    MouseWheel { x: i32, y: i32, dx: i32, dy: i32 },
    /// Message to indicate a key press event.
    KeyPress {
        character: u16,
        code: i32,
        down: bool,
    },
    /// Message to indicate that the browser is idle and should not be rendered.
    SetIdle,
    /// Message to indicate that the browser is active and should be rendered.
    SetActive,
}
