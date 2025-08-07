import { Browser } from "./browser.js";
export { KeyCode } from "./keyboard.js";
export { Browser } from "./browser.js";
export { MouseButton, LoadState, Cursor } from "./types.js";
export { TabEventStream } from "./event_stream.js";
export { Tab } from "./tab.js";


export async function connect(serverAddress: string): Promise<Browser> {
    let serverUrl = new URL(serverAddress);
    const WebSocket = require('ws');
    const websocket = new WebSocket(serverUrl.toString());
    await new Promise<void>((resolve, reject) => {
        websocket.onopen = () => resolve();
        websocket.onerror = (error: any) => reject(error);
    });

    return new Browser(serverUrl, websocket);
}