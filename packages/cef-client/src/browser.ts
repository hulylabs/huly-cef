import { KeyCode, keyCodeToMacOSVirtualKey, keyCodeToWindowsVirtualKey } from './keyboard.js';
import { Tab } from './tab.js';
import { ClickableElement, detectPlatform, OpenTabOptions, Platform } from './types.js';
import { MessageHandler } from './messages.js';

export class Browser {
    private websocket: WebSocket;
    private messageHandler: MessageHandler;
    private platform: Platform = detectPlatform();

    constructor(websocket: WebSocket) {
        this.websocket = websocket;
        this.messageHandler = new MessageHandler(this.websocket);
    }

    async openTab(options?: OpenTabOptions): Promise<Tab> {
        let id = await this.messageHandler.send(-1, 'OpenTab', { options: options });
        return new Tab(this.messageHandler, id);
    }

    async getTabs(): Promise<Tab[]> {
        let ids = await this.messageHandler.send(-1, 'GetTabs');
        return ids.map(id => new Tab(this.messageHandler, id));
    }

    resize(width: number, height: number): Promise<void> {
        return this.messageHandler.send(-1, 'Resize', {
            width: Math.floor(width),
            height: Math.floor(height)
        });
    }

    goTo(tabId: number, url: string): Promise<void> {
        return this.messageHandler.send(tabId, 'Navigate', { url });
    }

    mouseMove(tabId: number, x: number, y: number): Promise<void> {
        return this.messageHandler.send(tabId, 'MouseMove', { x, y });
    }

    mouseClick(tabId: number, x: number, y: number, button: number, down: boolean): Promise<void> {
        return this.messageHandler.send(tabId, 'Click', { x, y, button, down });
    }

    mouseWheel(tabId: number, x: number, y: number, dx: number, dy: number): Promise<void> {
        return this.messageHandler.send(tabId, 'Wheel', { x, y, dx, dy });
    }

    keyPress(
        tabId: number,
        keycode: KeyCode,
        character: number,
        down: boolean,
        ctrl: boolean,
        shift: boolean,
    ): Promise<void> {
        let platformKeyCode = 0;
        switch (this.platform) {
            case Platform.Windows:
            case Platform.Linux:
                platformKeyCode = keyCodeToWindowsVirtualKey(keycode);
                break;
            case Platform.MacOS:
                platformKeyCode = keyCodeToMacOSVirtualKey(keycode);
                break;
        }
        return this.messageHandler.send(tabId, 'Key', {
            character: character,
            windowscode: keyCodeToWindowsVirtualKey(keycode),
            code: platformKeyCode,
            down: down,
            ctrl: ctrl,
            shift: shift,
        });
    }

    char(tabId: number, character: number): Promise<void> {
        return this.messageHandler.send(tabId, 'Char', character);
    }

    stopVideo(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'StopVideo');
    }

    startVideo(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'StartVideo');
    }

    reload(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'Reload');
    }

    goBack(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'GoBack');
    }

    goForward(tabId: number): Promise<void> {
        return this.messageHandler.send(tabId, 'GoForward');
    }

    setFocus(tabId: number, focus: boolean): Promise<void> {
        return this.messageHandler.send(tabId, 'SetFocus', focus);
    }

    getDOM(tabId: number): Promise<string> {
        return this.messageHandler.send(tabId, 'GetDOM');
    }

    getClickableElements(tabId: number): Promise<ClickableElement[]> {
        return this.messageHandler.send(tabId, 'GetClickableElements');
    }

    clickElement(tabId: number, id: number): Promise<void> {
        return this.messageHandler.send(tabId, 'ClickElement', id);
    }
}
