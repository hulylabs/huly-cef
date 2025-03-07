use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseType {
    Left = 0,
    Middle = 1,
    Right = 2,
}

/// Represents different types of messages that can be sent over the WebSocket.
#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessage {
    /// Message to create a new browser instance.
    CreateBrowser {
        url: String,
        width: u32,
        height: u32,
    },
    /// Message to indicate a mouse movement event.
    MouseMove {
        x: i32,
        y: i32,
    },
    /// Message to indicate a mouse click event.
    MouseClick {
        x: i32,
        y: i32,
        button: MouseType,
        down: bool,
    },
    Scroll {
        dx: i32,
        dy: i32,
    },
    KeyPress {
        key: char,
        down: bool,
    },
    SetIdle,
    SetActive,
}
