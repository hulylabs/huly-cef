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

export class CEFClient {
    websocket: WebSocket;
    pendingMessages: string[] = [];

    heartbeatInterval: NodeJS.Timeout | number | undefined;

    public onConnectionBroken: (() => void) | undefined;
    public onLoadStateChanged: ((state: LoadState) => void) | undefined;
    public onTitleChanged: ((title: string) => void) | undefined;
    public onUrlChanged: ((url: string) => void) | undefined;
    public onCursorChanged: ((cursor: string) => void) | undefined;
    public onNewTabRequested: ((url: string) => void) | undefined;
    public onRender: ((data: Uint8Array) => void) | undefined;

    constructor(url: string) {
        this.websocket = this.createWebSocket(url);
        this.startHeartbeat();
    }

    goTo(url: string) {
        this.send(JSON.stringify({ GoTo: { url: url } }));
    };

    onMouseMove(x: number, y: number) {
        this.send(JSON.stringify({ MouseMove: { x: x, y: y } }));
    };

    onMouseDown(x: number, y: number, button: number) {
        this.send(JSON.stringify({ MouseClick: { x: x, y: y, down: true, button: button } }));
    };

    onMouseUp(x: number, y: number, button: number) {
        this.send(JSON.stringify({ MouseClick: { x: x, y: y, down: false, button: button } }));
    };

    onMouseWheel(x: number, y: number, dx: number, dy: number) {
        this.send(JSON.stringify({ MouseWheel: { x: x, y: y, dx: dx, dy: dy } }));
    };

    onKeyDown(key: string, code: number, ctrl: boolean, shift: boolean) {
        let char = 0;
        if (key.length === 1) {
            char = key.charCodeAt(0);
        }
        this.send(JSON.stringify({ KeyPress: { character: char, code: code, down: true, ctrl: ctrl, shift: shift } }));
    };

    onKeyUp(key: string, code: number, ctrl: boolean, shift: boolean) {
        let char = 0;
        if (key.length === 1) {
            char = key.charCodeAt(0);
        }
        this.send(JSON.stringify({ KeyPress: { character: char, code: code, down: false, ctrl: ctrl, shift: shift } }));
    };

    resize(width: number, height: number) {
        this.send(JSON.stringify({ Resize: { width: Math.floor(width), height: Math.floor(height) } }));
    };

    startVideo() {
        this.send(JSON.stringify("StartVideo"));
    }

    stopVideo() {
        this.send(JSON.stringify("StopVideo"));
    };

    close() {
        clearInterval(this.heartbeatInterval);
        this.send(JSON.stringify("Close"));
    }

    goBack() {
        this.send(JSON.stringify("GoBack"));
    };

    goForward() {
        this.send(JSON.stringify("GoForward"));
    }

    reload() {
        this.send(JSON.stringify("Reload"));
    }

    private createWebSocket(url: string) {
        let websocket = new WebSocket(url);
        websocket.binaryType = "arraybuffer";
        websocket.onopen = () => this.onopen();
        websocket.onmessage = (event) => this.onmessage(event);

        return websocket;
    }

    private onopen() {
        for (let i = 0; i < this.pendingMessages.length; i++) {
            this.websocket.send(this.pendingMessages[i]);
        }
        this.pendingMessages = [];
    }

    private onmessage(event: MessageEvent) {
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

    private send(message: string) {
        switch (this.websocket.readyState) {
            case WebSocket.CONNECTING:
                this.pendingMessages.push(message);
                break;
            case WebSocket.OPEN:
                this.websocket.send(message);
                break;
        }
    }

    private startHeartbeat() {
        this.heartbeatInterval = setInterval(() => {
            if (this.websocket.readyState === WebSocket.CLOSED) {
                this.websocket = this.createWebSocket(this.websocket.url);
            }
        }, HEARTBEAT_INTERVAL);
    }
}