import { Browser } from "./browser.js";
export { KeyCode } from "./keyboard.js";
export { Browser } from "./browser.js";
export { MouseButton } from "./types.js";

export async function connect(serverAddress: string): Promise<Browser> {
    let serverUrl = new URL(serverAddress);
    const websocket = new WebSocket(serverUrl.toString());
    await new Promise<void>((resolve, reject) => {
        websocket.onopen = () => resolve();
        websocket.onerror = (error) => reject(error);
    });

    return new Browser(serverUrl, websocket);
}