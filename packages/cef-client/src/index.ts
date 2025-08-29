import { Browser } from "./browser.js";
export { Config, setConfig, getConfig } from "./config.js";
export { KeyCode } from "./keyboard.js";
export { Browser } from "./browser.js";
export { MouseButton, LoadState, LoadStatus, Cursor } from "./types.js";
export { TabEventStream } from "./event_stream.js";
export { Tab } from "./tab.js";


export async function connect(serverAddress: string): Promise<Browser> {
    let serverUrl = new URL(serverAddress);
    const websocket = new WebSocket(serverUrl.toString());
    await new Promise<void>((resolve, reject) => {
        websocket.onopen = () => resolve();
        websocket.onerror = (error: Event) => reject(error);
    });

    return new Browser(serverUrl, websocket);
}