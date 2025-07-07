import { Browser } from "./browser.js";
export { KeyCode } from "./keyboard.js";

export async function connect(url: string): Promise<Browser> {
    const websocket = new WebSocket(url);
    await new Promise<void>((resolve, reject) => {
        websocket.onopen = () => resolve();
        websocket.onerror = (error) => reject(error);
    });

    return new Browser(websocket);
}