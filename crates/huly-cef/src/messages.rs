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
    pub dpr: f64,
    pub data: Vec<u8>,
}

impl Framebuffer {
    pub fn length_in_bytes(width: u32, height: u32, dpr: f64) -> usize {
        (width as f64 * dpr * height as f64 * dpr * 4.0) as usize
    }
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
    DownloadProgress,
    FileDialog,
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
    UrlHovered(String),
    NewTab(String),
    LoadState(LoadState),
    DownloadProgress {
        id: u32,
        path: String,
        received: u64,
        total: u64,
        is_complete: bool,
        is_aborted: bool,
    },
    FileDialog {
        mode: i32,
        title: String,
        default_file_path: String,
        accept_types: Vec<String>,
        accept_extensions: Vec<String>,
        accept_descriptions: Vec<String>,
    },
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
            TabMessage::DownloadProgress { .. } => TabMessageType::DownloadProgress,
            TabMessage::FileDialog { .. } => TabMessageType::FileDialog,
        }
    }
}
