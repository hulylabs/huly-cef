export enum LoadStatus {
    Loading = 0,
    Loaded = 1,
    Error = 2,
}

export type LoadState = {
    status: LoadStatus;
    canGoBack: boolean;
    canGoForward: boolean;
    errorCode?: number;
    errorMessage?: string;
};

export enum Cursor {
    Pointer = "Pointer",
}

export type Popup = {
    x: number;
    y: number;
    width: number;
    height: number;
    data: Uint8Array;
}

type TabEvent = {
    Title: string;
    Url: string;
    LoadState: LoadState;
    Favicon: string;
    Cursor: Cursor;
    NewTab: string;
    Render: Uint8Array;
    PopupRender: Popup;
}

interface Message<T extends keyof TabEvent> {
    type: T;
    data: TabEvent[T];
}

export class TabEventStream {
    websocket: WebSocket;

    subscribers: Map<keyof TabEvent, Set<(data: any) => void>> = new Map();

    constructor(url: string) {
        this.websocket = new WebSocket(url);
        this.websocket.binaryType = "arraybuffer";
        this.websocket.onmessage = (event) => this.onmessage(event);
    }

    public on<K extends keyof TabEvent>(eventType: K, callback: (data: TabEvent[K]) => void) {
        if (!this.subscribers.has(eventType)) {
            this.subscribers.set(eventType, new Set());
        }
        this.subscribers.get(eventType)!.add(callback);
    }

    public off<K extends keyof TabEvent>(eventType: K, callback: (data: TabEvent[K]) => void) {
        this.subscribers.get(eventType)?.delete(callback);
    }

    private onmessage(event: MessageEvent) {
        if (typeof event.data === "string") {
            let message: Message<keyof TabEvent> = JSON.parse(event.data);
            this.emit(message.type, message.data);
        }

        if (event.data instanceof ArrayBuffer) {
            let data = new Uint8Array(event.data);

            if (data[0] == 0) {
                let message: Message<keyof TabEvent> = {
                    type: "Render",
                    data: data.subarray(1)
                };

                this.emit(message.type, message.data);
            } else {
                // Use DataView here
                let x = data[1] | (data[2] << 8) | (data[3] << 16) | (data[4] << 24);
                let y = data[5] | (data[6] << 8) | (data[7] << 16) | (data[8] << 24);
                let w = data[9] | (data[10] << 8) | (data[11] << 16) | (data[12] << 24);
                let h =
                    data[13] | (data[14] << 8) | (data[15] << 16) | (data[16] << 24);

                let message: Message<keyof TabEvent> = {
                    type: "PopupRender",
                    data: {
                        x: x,
                        y: y,
                        width: w,
                        height: h,
                        data: data.subarray(17)
                    }
                };

                this.emit(message.type, message.data);
            }
        }
    }

    private emit<K extends keyof TabEvent>(type: K, data: TabEvent[K]) {
        let callbacks = this.subscribers.get(type);
        if (callbacks) {
            callbacks.forEach(cb => cb(data));
        }
    }
}