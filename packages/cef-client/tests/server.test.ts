import { describe, expect, test } from 'vitest';

import { connect } from '../src/index';

import { getPageUrl, launchCef } from './common';

describe('server', () => {
    const port = 8080;
    test('shutdown CEF', async () => {
        const cef = await launchCef(port, "cache/server", 5000);

        const browser = await connect("ws://localhost:" + port + "/browser");
        await browser.close();

        const exitCode = await cef.finished;
        expect(exitCode).toBe(0);
    });

    test('restore session', async () => {
        let cef = await launchCef(port, "cache/server", 5000);
        let browser = await connect("ws://localhost:" + port + "/browser");
        await browser.openTab({ url: getPageUrl("title.html"), wait_until_loaded: true });
        await browser.openTab({ url: getPageUrl("resize.html"), wait_until_loaded: true });
        await browser.openTab({ url: getPageUrl("mouse.html"), wait_until_loaded: true });
        await browser.close();
        await cef.finished;

        cef = await launchCef(port, "cache/server", 5000);
        browser = await connect("ws://localhost:" + port + "/browser");
        const tabs = await browser.restore();
        let urls = await Promise.all(tabs.map(tab => tab.url()));

        expect(tabs.length).toBe(3);
        expect(urls.sort()).toEqual([getPageUrl("mouse.html"), getPageUrl("resize.html"), getPageUrl("title.html")].sort());

        await browser.close();
        await cef.finished;
    });
});

