import { get } from "http";
import { getConfig } from "./config.js";

interface Request {
    id: string;
    method: string;
    params: any;
}

interface Response {
    id: string;
    result?: any;
    error?: {
        message: string;
        data?: any;
    };
}

export class MessageHandler {
    private pendingPromises: Map<string, { resolve: (value: any) => void, reject: (error: any) => void }> = new Map();

    constructor(private websocket: WebSocket) {
        this.websocket.onmessage = (event) => {
            this.resolve(JSON.parse(event.data));
        }
    }

    send(method: string, params: any): Promise<any> {
        const id = crypto.randomUUID();
        const message: Request = {
            id,
            method,
            params
        };

        if (getConfig().logging) {
            console.log(`Sending message: ${method} with params:`, params);
        }

        return new Promise((resolve, reject) => {
            this.pendingPromises.set(id, { resolve, reject });
            this.websocket.send(JSON.stringify(message));

            setTimeout(() => {
                if (this.pendingPromises.has(id)) {
                    this.pendingPromises.delete(id);
                    reject(new Error(`Timeout waiting for response to ${method}`));
                }
            }, getConfig().requestTimeout);
        });
    }

    private resolve(response: Response): void {
        if (getConfig().logging) {
            console.log(`Received response for ID ${response.id}:`, response);
        }

        const pendingPromise = this.pendingPromises.get(response.id);
        if (!pendingPromise) {
            console.warn(`No pending promise for response ID: ${response.id}`);
            return;
        }

        this.pendingPromises.delete(response.id);

        if (response.error) {
            const error = new Error(response.error.message);
            (error as any).data = response.error.data;
            pendingPromise.reject(error);
        } else {
            pendingPromise.resolve(response.result);
        }
    }
}