import { Tab } from './tab.js';
import { OpenTabOptions } from './types.js';
import { MessageHandler } from './messages.js';

export class Browser {
    private websocket: WebSocket;
    private messageHandler: MessageHandler;

    constructor(websocket: WebSocket) {
        this.websocket = websocket;
        this.messageHandler = new MessageHandler(this.websocket);
    }

    async openTab(options?: OpenTabOptions): Promise<Tab> {
        let id = await this.messageHandler.send(-1, 'OpenTab', { options: options });
        return new Tab(this.messageHandler, id);
    }

    async tabs(): Promise<Tab[]> {
        let ids = await this.messageHandler.send(-1, 'GetTabs');
        return ids.map(id => new Tab(this.messageHandler, id));
    }

    resize(width: number, height: number) {
        this.messageHandler.sendNoResponse(-1, 'Resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }

}
