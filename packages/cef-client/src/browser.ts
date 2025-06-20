import { KeyCode, keyCodeToMacOSVirtualKey, keyCodeToWindowsVirtualKey } from "./keyboard.js";
import { v4 as uuidv4 } from 'uuid';

const REQUEST_TIMEOUT = 5000;

enum Platform {
    Windows,
    MacOS,
    Linux,
}

function detectPlatform(): Platform {
    const platform = navigator.userAgent;
    if (platform.includes("Windows")) {
        return Platform.Windows;
    }
    if (platform.includes("Mac")) {
        return Platform.MacOS;
    }
    return Platform.Linux;
}

export class BrowserClient {
    private websocket: WebSocket;
    private pendingMessages: string[] = [];

    private pendingPromises: Map<string, { resolve: (value: any) => void, reject: () => void }> = new Map();

    private platform: Platform = detectPlatform();

    constructor(url: string) {
        this.websocket = new WebSocket(url);

        this.websocket.onopen = () => {
            for (const message of this.pendingMessages) {
                this.websocket.send(message);
            };
        }

        this.websocket.onmessage = (event) => {
            let msg = JSON.parse(event.data);

            if (msg.body.Tab) {
                this.resolvePromise<number>(msg.id, msg.body.Tab);
            }

            if (msg.body.Tabs) {
                this.resolvePromise<string[]>(msg.id, msg.body.Tabs);
            }

            if (msg.body.Screenshot) {
                this.resolvePromise<string>(msg.id, msg.body.Screenshot);
            }

            if (msg.body.DOM) {
                this.resolvePromise<string>(msg.id, msg.body.DOM);
            }

            if (msg.body.ElementCenter) {
                this.resolvePromise<{ x: number, y: number }>(msg.id, msg.body.ElementCenter);
            }
        }
    }

    closeBrowser(): void {
        this.send(JSON.stringify({ id: "", body: "Close", tab_id: -1 }));
    }

    restoreSession(): Promise<String[]> {
        const id = uuidv4();
        return this.sendWithPromise<String[]>(id, JSON.stringify({
            id: id,
            tab_id: -1,
            body: "RestoreSession"
        }));
    }

    openTab(url?: string): Promise<number> {
        const id = uuidv4();
        return this.sendWithPromise<number>(id, JSON.stringify({
            id: id,
            tab_id: -1,
            body: {
                OpenTab: url
            },
        }));
    }

    closeTab(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "CloseTab"
        }));
    }

    getTabs(): Promise<number[]> {
        const id = uuidv4();
        return this.sendWithPromise<number[]>(id, JSON.stringify({
            id: id,
            tab_id: -1,
            body: "GetTabs"
        }));
    }

    resize(width: number, height: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: -1,
            body: {
                Resize: {
                    width: Math.floor(width),
                    height: Math.floor(height)
                }
            }
        }));
    }

    screenshot(tabId: number): Promise<string> {
        const id = uuidv4();
        return this.sendWithPromise<string>(id, JSON.stringify({
            id: id,
            tab_id: tabId,
            body: "TakeScreenshot"
        }));
    }

    goTo(tabId: number, url: string): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                GoTo: {
                    url
                }
            }
        }));
    }

    mouseMove(tabId: number, x: number, y: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                MouseMove: {
                    x,
                    y
                }
            }
        }));
    }

    mouseClick(tabId: number, x: number, y: number, button: number, down: boolean): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                MouseClick: {
                    x,
                    y,
                    button,
                    down
                }
            }
        }));
    }

    mouseWheel(tabId: number, x: number, y: number, dx: number, dy: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                MouseWheel: {
                    x,
                    y,
                    dx,
                    dy
                }
            }
        }));
    }

    keyPress(
        tabId: number,
        keycode: KeyCode,
        character: number,
        down: boolean,
        ctrl: boolean,
        shift: boolean,
    ) {
        let platformKeyCode = 0;
        switch (this.platform) {
            case Platform.Windows:
            case Platform.Linux:
                platformKeyCode = keyCodeToWindowsVirtualKey(keycode);
                break;
            case Platform.MacOS:
                platformKeyCode = keyCodeToMacOSVirtualKey(keycode);
                break;
        }
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                KeyPress: {
                    character: character,
                    windowscode: keyCodeToWindowsVirtualKey(keycode),
                    code: platformKeyCode,
                    down: down,
                    ctrl: ctrl,
                    shift: shift,
                },
            }
        }));
    }

    stopVideo(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "StopVideo"
        }));
    }

    startVideo(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "StartVideo"
        }));
    }

    reload(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "Reload"
        }));
    }

    goBack(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "GoBack"
        }));
    }

    goForward(tabId: number): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: "GoForward"
        }));
    }

    setFocus(tabId: number, focus: boolean): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                SetFocus: focus
            }
        }));
    }

    getDOM(tabId: number): Promise<string> {
        const id = uuidv4();
        return this.sendWithPromise<string>(id, JSON.stringify({
            id: id,
            tab_id: tabId,
            body: "GetDOM"
        }));
    }

    getElementCenter(tabId: number, selector: string): Promise<{ x: number, y: number }> {
        const id = uuidv4();
        return this.sendWithPromise<{ x: number, y: number }>(id, JSON.stringify({
            id: id,
            tab_id: tabId,
            body: {
                GetElementCenter: { selector: selector }
            }
        }));
    }

    setText(tabId: number, selector: string, text: string): void {
        this.send(JSON.stringify({
            id: "",
            tab_id: tabId,
            body: {
                SetText: {
                    selector: selector,
                    text: text
                }
            }
        }));
    }

    private sendWithPromise<T>(id: string, message: string): Promise<T> {
        return new Promise<T>((resolve, reject) => {
            this.pendingPromises.set(id, { resolve, reject });
            this.send(message);
            setTimeout(() => {
                if (this.pendingPromises.has(id)) {
                    this.pendingPromises.delete(id);
                    reject(new Error("Timeout waiting for response"));
                }
            }, REQUEST_TIMEOUT);
        });
    }

    private resolvePromise<T>(id: string, value: T): void {
        const pendingPromise = this.pendingPromises.get(id);
        if (pendingPromise) {
            pendingPromise.resolve(value);
            this.pendingPromises.delete(id);
        }
    }

    private send(message: string): void {
        if (this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(message);
        } else {
            this.pendingMessages.push(message);
        }
    }
}
