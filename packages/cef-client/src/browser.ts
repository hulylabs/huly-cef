const REQUEST_TIMEOUT = 5000;


export class BrowserClient {
    private websocket: WebSocket;
    private pendingMessages: string[] = [];

    // TODO: use ids instead of hardcoded "Tab"
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
            this.send(JSON.stringify({
                body: {
                    OpenTab: url
                },
                tab_id: -1
            }));
            setTimeout(() => {
                if (this.pendingTabPromises.has("Tab")) {
                    this.pendingTabPromises.delete("Tab");
                    reject(new Error("Timeout opening tab"));
                }
            }, REQUEST_TIMEOUT);
        });
    }

    restoreSession(): void {
        this.send(JSON.stringify("RestoreSession"));
    }

    closeTab(id: number): void {
        this.send(JSON.stringify({ CloseTab: id }));
    }

    close(): void {
        this.send(JSON.stringify({ body: "Close", tab_id: 0 }));
    }

    resize(width: number, height: number): void {
        this.send(JSON.stringify({
            tab_id: -1,
            body: {
                Resize: {
                    width,
                    height
                }
            }
        }));
    }

    goTo(tabId: number, url: string): void {
        this.send(JSON.stringify({
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
            tab_id: tabId,
            body: {
                MouseMove: {
                    x,
                    y
                }
            }
        }));
    }

    mouseClick(tabId: number, x: number, y: number, button: string, down: boolean): void {
        this.send(JSON.stringify({
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

    keyPress(tabId: number, character: number, code: number, windowscode: number, down: boolean, ctrl: boolean, shift: boolean): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: {
                KeyPress: {
                    character,
                    code,
                    windowscode,
                    down,
                    ctrl,
                    shift
                }
            }
        }));
    }

    stopVideo(tabId: number): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: "StopVideo"
        }));
    }

    startVideo(tabId: number): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: "StartVideo"
        }));
    }

    reload(tabId: number): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: "Reload"
        }));
    }

    goBack(tabId: number): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: "GoBack"
        }));
    }

    goForward(tabId: number): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: "GoForward"
        }));
    }

    setFocus(tabId: number, focus: boolean): void {
        this.send(JSON.stringify({
            tab_id: tabId,
            body: {
                SetFocus: focus
            }
        }));
    }

    private send(message: string): void {
        if (this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(message);
        } else {
            this.pendingMessages.push(message);
        }
    }
}