use cef_ui::ProcessMessage;
use log::info;
use serde::{Deserialize, Serialize};

use crate::ClickableElement;

pub struct Request {
    pub id: String,
    pub body: RequestBody,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RequestBody {
    GetClickableElements,
    GetElementCenter { selector: String },
    CheckElementClicked { selector: String },
}

impl Into<ProcessMessage> for Request {
    fn into(self) -> ProcessMessage {
        let message = ProcessMessage::new(&self.id);
        let arg_list = message
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get arg list");

        let json = serde_json::to_string(&self.body).expect("failed to serialize response to JSON");
        arg_list
            .set_string(0, &json)
            .expect("failed to set argument string");

        message
    }
}

impl From<ProcessMessage> for Request {
    fn from(msg: ProcessMessage) -> Self {
        let id = msg.get_name().expect("failed to get message name");
        let arg_list = msg
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get argument list");
        let json = arg_list
            .get_string(0)
            .ok()
            .flatten()
            .expect("failed to get argument string");

        let body = serde_json::from_str(&json).expect("failed to deserialize request from JSON");
        Request { id, body }
    }
}

pub struct Response {
    pub id: String,
    pub body: ResponseBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseBody {
    ClickableElements(Vec<ClickableElement>),
    ElementCenter { x: i32, y: i32 },
    Clicked(bool),
}

impl From<ProcessMessage> for Response {
    fn from(msg: ProcessMessage) -> Self {
        let id = msg.get_name().expect("failed to get message name");
        let arg_list = msg
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get argument list");
        let json = arg_list
            .get_string(0)
            .ok()
            .flatten()
            .expect("failed to get argument string");

        info!("IPC Response JSON: {}", json);

        let body = serde_json::from_str(&json).expect("failed to deserialize response from JSON");
        Response { id, body }
    }
}
