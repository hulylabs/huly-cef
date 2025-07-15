use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum LoadStatus {
    Loading = 0,
    Loaded = 1,
    LoadError = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadState {
    pub status: LoadStatus,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub error_code: i32,
    pub error_message: String,
}

impl Default for LoadState {
    fn default() -> Self {
        LoadState {
            status: LoadStatus::Loading,
            can_go_back: false,
            can_go_forward: false,
            error_code: 0,
            error_message: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickableElement {
    pub id: i32,
    pub tag: String,
    pub text: String,
}

/// Represents different types of messages that can be sent from CEF to the browser.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "data")]
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
    Cursor(String),
    /// Message indicating that title has changed.
    Title(String),
    /// Message indicating that URL has changed.
    Url(String),
    /// Message indicating that favicon has changed.
    Favicon(String),
    /// Message indicating that CEF has closed the browser.
    Closed,
    /// Message indicating that the mouse has hovered over a URL.
    UrlHovered { url: String, hovered: bool },
    /// Message indicating that a new tab has been requested.
    NewTab(String),
    /// Message indicating that load state has changed.
    LoadState(LoadState),
}
