import { Tab } from './tab.js';
import { OpenTabOptions } from './types.js';
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

    async openTab(options?: OpenTabOptions): Promise<Tab> {
        let id = await this.messageHandler.send(-1, 'OpenTab', { options: options });
        return new Tab(id, this.serverUrl, this.messageHandler);
    }

    async tabs(): Promise<Tab[]> {
        let ids = await this.messageHandler.send(-1, 'GetTabs');
        return ids.map(id => new Tab(id, this.serverUrl, this.messageHandler));
    }

    resize(width: number, height: number) {
        this.messageHandler.sendNoResponse(-1, 'Resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }

}
