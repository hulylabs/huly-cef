use std::path::PathBuf;

use futures::{SinkExt, StreamExt};
use huly_cef::{browser::Browser, MouseButton};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use crate::server::SharedServerState;
use log::{error, info};
use tokio::{fs, net::TcpStream};
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

        let result = match request.method.as_str() {
            "close" => close(&state).await,
            "restore" => restore(&state).await,
            "openTab" => match parse_params(request.params) {
                Ok(params) => open_tab(&state, params).await,
                Err(err) => Err(err),
            },
            "screenshot" => match parse_params(request.params) {
                Ok(params) => screenshot(&state, params).await,
                Err(err) => Err(err),
            },
            "navigate" => match parse_params(request.params) {
                Ok(params) => navigate(&state, params).await,
                Err(err) => Err(err),
            },
            "getDOM" => match parse_params(request.params) {
                Ok(params) => get_dom(&state, params).await,
                Err(err) => Err(err),
            },
            "getClickableElements" => match parse_params(request.params) {
                Ok(params) => get_clickable_elements(&state, params).await,
                Err(err) => Err(err),
            },
            "clickElement" => match parse_params(request.params) {
                Ok(params) => click_element(&state, params).await,
                Err(err) => Err(err),
            },
            "reload" => match parse_params(request.params) {
                Ok(params) => reload(&state, params).await,
                Err(err) => Err(err),
            },
            "goBack" => match parse_params(request.params) {
                Ok(params) => go_back(&state, params).await,
                Err(err) => Err(err),
            },
            "goForward" => match parse_params(request.params) {
                Ok(params) => go_forward(&state, params).await,
                Err(err) => Err(err),
            },
            method => handle_sync_method(&state, method, request.params),
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

fn handle_sync_method(
    state: &SharedServerState,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    match method {
        "closeTab" => parse_params(params).and_then(|params| close_tab(&state, params)),
        "getTabs" => parse_params(params).and_then(|_: EmptyParams| tabs(&state)),
        "getSize" => parse_params(params).and_then(|_: EmptyParams| size(&state)),
        "getTitle" => parse_params(params).and_then(|params| title(&state, params)),
        "getUrl" => parse_params(params).and_then(|params| url(&state, params)),
        "resize" => parse_params(params).and_then(|params| resize(&state, params)),
        "mouseMove" => parse_params(params).and_then(|params| mouse_move(&state, params)),
        "click" => parse_params(params).and_then(|params| click(&state, params)),
        "wheel" => parse_params(params).and_then(|params| wheel(&state, params)),
        "key" => parse_params(params).and_then(|params| key(&state, params)),
        "char" => parse_params(params).and_then(|params| char(&state, params)),
        "stopVideo" => parse_params(params).and_then(|params| stop_video(&state, params)),
        "startVideo" => parse_params(params).and_then(|params| start_video(&state, params)),
        "setFocus" => parse_params(params).and_then(|params| set_focus(&state, params)),
        "undo" => parse_params(params).and_then(|params| undo(&state, params)),
        "redo" => parse_params(params).and_then(|params| redo(&state, params)),
        "selectAll" => parse_params(params).and_then(|params| select_all(&state, params)),
        "copy" => parse_params(params).and_then(|params| copy(&state, params)),
        "paste" => parse_params(params).and_then(|params| paste(&state, params)),
        "cut" => parse_params(params).and_then(|params| cut(&state, params)),
        "delete" => parse_params(params).and_then(|params| delete(&state, params)),
        "continueFileDialog" => {
            parse_params(params).and_then(|params| continue_file_dialog(&state, params))
        }
        "cancelFileDialog" => {
            parse_params(params).and_then(|params| cancel_file_dialog(&state, params))
        }
        "cancelDownloading" => {
            parse_params(params).and_then(|params| cancel_downloading(&state, params))
        }
        _ => {
            error!("method not found: {}", method);
            Err(json!({
                "message": "Method not found",
                "data": { "method": method }
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
struct EmptyParams {}

#[derive(Debug, Deserialize)]
struct TabParams {
    tab: i32,
}

fn default_dpr() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
struct OpenTabParamss {
    url: String,
    wait_until_loaded: bool,
    #[serde(default = "default_dpr")]
    dpr: f64,
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
    #[serde(default)]
    url: String,
    #[serde(default)]
    wait_until_loaded: bool,
}

#[derive(Debug, Deserialize)]
struct PositionParams {
    tab: i32,
    x: i32,
    y: i32,
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

#[derive(Debug, Deserialize)]
struct ClickElementParams {
    tab: i32,
    element_id: i32,
}

#[derive(Debug, Deserialize)]
struct ContinueFileDialogParams {
    tab: i32,
    filepaths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CancelDownloadingParams {
    tab: i32,
    download_id: u32,
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

fn parse_params<T: DeserializeOwned>(params: serde_json::Value) -> Result<T, serde_json::Value> {
    serde_json::from_value(params.clone()).map_err(|e| {
        error!("failed to deserialize params {}: {}", params, e);
        json!({
            "message": format!("failed to deserialize params {}: {}", params, e)
        })
    })
}

async fn save_session(state: &SharedServerState) -> Result<(), String> {
    let (file_path, urls) = {
        let state = state.lock();
        let file_path = PathBuf::from(state.cache_dir.clone()).join("session.json");
        let urls = state.tabs.values().map(|t| t.get_url()).collect::<Vec<_>>();
        (file_path, urls)
    };

    let data =
        serde_json::to_string(&urls).map_err(|e| format!("failed to serialize session: {}", e))?;

    fs::write(file_path, data)
        .await
        .map_err(|e| format!("failed to write session file: {}", e))
}

async fn restore_session(state: &SharedServerState) -> Result<Vec<String>, String> {
    let file_path = PathBuf::from(state.lock().cache_dir.clone()).join("session.json");

    let data = fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("failed to read session file: {}", e))?;

    serde_json::from_str(&data).map_err(|e| format!("failed to parse session file: {}", e))
}

async fn close(state: &SharedServerState) -> Result<serde_json::Value, serde_json::Value> {
    if let Err(result) = save_session(state).await {
        error!("failed to save session: {}", result);
    }

    match state.lock().shutdown_tx.send(()) {
        Ok(_) => Ok(json!({ "success": true })),
        Err(_) => {
            error!("failed to send shutdown signal");
            return Err(json!({ "message": "failed to send shutdown signal" }));
        }
    }
}

async fn restore(state: &SharedServerState) -> Result<serde_json::Value, serde_json::Value> {
    match restore_session(state).await {
        Ok(urls) => Ok(json!({ "urls": urls })),
        Err(e) => {
            error!("failed to restore session: {}", e);
            Err(json!({ "message": format!("failed to restore session: {}", e) }))
        }
    }
}

async fn open_tab(
    state: &SharedServerState,
    params: OpenTabParamss,
) -> Result<serde_json::Value, serde_json::Value> {
    let (width, height) = { state.lock().size };
    info!(
        "[open_tab] size: ({}, {}), url: {}",
        width, height, params.url
    );
    let mut tab = Browser::new(width, height, params.dpr, &params.url);
    let id = tab.get_id();
    state.set_tab(id, tab.clone());

    if params.wait_until_loaded {
        match tab.automation.wait_until_loaded().await {
            Ok(_) => info!("[open_tab] tab {} is loaded", id),
            Err(e) => {
                error!("[open_tab] tab {} hasn't loaded yet: {}", id, e);
                return Err(json!({
                    "message": format!("failed to wait for page load: {}", e),
                    "data": { "id": id, "url": params.url, "width": width, "height": height }
                }));
            }
        }
    }

    info!("[open_tab] tab {} opened", id);

    Ok(json!({
        "id": id,
        "url": params.url,
        "width": width,
        "height": height,
    }))
}

fn close_tab(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = state.remove_tab(params.tab);
    if let Some(tab) = tab {
        tab.close();
        Ok(json!({ "id": params.tab }))
    } else {
        Err(json!({
            "message": format!("tab with id {} not found", params.tab)
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

fn size(state: &SharedServerState) -> Result<serde_json::Value, serde_json::Value> {
    let state = state.lock();
    Ok(json!({
        "width": state.size.0,
        "height": state.size.1,
    }))
}

fn resize(
    state: &SharedServerState,
    params: ResizeParams,
) -> Result<serde_json::Value, serde_json::Value> {
    info!("[resize] ({}, {})", params.width, params.height);

    let mut state = state.lock();
    if state.use_server_size {
        error!("cannot resize, server size is used");
        return Err(json!({
            "message": "server size is used, cannot resize"
        }));
    }

    state.size = (params.width, params.height);
    state
        .tabs
        .iter()
        .for_each(|t| t.1.resize(params.width, params.height));

    Ok(json!({ "success": true }))
}

async fn screenshot(
    state: &SharedServerState,
    params: ScreenshotParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;

    match tab.automation.screenshot(params.width, params.height).await {
        Ok(data) => Ok(json!({ "screenshot": data })),
        Err(e) => Err(json!({
            "message": format!("failed to take screenshot: {}", e)
        })),
    }
}

fn title(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    Ok(json!({ "title": tab.get_title() }))
}

fn url(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    Ok(json!({ "url": tab.get_url() }))
}

async fn navigate(
    state: &SharedServerState,
    params: NavigateParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let mut tab = get_tab(state, params.tab)?;
    tab.go_to(&params.url);
    let id = tab.get_id();

    if params.wait_until_loaded {
        match tab.automation.wait_until_loaded().await {
            Ok(_) => info!("tab with id {} is loaded", id),
            Err(e) => {
                error!("failed to wait until tab with id {} is loaded: {}", id, e);
                return Err(json!({
                    "message": format!("failed to wait for page load: {}", e),
                }));
            }
        }
    }

    Ok(json!({ "success": true }))
}

fn mouse_move(
    state: &SharedServerState,
    params: PositionParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse.move_to(params.x, params.y);

    Ok(json!({ "success": true }))
}

fn click(
    state: &SharedServerState,
    params: ClickParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse
        .click(params.x, params.y, params.button, params.down);

    Ok(json!({ "success": true }))
}

fn wheel(
    state: &SharedServerState,
    params: WheelParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.mouse.wheel(params.x, params.y, params.dx, params.dy);

    Ok(json!({ "success": true }))
}

fn key(
    state: &SharedServerState,
    params: KeyParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.keyboard.key(
        params.character,
        params.windowscode,
        params.code,
        params.down,
        params.ctrl,
        params.shift,
    );

    Ok(json!({ "success": true }))
}

fn char(
    state: &SharedServerState,
    params: CharParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.keyboard.char(params.unicode);

    Ok(json!({ "success": true }))
}

fn stop_video(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    info!("[tab: {}] stop video", tab.get_title());
    tab.stop_video();

    Ok(json!({ "success": true }))
}

fn start_video(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    info!("[tab: {}] start video", tab.get_title());
    tab.start_video();

    Ok(json!({ "success": true }))
}

async fn reload(
    state: &SharedServerState,
    params: NavigateParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let mut tab = get_tab(state, params.tab)?;
    tab.reload();
    let id = tab.get_id();

    if params.wait_until_loaded {
        match tab.automation.wait_until_loaded().await {
            Ok(_) => info!("tab with id {} is loaded", id),
            Err(e) => {
                error!("failed to wait until tab with id {} is loaded: {}", id, e);
                return Err(json!({
                    "message": format!("failed to wait for page load: {}", e),
                }));
            }
        }
    }

    Ok(json!({ "success": true }))
}

async fn go_back(
    state: &SharedServerState,
    params: NavigateParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let mut tab = get_tab(state, params.tab)?;
    tab.go_back();
    let id = tab.get_id();

    if params.wait_until_loaded {
        match tab.automation.wait_until_loaded().await {
            Ok(_) => info!("tab with id {} is loaded", id),
            Err(e) => {
                error!("failed to wait until tab with id {} is loaded: {}", id, e);
                return Err(json!({
                    "message": format!("failed to wait for page load: {}", e),
                }));
            }
        }
    }

    Ok(json!({ "success": true }))
}

async fn go_forward(
    state: &SharedServerState,
    params: NavigateParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let mut tab = get_tab(state, params.tab)?;
    tab.go_forward();
    let id = tab.get_id();

    if params.wait_until_loaded {
        match tab.automation.wait_until_loaded().await {
            Ok(_) => info!("tab with id {} is loaded", id),
            Err(e) => {
                error!("failed to wait until tab with id {} is loaded: {}", id, e);
                return Err(json!({
                    "message": format!("failed to wait for page load: {}", e),
                }));
            }
        }
    }

    Ok(json!({ "success": true }))
}

fn set_focus(
    state: &SharedServerState,
    params: SetFocusParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.set_focus(params.focus);

    Ok(json!({ "success": true }))
}

async fn get_dom(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    let dom = tab.automation.get_dom().await;

    Ok(json!({ "dom": dom }))
}

async fn get_clickable_elements(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    let elements = tab.automation.get_clickable_elements().await;

    Ok(json!({ "elements": elements }))
}

async fn click_element(
    state: &SharedServerState,
    params: ClickElementParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    // TODO: check that element exists
    tab.automation.click_element(params.element_id).await;

    Ok(json!({ "success": true }))
}

fn delete(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.delete();

    Ok(json!({ "success": true }))
}

fn undo(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.undo();

    Ok(json!({ "success": true }))
}

fn redo(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.redo();

    Ok(json!({ "success": true }))
}

fn select_all(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.select_all();

    Ok(json!({ "success": true }))
}

fn copy(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.copy();

    Ok(json!({ "success": true }))
}

fn paste(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.paste();

    Ok(json!({ "success": true }))
}

fn cut(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.cut();

    Ok(json!({ "success": true }))
}

fn continue_file_dialog(
    state: &SharedServerState,
    params: ContinueFileDialogParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.continue_file_dialog(params.filepaths);

    Ok(json!({ "success": true }))
}

fn cancel_file_dialog(
    state: &SharedServerState,
    params: TabParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.cancel_file_dialog();

    Ok(json!({ "success": true }))
}

fn cancel_downloading(
    state: &SharedServerState,
    params: CancelDownloadingParams,
) -> Result<serde_json::Value, serde_json::Value> {
    let tab = get_tab(state, params.tab)?;
    tab.cancel_downloading(params.download_id);

    Ok(json!({ "success": true }))
}
