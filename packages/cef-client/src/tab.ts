import { TabEventStream } from "./event_stream.js";
import { KeyCode, keyCodeToMacOSVirtualKey, keyCodeToWindowsVirtualKey } from "./keyboard.js";
import { MessageHandler } from "./messages.js";
import { ClickableElement, detectPlatform, MouseButton, Platform, ScreenshotOptions } from "./types.js";

export class Tab {
    id: number;

    private serverUrl: URL;
    private messageHandler: MessageHandler;
    private platform: Platform = detectPlatform();

    constructor(id: number, serverUrl: URL, messageHandler: MessageHandler) {
        this.id = id;
        this.serverUrl = serverUrl;
        this.messageHandler = messageHandler;
    }

    async title(): Promise<string> {
        const result = await this.messageHandler.send('getTitle', { tab: this.id });
        return result.title;
    }

    async url(): Promise<string> {
        const result = await this.messageHandler.send('getUrl', { tab: this.id });
        return result.url;
    }

    async screenshot(options?: ScreenshotOptions): Promise<string> {
        const { width, height } = options?.size || { width: 800, height: 600 };
        const result = await this.messageHandler.send('screenshot', {
            tab: this.id,
            width,
            height
        });
        return result.screenshot;
    }

    async navigate(url: string, waitUntilLoaded: boolean = false): Promise<void> {
        await this.messageHandler.send('navigate', {
            tab: this.id,
            url,
            wait_until_loaded: waitUntilLoaded
        });
    }

    async back(waitUntilLoaded: boolean = false): Promise<void> {
        await this.messageHandler.send('goBack', { tab: this.id, wait_until_loaded: waitUntilLoaded });
    }

    async forward(waitUntilLoaded: boolean = false): Promise<void> {
        await this.messageHandler.send('goForward', { tab: this.id, wait_until_loaded: waitUntilLoaded });
    }

    async reload(waitUntilLoaded: boolean = false): Promise<void> {
        await this.messageHandler.send('reload', { tab: this.id, wait_until_loaded: waitUntilLoaded });
    }

    async close(): Promise<void> {
        await this.messageHandler.send('closeTab', { tab: this.id });
    }

    async mouseMove(x: number, y: number): Promise<void> {
        await this.messageHandler.send('mouseMove', {
            tab: this.id,
            x: Math.floor(x),
            y: Math.floor(y)
        });
    }

    async click(x: number, y: number, button: MouseButton = MouseButton.Left, down: boolean = true): Promise<void> {
        await this.messageHandler.send('click', {
            tab: this.id,
            x: Math.floor(x),
            y: Math.floor(y),
            button,
            down
        });
    }

    async scroll(x: number, y: number, dx: number, dy: number): Promise<void> {
        await this.messageHandler.send('wheel', {
            tab: this.id,
            x: Math.floor(x),
            y: Math.floor(y),
            dx: Math.floor(dx),
            dy: Math.floor(dy)
        });
    }

    async key(
        keycode: KeyCode,
        character: number,
        down: boolean,
        ctrl: boolean = false,
        shift: boolean = false,
        alt: boolean = false,
        meta: boolean = false
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

        await this.messageHandler.send('key', {
            tab: this.id,
            character: character,
            windowscode: keyCodeToWindowsVirtualKey(keycode),
            code: platformKeyCode,
            down: down,
            ctrl: ctrl,
            shift: shift,
            alt: alt,
            meta: meta,
        });
    }

    async char(unicode: number): Promise<void> {
        await this.messageHandler.send('char', {
            tab: this.id,
            unicode
        });
    }

    async clickableElements(): Promise<ClickableElement[]> {
        const result = await this.messageHandler.send('getClickableElements', { tab: this.id });
        return result.elements;
    }

    async clickElement(elementId: number): Promise<void> {
        await this.messageHandler.send('clickElement', {
            tab: this.id,
            element_id: elementId
        });
    }

    async stopVideo(): Promise<void> {
        await this.messageHandler.send('stopVideo', { tab: this.id });
    }

    async startVideo(): Promise<void> {
        await this.messageHandler.send('startVideo', { tab: this.id });
    }

    async focus(focus: boolean): Promise<void> {
        await this.messageHandler.send('setFocus', {
            tab: this.id,
            focus
        });
    }

    async dom(): Promise<string> {
        const result = await this.messageHandler.send('getDOM', { tab: this.id });
        return result.dom;
    }

    async undo(): Promise<void> {
        await this.messageHandler.send('undo', { tab: this.id });
    }

    async redo(): Promise<void> {
        await this.messageHandler.send('redo', { tab: this.id });
    }

    async selectAll(): Promise<void> {
        await this.messageHandler.send('selectAll', { tab: this.id });
    }

    async copy(): Promise<void> {
        await this.messageHandler.send('copy', { tab: this.id });
    }

    async paste(): Promise<void> {
        await this.messageHandler.send('paste', { tab: this.id });
    }

    async cut(): Promise<void> {
        await this.messageHandler.send('cut', { tab: this.id });
    }

    async delete(): Promise<void> {
        await this.messageHandler.send('delete', { tab: this.id });
    }

    async continueFileDialog(filepaths: string[]): Promise<void> {
        await this.messageHandler.send('continueFileDialog', { tab: this.id, filepaths });
    }

    async cancelFileDialog(): Promise<void> {
        await this.messageHandler.send('cancelFileDialog', { tab: this.id });
    }

    async cancelDownloading(downloadId: number): Promise<void> {
        await this.messageHandler.send('cancelDownloading', { tab: this.id, download_id: downloadId });
    }

    events(): TabEventStream {
        let address = this.serverUrl.origin + "/tab/" + this.id;
        return new TabEventStream(address);
    }
}