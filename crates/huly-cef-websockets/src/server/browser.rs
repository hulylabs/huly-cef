use futures::{SinkExt, StreamExt};
use huly_cef::{browser::Browser, MouseButton};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use crate::server::SharedServerState;
use log::{error, info};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

pub async fn handle(state: SharedServerState, mut websocket: WebSocketStream<TcpStream>) {
    while let Some(msg) = websocket.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to read a message: {:?}", e);
                continue;
            }
        };

        if msg.is_close() {
            break;
        }

        let request = match serde_json::from_slice::<Request>(&msg.clone().into_data()) {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to deserialize message: {:?} (error: {:?})", msg, e);
                continue;
            }
        };

        // TODO: use registry for handlers
        let result = match request.method.as_str() {
            "openTab" => match params(request.params) {
                Ok(params) => open_tab(&state, params).await,
                Err(err) => Err(err),
            },
            "closeTab" => params(request.params).and_then(|params| close_tab(&state, params)),
            "getTabs" => params(request.params).and_then(|_: EmptyParams| tabs(&state)),
            "getTitle" => params(request.params).and_then(|params| title(&state, params)),
            "getUrl" => params(request.params).and_then(|params| url(&state, params)),
            "resize" => params(request.params).and_then(|params| resize(&state, params)),
            "screenshot" => match params(request.params) {
                Ok(params) => screenshot(&state, params).await,
                Err(err) => Err(err),
            },
            "navigate" => params(request.params).and_then(|params| navigate(&state, params)),
            "mouseMove" => params(request.params).and_then(|params| mouse_move(&state, params)),
            "click" => params(request.params).and_then(|params| click(&state, params)),
            "wheel" => params(request.params).and_then(|params| wheel(&state, params)),
            "key" => params(request.params).and_then(|params| key(&state, params)),
            "char" => params(request.params).and_then(|params| char(&state, params)),
            "stopVideo" => params(request.params).and_then(|params| stop_video(&state, params)),
            "startVideo" => params(request.params).and_then(|params| start_video(&state, params)),
            "reload" => params(request.params).and_then(|params| reload(&state, params)),
            "goBack" => params(request.params).and_then(|params| go_back(&state, params)),
            "goForward" => params(request.params).and_then(|params| go_forward(&state, params)),
            "setFocus" => params(request.params).and_then(|params| set_focus(&state, params)),
            // "getDOM" => params(request.params).and_then(|params| Ok(get_dom(&state, params).await)),
            // "getClickableElements" => params(request.params).and_then(|params| Ok(get_clickable_elements(&state, params).await)),
            // "clickElement" => params(request.params).and_then(|params| click_element(&state, params).await),
            _ => Err(json!( {"message": "unknown method"} )),
        };

        let response = match result {
            Ok(v) => Response {
                id: request.id,
                result: Some(v),
                error: None,
            },
            Err(v) => Response {
                id: request.id,
                result: None,
                error: Some(v),
            },
        };

        let response =
            serde_json::to_string(&response).expect("failed to serialize response message");
        websocket
            .send(tungstenite::Message::Text(response.into()))
            .await
            .expect("failed to send session message");
    }
}

#[derive(Debug, Deserialize)]
struct EmptyParams;

#[derive(Debug, Deserialize)]
struct DefaultParams {
    id: i32,
}

#[derive(Debug, Deserialize)]
struct OpenTabParams {
    url: String,
    wait_until_loaded: bool,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct ResizeParams {
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct ScreenshotParams {
    tab: i32,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct NavigateParams {
    tab: i32,
    url: String,
    wait_until_loaded: bool,
}

#[derive(Debug, Deserialize)]
struct ClickParams {
    tab: i32,
    x: i32,
    y: i32,
    button: MouseButton,
    down: bool,
}

#[derive(Debug, Deserialize)]
struct MouseMoveParams {
    tab: i32,
    x: i32,
    y: i32,
}

#[derive(Debug, Deserialize)]
struct WheelParams {
    tab: i32,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
}

#[derive(Debug, Deserialize)]
struct KeyParams {
    tab: i32,
    character: u16,
    windowscode: i32,
    code: i32,
    down: bool,
    ctrl: bool,
    shift: bool,
}

#[derive(Debug, Deserialize)]
struct CharParams {
    tab: i32,
    unicode: u16,
}

#[derive(Debug, Deserialize)]
struct SetFocusParams {
    tab: i32,
    focus: bool,
}

fn get_tab(state: &SharedServerState, id: i32) -> Result<Browser, serde_json::Value> {
    state.get_tab(id).ok_or_else(|| {
        json!({
            "error": {
                "message": format!("tab with id {} not found", id)
            }
        })
    })
}

fn params<T: DeserializeOwned>(params: serde_json::Value) -> Result<T, serde_json::Value> {
    serde_json::from_value(params).map_err(|e| {
        error!("failed to deserialize params: {:?}", e);
        json!({
            "error": {
                "message": format!("failed to deserialize params: {}", e)
            }
        })
    })
}

async fn open_tab(
    state: &SharedServerState,
    params: OpenTabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    info!("opening a new tab with url: {}", params.url);

    let tab = Browser::new(params.width, params.height, &params.url);
    let id = tab.get_id();
    state.set_tab(id, tab.clone());

    // TODO: return json object instead of printing errors
    if params.wait_until_loaded {
        let result = tab.automation.wait_until_loaded().await;
        match result {
            Ok(_) => info!("tab with id {} is loaded", id),
            Err(e) => error!("failed to wait until tab with id {} is loaded: {}", id, e),
        }
    }

    info!("tab with id {} opened", id);

    Ok(json!({
        "id": id,
        "url": params.url,
        "width": params.width,
        "height": params.height,
    }))
}

fn close_tab(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = state.remove_tab(params.id);
    if let Some(tab) = tab {
        tab.close();
        Ok(json!({ "id": params.id }))
    } else {
        Err(json!({
            "message": format!("tab with id {} not found", params.id)
        }))
    }
}

fn tabs(state: &SharedServerState) -> Result<serde_json::Value, serde_json::Value> {
    let ids = {
        let state = state.lock();
        state
            .tabs
            .values()
            .map(|tab| tab.get_id().clone())
            .collect::<Vec<_>>()
    };

    Ok(json!({ "tabs": ids }))
}

fn resize(
    state: &SharedServerState,
    params: ResizeParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let state = state.lock();
    state
        .tabs
        .iter()
        .for_each(|t| t.1.resize(params.width, params.height));

    Ok(json!({}))
}

async fn screenshot(
    state: &SharedServerState,
    params: ScreenshotParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = match state.get_tab(params.tab) {
        Some(tab) => tab,
        None => {
            return Err(json!({ "message": "tab not found" }));
        }
    };
    let result = tab.automation.screenshot(params.width, params.height).await;
    match result {
        Ok(data) => Ok(json!({ "screenshot": data })),
        Err(e) => Err(json!({
            "message": format!("failed to take screenshot: {}", e)
        })),
    }
}

fn title(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    Ok(json!({ "title": tab.get_title() }))
}

fn url(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    Ok(json!({ "url": tab.get_url() }))
}

fn navigate(
    state: &SharedServerState,
    params: NavigateParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.go_to(&params.url);

    // TODO: add waiting if `wait_until_loaded` is true

    Ok(json!({}))
}

fn mouse_move(
    state: &SharedServerState,
    params: MouseMoveParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse.move_to(params.x, params.y);

    Ok(json!({}))
}

fn click(
    state: &SharedServerState,
    params: ClickParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse
        .click(params.x, params.y, params.button, params.down);

    Ok(json!({}))
}

fn wheel(
    state: &SharedServerState,
    params: WheelParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse.wheel(params.x, params.y, params.dx, params.dy);

    Ok(json!({}))
}

fn key(
    state: &SharedServerState,
    params: KeyParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.keyboard.key(
        params.character,
        params.code,
        params.windowscode,
        params.down,
        params.ctrl,
        params.shift,
    );

    Ok(json!({}))
}

fn char(
    state: &SharedServerState,
    params: CharParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.keyboard.char(params.unicode);

    Ok(json!({}))
}

fn stop_video(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    tab.stop_video();

    Ok(json!({}))
}

fn start_video(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    tab.start_video();

    Ok(json!({}))
}

fn reload(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    tab.reload();

    Ok(json!({}))
}

fn go_back(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    tab.go_back();

    Ok(json!({}))
}

fn go_forward(
    state: &SharedServerState,
    params: DefaultParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.id)?;
    tab.go_forward();

    Ok(json!({}))
}

fn set_focus(
    state: &SharedServerState,
    params: SetFocusParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.set_focus(params.focus);

    Ok(json!({}))
}
