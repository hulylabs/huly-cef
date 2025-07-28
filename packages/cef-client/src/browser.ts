import { Tab } from './tab.js';
import { DEFAULT_HEIGHT, DEFAULT_URL, DEFAULT_WIDTH, OpenTabOptions } from './types.js';
import { MessageHandler } from './messages.js';

export class Browser {
    private serverUrl: URL;
    private websocket: WebSocket;
    private messageHandler: MessageHandler;

    constructor(serverUrl: URL, websocket: WebSocket) {
        this.serverUrl = serverUrl;
        this.websocket = websocket;
        this.messageHandler = new MessageHandler(this.websocket);
    }

    async openTab(options?: Partial<OpenTabOptions>): Promise<Tab> {
        const params = {
            url: options?.url || DEFAULT_URL,
            wait_until_loaded: options?.wait_until_loaded ?? false,
            width: options?.width || DEFAULT_WIDTH,
            height: options?.height || DEFAULT_HEIGHT
        };

        const result = await this.messageHandler.send('openTab', params);

        // Extract the tab ID from the result
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

    async resize(width: number, height: number): Promise<void> {
        await this.messageHandler.send('resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }

    async closeTab(tabId: number): Promise<void> {
        await this.messageHandler.send('closeTab', { tab: tabId });
    }
}