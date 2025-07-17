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
    Frame: Uint8Array;
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
            let message: Message<keyof TabEvent> = {
                type: "Frame",
                data: new Uint8Array(event.data)
            };

            this.emit(message.type, message.data);
        }
    }

    private emit<K extends keyof TabEvent>(type: K, data: TabEvent[K]) {
        let callbacks = this.subscribers.get(type);
        if (callbacks) {
            callbacks.forEach(cb => cb(data));
        }
    }
}