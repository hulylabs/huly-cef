import { Cursor, LoadState } from "./types.js";

interface Frame {
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
    UrlHovered: string;
    NewTab: string;
    Frame: Frame;
    Download: string;
    DownloadProgress: { received: number; total: number };
}

interface Message<T extends keyof TabEvent> {
    type: T;
    data: TabEvent[T];
}

export class TabEventStream {
    websocket: WebSocket;

    subscribers: Map<keyof TabEvent, (data: any) => void> = new Map();

    constructor(url: string) {
        this.websocket = new WebSocket(url);
        this.websocket.binaryType = "arraybuffer";
        this.websocket.onmessage = (event) => this.onmessage(event);
    }

    closeConnection() {
        this.websocket.close();
    }

    on<K extends keyof TabEvent>(eventType: K, callback: (data: TabEvent[K]) => void) {
        if (!this.subscribers.has(eventType)) {
            this.subscribers.set(eventType, callback);
        }
    }

    private onmessage(event: MessageEvent) {
        if (typeof event.data === "string") {
            console.log("Received message:", event.data);
            let message: Message<keyof TabEvent> = JSON.parse(event.data);
            this.emit(message.type, message.data);
        }

        if (event.data instanceof ArrayBuffer) {
            let view = new DataView(event.data);
            let width = view.getUint32(0, true);
            let height = view.getUint32(4, true);
            let data = new Uint8Array(event.data, 8);

            let message: Message<keyof TabEvent> = {
                type: "Frame",
                data: {
                    width,
                    height,
                    data
                }
            };

            this.emit(message.type, message.data);
        }
    }

    private emit<K extends keyof TabEvent>(type: K, data: TabEvent[K]) {
        let callback = this.subscribers.get(type);
        if (callback) {
            callback(data);
        }
    }
}