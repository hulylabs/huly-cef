export class BrowserClient {
    private websocket: WebSocket;
    private pendingMessages: string[] = [];
    private pendingTabPromises: Map<string, { resolve: (tabId: number) => void, reject: (error: Error) => void }> = new Map();

    public onSessionRestored: ((tabs: string[]) => void) | undefined;

    constructor(url: string) {
        this.websocket = new WebSocket(url);

        this.websocket.onopen = () => {
            console.log("WebSocket connection established.");
            for (const message of this.pendingMessages) {
                this.websocket.send(message);
            };
        }

        this.websocket.onmessage = (event) => {
            let msg = JSON.parse(event.data);
            if (msg.Session) {
                this.onSessionRestored?.(msg.Session);
            }

            if (msg.Tab) {
                const pendingPromise = this.pendingTabPromises.get("Tab");
                if (pendingPromise) {
                    pendingPromise.resolve(msg.Tab);
                    this.pendingTabPromises.delete("Tab");
                }
            }
        }
    }

    openTab(url: string): Promise<number> {
        return new Promise<number>((resolve, reject) => {
            this.pendingTabPromises.set("Tab", { resolve, reject });
            this.send(JSON.stringify({ OpenTab: url }));
            setTimeout(() => {
                if (this.pendingTabPromises.has("Tab")) {
                    this.pendingTabPromises.delete("Tab");
                    reject(new Error("Timeout opening tab"));
                }
            }, 30000);
        });
    }

    restoreSession(): void {
        this.send(JSON.stringify("RestoreSession"));
    }

    closeTab(id: number): void {
        this.send(JSON.stringify({ CloseTab: id }));
    }

    private send(message: string): void {
        if (this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(message);
        } else {
            this.pendingMessages.push(message);
        }
    }
}