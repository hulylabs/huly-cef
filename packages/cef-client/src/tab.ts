import { KeyCode, keyCodeToMacOSVirtualKey, keyCodeToWindowsVirtualKey } from "./keyboard.js";
import { MessageHandler } from "./messages.js";
import { ClickableElement, detectPlatform, MouseButton, Platform, ScreenshotOptions } from "./types.js";

export class Tab {
    id: number;
    private messageHandler: MessageHandler;

    private platform: Platform = detectPlatform();

    constructor(messageHandler: MessageHandler, id: number) {
        this.messageHandler = messageHandler;
        this.id = id;
    }

    async title(): Promise<string> {
        return this.messageHandler.send(this.id, 'GetTitle');
    }

    async url(): Promise<string> {
        return this.messageHandler.send(this.id, 'GetUrl');
    }

    async screenshot(options?: ScreenshotOptions): Promise<string> {
        return this.messageHandler.send(this.id, 'Screenshot', { options: options });
    }

    navigate(url: string): void {
        return this.messageHandler.sendNoResponse(this.id, 'Navigate', { url: url });
    }

    back(): void {
        return this.messageHandler.sendNoResponse(this.id, 'GoBack');
    }

    forward(): void {
        return this.messageHandler.sendNoResponse(this.id, 'GoForward');
    }

    reload(): void {
        return this.messageHandler.sendNoResponse(this.id, 'Reload');
    }

    close(): void {
        return this.messageHandler.sendNoResponse(this.id, 'CloseTab');
    }

    mouseMove(x: number, y: number): void {
        return this.messageHandler.sendNoResponse(this.id, 'MouseMove', { x, y });
    }

    click(x: number, y: number, button: MouseButton, down: boolean): void {
        return this.messageHandler.sendNoResponse(this.id, 'Click', { x, y, button, down });
    }

    scroll(x: number, y: number, dx: number, dy: number): void {
        return this.messageHandler.sendNoResponse(this.id, 'Wheel', { x, y, dx, dy });
    }

    key(keycode: KeyCode,
        character: number,
        down: boolean,
        ctrl: boolean,
        shift: boolean,
    ): void {
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
        return this.messageHandler.sendNoResponse(this.id, 'Key', {
            character: character,
            windowscode: keyCodeToWindowsVirtualKey(keycode),
            code: platformKeyCode,
            down: down,
            ctrl: ctrl,
            shift: shift,
        });
    }

    char(unicode: number): void {
        return this.messageHandler.sendNoResponse(this.id, 'Char', { unicode });
    }

    clickableElements(): Promise<ClickableElement[]> {
        return this.messageHandler.send(this.id, 'GetClickableElements');
    }

    clickElement(id: number): Promise<void> {
        return this.messageHandler.send(this.id, 'ClickElement', { id: id });
    }
}