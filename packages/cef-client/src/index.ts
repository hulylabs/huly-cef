export enum LoadState {
    Loading,
    Loaded,
    Error,
}

// export enum CursorType {
//     Default = "default",
//     Pointer = "pointer",
//     Text = "text",
//     Wait = "wait",
//     Crosshair = "crosshair",
//     Move = "move",
//     NotAllowed = "not-allowed",
// }


const HEARTBEAT_INTERVAL = 5000;
const HEARTBEAT_TIMEOUT = 2500;

export class CEFClient {
    websocket: WebSocket;
    heartbeatInterval: number;
    heartbeatTimeout: number;
    lastPongTime: number;

    public onConnectionBroken: (() => void) | undefined;
    public onLoadStateChanged: ((state: LoadState) => void) | undefined;
    public onTitleChanged: ((title: string) => void) | undefined;
    public onUrlChanged: ((url: string) => void) | undefined;
    public onCursorChanged: ((cursor: string) => void) | undefined;
    public onNewTabRequested: ((url: string) => void) | undefined;
    public onRender: ((data: Uint8Array) => void) | undefined;

    constructor(websocket: WebSocket) {
        this.websocket = websocket;
        this.websocket.binaryType = "arraybuffer";
        this.lastPongTime = Date.now();
        this.heartbeatTimeout = -1;

        this.heartbeatInterval = setInterval(() => {
            if (this.websocket.readyState === WebSocket.CLOSED) {
                this.onConnectionBroken?.();
                return;
            }

            if (this.websocket.readyState === WebSocket.OPEN) {
                this.websocket.send("Ping");
            }

            if (this.heartbeatTimeout !== -1) {
                clearTimeout(this.heartbeatTimeout);
                this.heartbeatTimeout = -1;
            }

            this.heartbeatTimeout = setTimeout(() => {
                if (Date.now() - this.lastPongTime > HEARTBEAT_TIMEOUT) {
                    console.log("Heartbeat timeout, closing connection.");
                    this.onConnectionBroken?.();
                } else {
                    console.log("Heartbeat timeout, but still connected.");
                }
            }, HEARTBEAT_TIMEOUT);
        }, HEARTBEAT_INTERVAL);

        this.websocket.onmessage = (event) => {
            if (event.data instanceof ArrayBuffer) {
                let imageData = new Uint8Array(event.data);
                this.onRender?.(imageData);
            }

            if (typeof event.data === "string") {
                let parsed = JSON.parse(event.data);

                if (typeof parsed === "string") {
                    if (parsed === "Loading") {
                        this.onLoadStateChanged?.(LoadState.Loading);
                    }
                    if (parsed === "Loaded") {
                        this.onLoadStateChanged?.(LoadState.Loaded);
                    }
                    if (parsed === "LoadError") {
                        this.onLoadStateChanged?.(LoadState.Error);
                    }
                    if (parsed === "Pong") {
                        this.lastPongTime = Date.now();
                    }
                }

                if (typeof parsed === "object") {
                    if (parsed.TitleChanged) {
                        this.onTitleChanged?.(parsed.TitleChanged);
                    }

                    if (parsed.CursorChanged) {
                        this.onCursorChanged?.(parsed.CursorChanged);
                    }

                    if (parsed.UrlChanged) {
                        this.onUrlChanged?.(parsed.UrlChanged);
                    }

                    if (parsed.NewTabRequested) {
                        this.onNewTabRequested?.(parsed.NewTabRequested);
                    }
                }
            }
        }
    }

    goTo(url: string) {
        this.websocket.send(JSON.stringify({ GoTo: { url: url } }));
    };

    onMouseMove(x: number, y: number) {
        this.websocket.send(JSON.stringify({ MouseMove: { x: x, y: y } }));
    };

    onMouseDown(x: number, y: number, button: number) {
        this.websocket.send(JSON.stringify({ MouseClick: { x: x, y: y, down: true, button: button } }));
    };

    onMouseUp(x: number, y: number, button: number) {
        this.websocket.send(JSON.stringify({ MouseClick: { x: x, y: y, down: false, button: button } }));
    };

    onMouseWheel(x: number, y: number, dx: number, dy: number) {
        this.websocket.send(JSON.stringify({ MouseWheel: { x: x, y: y, dx: dx, dy: dy } }));
    };

    onKeyDown(key: string, code: number, ctrl: boolean, shift: boolean) {
        let char = 0;
        if (key.length === 1) {
            char = key.charCodeAt(0);
        }
        this.websocket.send(JSON.stringify({ KeyPress: { character: char, code: code, down: true, ctrl: ctrl, shift: shift } }));
    };

    onKeyUp(key: string, code: number, ctrl: boolean, shift: boolean) {
        let char = 0;
        if (key.length === 1) {
            char = key.charCodeAt(0);
        }
        this.websocket.send(JSON.stringify({ KeyPress: { character: char, code: code, down: false, ctrl: ctrl, shift: shift } }));
    };

    onResize(width: number, height: number) {
        this.websocket.send(JSON.stringify({ Resize: { width: Math.floor(width), height: Math.floor(height) } }));
    };

    startVideo() {
        this.websocket.send(JSON.stringify("StartVideo"));
    }

    stopVideo() {
        this.websocket.send(JSON.stringify("StopVideo"));
    };

    close() {
        clearInterval(this.heartbeatInterval);
        clearTimeout(this.heartbeatTimeout);

        this.websocket.send(JSON.stringify("Close"));
        this.websocket.close();
    }

    goBack() {
        this.websocket.send(JSON.stringify("GoBack"));
    };

    goForward() {
        this.websocket.send(JSON.stringify("GoForward"));
    }

    reload() {
        this.websocket.send(JSON.stringify("Reload"));
    }
}