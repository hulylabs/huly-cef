import { MessageHandler } from "./messages.js";

const REQUEST_TIMEOUT = 5000;


export class Tab {
    private id: number;
    private messageHandler: MessageHandler;

    pendingPromises: Map<string, { resolve: (value: any) => void, reject: () => void }> = new Map();

    constructor(messageHandler: MessageHandler, id: number) {
        this.messageHandler = messageHandler;
        this.id = id;
    }

    async title(): Promise<string> {
        return this.messageHandler.send(this.id, 'GetTitle');
    }
}