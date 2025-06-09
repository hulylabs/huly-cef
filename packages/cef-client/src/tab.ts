const HEARTBEAT_INTERVAL = 5000;

export enum LoadStateKind {
    Loading,
    Loaded,
    Error,
}

export type LoadState = {
    LoadStateKind: LoadStateKind;
    canGoBack: boolean;
    canGoForward: boolean;
    error_code?: number;
    error_message?: string;
};

// export enum CursorType {
//     Default = "default",
//     Pointer = "pointer",
//     Text = "text",
//     Wait = "wait",
//     Crosshair = "crosshair",
//     Move = "move",
//     NotAllowed = "not-allowed",
// }


export class TabEventStream {
    websocket: WebSocket;

    public onLoadStateChanged: ((state: LoadState) => void) | undefined;
    public onTitleChanged: ((title: string) => void) | undefined;
    public onUrlChanged: ((url: string) => void) | undefined;
    public onFaviconUrlChanged: ((url: string) => void) | undefined;
    public onCursorChanged: ((cursor: string) => void) | undefined;
    public onNewTabRequested: ((url: string) => void) | undefined;
    public onRender: ((data: Uint8Array) => void) | undefined;
    public onPopupRender:
        | ((x: number, y: number, w: number, h: number, data: Uint8Array) => void)
        | undefined;

    constructor(url: string) {
        this.websocket = this.createWebSocket(url);
    }

    private createWebSocket(url: string, reconnect: boolean = false): WebSocket {
        let websocket = new WebSocket(url);
        websocket.binaryType = "arraybuffer";

        websocket.onmessage = (event) => this.onmessage(event);

        websocket.onclose = () => {
            console.log("WebSocket connection closed.");
        }

        return websocket;
    }

    private onmessage(event: MessageEvent) {
        if (event.data instanceof ArrayBuffer) {
            let data = new Uint8Array(event.data);

            if (data[0] == 0) {
                this.onRender?.(data.subarray(1));
            } else {
                let x = data[1] | (data[2] << 8) | (data[3] << 16) | (data[4] << 24);
                let y = data[5] | (data[6] << 8) | (data[7] << 16) | (data[8] << 24);
                let w = data[9] | (data[10] << 8) | (data[11] << 16) | (data[12] << 24);
                let h =
                    data[13] | (data[14] << 8) | (data[15] << 16) | (data[16] << 24);

                this.onPopupRender?.(x, y, w, h, data.subarray(17));
            }
            return;
        }

        if (typeof event.data === "string") {
            let parsed = JSON.parse(event.data);

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

                if (parsed.FaviconUrlChanged) {
                    this.onFaviconUrlChanged?.(parsed.FaviconUrlChanged);
                }

                if (parsed.LoadStateChanged) {
                    let state = parsed.LoadStateChanged;
                    let loadState: LoadState = {
                        LoadStateKind: LoadStateKind.Loading,
                        canGoBack: state.can_go_back,
                        canGoForward: state.can_go_forward,
                    };

                    switch (state.state) {
                        case "Loading":
                            loadState.LoadStateKind = LoadStateKind.Loading;
                            break;
                        case "Loaded":
                            loadState.LoadStateKind = LoadStateKind.Loaded;
                            break;
                        case "LoadError":
                            loadState.LoadStateKind = LoadStateKind.Error;
                            break;
                    }

                    if (state.error_code != 0) {
                        loadState.error_code = state.error_code;
                    }

                    if (state.error_message != "") {
                        loadState.error_message = state.error_message;
                    }

                    this.onLoadStateChanged?.(loadState);
                }
            }
        }
    }
}