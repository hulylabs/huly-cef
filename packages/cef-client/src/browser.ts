import { Tab } from './tab.js';
import { OpenTabOptions } from './types.js';
import { MessageHandler } from './messages.js';
import { getConfig } from './config.js';

export class Browser {
    private serverUrl: URL;
    private websocket: WebSocket;
    private messageHandler: MessageHandler;

    constructor(serverUrl: URL, websocket: WebSocket) {
        this.serverUrl = serverUrl;
        this.websocket = websocket;
        this.messageHandler = new MessageHandler(this.websocket);
    }

    closeConnection() {
        this.websocket.close();
    }

    close(): Promise<void> {
        return this.messageHandler.send('close', {});
    }

    async openTab(options?: Partial<OpenTabOptions>): Promise<Tab> {
        const params = {
            url: (options && options.url !== "") ? options.url : getConfig().defaultUrl,
            wait_until_loaded: options?.wait_until_loaded ?? false,
            dpr: (typeof window !== 'undefined' ? window.devicePixelRatio : 1.0) || 1.0
        };

        const result = await this.messageHandler.send('openTab', params);

        if (result && typeof result.id === 'number') {
            return new Tab(result.id, this.serverUrl, this.messageHandler);
        }

        throw new Error('Invalid response from openTab');
    }

    async tabs(): Promise<Tab[]> {
        const result = await this.messageHandler.send('getTabs', {});

        if (result && Array.isArray(result.tabs)) {
            return result.tabs.map((id: number) => new Tab(id, this.serverUrl, this.messageHandler));
        }

        throw new Error('Invalid response from getTabs');
    }

    async size(): Promise<{ width: number; height: number }> {
        const result = await this.messageHandler.send('getSize', {});
        if (result && typeof result.width === 'number' && typeof result.height === 'number') {
            return {
                width: result.width,
                height: result.height
            };
        }

        throw new Error('Invalid response from getSize');
    }

    async resize(width: number, height: number): Promise<void> {
        await this.messageHandler.send('resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }
}