import { KeyCode, keyCodeToMacOSVirtualKey, keyCodeToWindowsVirtualKey } from './keyboard.js';
import { Tab } from './tab.js';
import { ClickableElement, detectPlatform, OpenTabOptions, Platform } from './types.js';
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

    resize(width: number, height: number): Promise<void> {
        return this.messageHandler.send(-1, 'Resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }

    stopVideo(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'StopVideo');
    }

    startVideo(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'StartVideo');
    }

    setFocus(tabId: number, focus: boolean): Promise<void> {
        return this.messageHandler.send(tabId, 'SetFocus', focus);
    }

    getDOM(tabId: number): Promise<string> {
        return this.messageHandler.send(tabId, 'GetDOM');
    }
}
