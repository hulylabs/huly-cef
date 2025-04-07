export enum LoadState {
    Loading,
    Loaded,
    Error,
}

export class CEFClient {
    websocket: WebSocket;

    public onLoadStateChanged: ((state: LoadState) => void) | undefined;
    public onTitleChanged: ((title: string) => void) | undefined;
    public onUrlChanged: ((url: string) => void) | undefined;
    public onCursorChanged: ((cursor: string) => void) | undefined;
    public onNewTabRequested: ((url: string) => void) | undefined;
    public onRender: ((data: Uint8Array) => void) | undefined;

    constructor(websocket: WebSocket) {
        this.websocket = websocket;

        this.websocket.onmessage = (event) => {
            if (event.data instanceof ArrayBuffer) {
                let imageData = new Uint8Array(event.data);
                this.onRender?.(imageData);
            }

            if (typeof event.data === "string") {
                let parsed = JSON.parse(event.data);
                console.log("parsed", parsed);

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