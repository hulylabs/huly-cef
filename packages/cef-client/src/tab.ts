import { MessageHandler } from "./messages.js";
import { ScreenshotOptions } from "./types.js";

const REQUEST_TIMEOUT = 5000;


export class Tab {
    id: number;
    private messageHandler: MessageHandler;

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

    close(): void {
        return this.messageHandler.sendNoResponse(this.id, 'CloseTab');
    }
}