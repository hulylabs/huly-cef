import { afterAll, afterEach, beforeAll, describe, expect, test } from 'vitest';
import sharp from 'sharp';

import { Browser, connect, KeyCode, MouseButton, Tab } from '../src/index';

import { pollTimeout, getPageUrl, launchCef, CefProcess } from './common';

describe('tabs', () => {
    let browser: Browser;
    let cef_process: CefProcess;
    let port: number;

    beforeAll(async () => {
        port = 8079;
        cef_process = await launchCef(port, "cache/automation", 5000);
        browser = await connect("ws://localhost:" + port + "/browser");
    });

    afterAll(() => {
        cef_process.cef.kill();
    });

    afterEach(async () => {
        let tabs = await browser.tabs();
        for (let tab of tabs) {
            tab.close();
        }
        tabs = [];
    });

    test('screenshot', async () => {
        browser.resize(1920, 1080);

        const tab = await browser.openTab({ url: getPageUrl("title.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Title");

        let [width, height] = [800, 600];
        const screenshot = await tab.screenshot({ size: { width, height } });
        expect(screenshot).toBeDefined();

        const img = Buffer.from(screenshot, 'base64');
        const metadata = await sharp(img).metadata();
        expect(metadata.width).toBe(width);
        expect(metadata.height).toBe(height);
        expect(metadata.format).toBe('png');
    });

    test('clickable_elements', async () => {
        const tab = await browser.openTab({ url: getPageUrl("links.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Links");

        const result = await tab.clickableElements();
        expect(result).toEqual([
            { id: 0, tag: "a", text: "External Link (Title)" },
            { id: 1, tag: "a", text: "Internal Link (Jump to Section)" },
        ]);

        // TODO: click element
        // TODO: click non-existing element
    });

    test('subframes', async () => {
        const tab = await browser.openTab({ url: getPageUrl("frames.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Frames");
    });
});

