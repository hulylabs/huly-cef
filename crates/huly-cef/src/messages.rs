use std::{
    hash::Hash,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[derive(Debug, Clone, Serialize_repr, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum LoadStatus {
    Loading = 0,
    Loaded = 1,
    LoadError = 2,
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Hash, PartialEq, Eq)]
pub enum TabMessageType {
    Frame,
    Cursor,
    Title,
    Url,
    Favicon,
    Closed,
    UrlHovered,
    NewTab,
    LoadState,
}

/// Represents different types of events that can be sent from CEF browser
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum TabMessage {
    Frame(Arc<Mutex<Framebuffer>>),
    Cursor(String),
    Title(String),
    Url(String),
    Favicon(String),
    Closed,
    UrlHovered { url: String, hovered: bool },
    NewTab(String),
    LoadState(LoadState),
}

impl TabMessage {
    pub fn event_type(&self) -> TabMessageType {
        match self {
            TabMessage::Frame(_) => TabMessageType::Frame,
            TabMessage::Cursor(_) => TabMessageType::Cursor,
            TabMessage::Title(_) => TabMessageType::Title,
            TabMessage::Url(_) => TabMessageType::Url,
            TabMessage::Favicon(_) => TabMessageType::Favicon,
            TabMessage::Closed => TabMessageType::Closed,
            TabMessage::UrlHovered { .. } => TabMessageType::UrlHovered,
            TabMessage::NewTab(_) => TabMessageType::NewTab,
            TabMessage::LoadState(_) => TabMessageType::LoadState,
        }
    }
}
