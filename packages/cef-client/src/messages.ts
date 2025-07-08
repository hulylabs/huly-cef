import { REQUEST_TIMEOUT, ClickableElement, ScreenshotOptions, OpenTabOptions } from './types.js';

type CefRequestType = {
    OpenTab: { options?: OpenTabOptions };
    CloseTab: never;
    GetTabs: never;
    Resize: { width: number; height: number };
    Screenshot: {
        options?: ScreenshotOptions
    };
    Navigate: { url: string };
    MouseMove: { x: number; y: number };
    Click: { x: number; y: number; button: number; down: boolean };
    Wheel: { x: number; y: number; dx: number; dy: number };
    Key: { character: number; windowscode: number; code: number; down: boolean; ctrl: boolean; shift: boolean };
    Char: { unicode: number };
    StopVideo: never;
    StartVideo: never;
    Reload: never;
    GoBack: never;
    GoForward: never;
    GetDOM: never;
    GetClickableElements: never;
    SetFocus: boolean;
    ClickElement: { id: number };
    GetTitle: never;
    GetUrl: never;
}

type CefResponseType = {
    Tab: number;
    Tabs: number[];
    Screenshot: string;
    GetDOM: string;
    GetClickableElements: ClickableElement[];
    ClickableElements: ClickableElement[];
    Title: string;
    Url: string;
}

type RequestToResponseMapping = {
    OpenTab: 'Tab';
    CloseTab: never;
    GetTabs: 'Tabs';
    Resize: never;
    Screenshot: 'Screenshot';
    Navigate: never;
    MouseMove: never;
    Click: never;
    Wheel: never;
    Key: never;
    Char: never;
    StopVideo: never;
    StartVideo: never;
    Reload: never;
    GoBack: never;
    GoForward: never;
    GetDOM: 'GetDOM';
    GetClickableElements: 'GetClickableElements';
    SetFocus: never;
    ClickElement: never;
    GetTitle: 'Title';
    GetUrl: 'Url';
}

interface RequestMessage<T extends keyof CefRequestType> {
    id: string;
    tab_id: number;
    body: {
        type: T;
        data: CefRequestType[T];
    };
}

interface ResponseMessage<T extends keyof CefResponseType> {
    id: string;
    tab_id: number;
    body: {
        type: T;
        data: CefResponseType[T];
    };
}


export class MessageHandler {
    private pendingPromises: Map<string, { resolve: (value: any) => void, reject: (error: any) => void }> = new Map();

    constructor(private websocket: WebSocket) {
        this.websocket.onmessage = (event) => {
            let resp: ResponseMessage<keyof CefResponseType> = JSON.parse(event.data);
            this.resolve(resp);
        }
    }

    send<T extends keyof CefRequestType>(
        tabId: number,
        type: T,
        data?: CefRequestType[T]
    ): Promise<RequestToResponseMapping[T] extends keyof CefResponseType ? CefResponseType[RequestToResponseMapping[T]] : void> {
        const id = crypto.randomUUID();
        const message: RequestMessage<T> = {
            id,
            tab_id: tabId,
            body: {
                type,
                data: data !== undefined ? data : (undefined as never)
            }
        };

        return new Promise((resolve, reject) => {
            this.pendingPromises.set(id, { resolve, reject });
            this.websocket.send(JSON.stringify(message));

            setTimeout(() => {
                if (this.pendingPromises.has(id)) {
                    this.pendingPromises.delete(id);
                    reject(new Error('Timeout waiting for response'));
                }
            }, REQUEST_TIMEOUT);
        });
    }

    sendNoResponse<T extends keyof CefRequestType>(
        tabId: number,
        type: T,
        data?: CefRequestType[T]
    ): void {
        const id = crypto.randomUUID();
        const message: RequestMessage<T> = {
            id,
            tab_id: tabId,
            body: {
                type,
                data: data !== undefined ? data : (undefined as never)
            }
        };

        this.websocket.send(JSON.stringify(message));
    }

    private resolve(response: ResponseMessage<keyof CefResponseType>): void {
        const pendingPromise = this.pendingPromises.get(response.id);
        if (!pendingPromise) {
            console.warn(`No pending promise for response ID: ${response.id}`);
            return;
        }

        try {
            pendingPromise.resolve(response.body.data);
        } catch (err) {
            pendingPromise.reject(err);
        } finally {
            this.pendingPromises.delete(response.id);
        }
    }
}
