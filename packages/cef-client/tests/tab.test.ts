import { afterAll, afterEach, beforeAll, describe, expect, test } from 'vitest';
import sharp from 'sharp';

import { Browser, connect, KeyCode, MouseButton, Tab } from '../src/index';

import { pollTimeout, getPageUrl, launchCef, CefProcess } from './common';

describe('tabs', () => {
    let browser: Browser;
    let cef_process: CefProcess;
    let port: number;

    beforeAll(async () => {
        port = 8081;
        cef_process = await launchCef(port, "cache/tabs", 5000);
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

    test.skip('open a new tab', async () => {
        const url = getPageUrl("title.html");
        const tab = await browser.openTab({ url: url, wait_until_loaded: true });
        expect(await tab.title()).toBe("Title");
        expect(await tab.url()).toBe(url);
    });

    test.skip('resize', async () => {
        let [width, height] = [800, 600];
        browser.resize(width, height);

        const url = getPageUrl("resize.html");
        const tab = await browser.openTab({ url });
        await expect.poll(() => tab.title(), pollTimeout).toBe("800x600");

        [width, height] = [1024, 768];
        browser.resize(width, height);
        await expect.poll(() => tab.title(), pollTimeout).toBe("1024x768");
    });

    test.skip('go to a url', async () => {
        const tab = await browser.openTab({ url: "", wait_until_loaded: true });
        expect(await tab.title()).toBe("New Tab");
        expect(await tab.url()).toBe("huly://newtab");

        await tab.navigate("https://www.google.com/", true);
        expect(await tab.title()).toBe("Google");
    });

    test.skip('multiple tabs', async () => {
        let client = await connect("ws://localhost:" + port + "/browser");
        await browser.openTab({ url: getPageUrl("title.html"), wait_until_loaded: true });
        await client.openTab({ url: getPageUrl("keyboard.html"), wait_until_loaded: true });
        await client.openTab({ url: getPageUrl("links.html"), wait_until_loaded: true });

        const tabs = await browser.tabs();
        const titles = await Promise.all(tabs.map(tab => tab.title()));
        expect(titles.sort()).toEqual(["Title", "Keyboard", "Links"].sort());
    });

    test('tab navigation', async () => {
        const tab = await browser.openTab({ url: getPageUrl("links.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Links");

        let elements = await tab.clickableElements();
        expect(elements[0].id).toBe(0);
        expect(elements[0].tag).toBe("a");
        expect(elements[0].text).toBe("External Link (Title)");

        // TODO: check load state

        tab.clickElement(elements[0].id);
        await expect.poll(() => tab.title(), pollTimeout).toBe("Title");

        tab.back(true);
        expect(await tab.title()).toBe("Links");

        tab.forward(true);
        expect(await tab.title()).toBe("Title");
    });

    test.skip('tab reloading', async () => {
        const tab = await browser.openTab({ url: getPageUrl("reload.html"), wait_until_loaded: true });
        tab.reload(true);
        expect(await tab.title()).toBe("Reloads: 2");
    });

    test.skip('mouse', async () => {
        browser.resize(800, 600);
        const tab = await browser.openTab({ url: getPageUrl("mouse.html"), wait_until_loaded: true });

        await tab.mouseMove(300, 400);
        await expect.poll(() => tab.title(), pollTimeout).toBe("Move: (300, 400)");

        const buttons = [MouseButton.Left, MouseButton.Middle, MouseButton.Right];
        buttons.forEach((button, i) => async () => {
            await tab.click(100, 200, button, true);
            await expect.poll(() => tab.title(), pollTimeout).toBe(`Mouse Down: (100, 200) Button: ${i}`);

            await tab.click(100, 200, button, false);
            await expect.poll(() => tab.title(), pollTimeout).toBe(`Mouse Up: (100, 200) Button: ${i}`);
        });

        await tab.scroll(250, 350, 30, 50);
        await expect.poll(() => tab.title(), pollTimeout).toMatch("Scroll: (250, 350) Delta: (-30, -50)");
    });

    test.skip('keyboard', async () => {
        let press = async (tab: Tab, code: KeyCode) => {
            tab.key(code, 0, true, false, false);
            await new Promise(resolve => setTimeout(resolve, 20));
            tab.key(code, 0, false, false, false);
        }

        let tab = await browser.openTab({ url: getPageUrl("keyboard.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Keyboard");

        const text = "Hello, World! 🌍 Café, naïve, résumé Здравствуй мир こんにちは世界 مرحبا بالعالم Γεια σας κόσμε α²+β²=γ² ∑∞∫∆ €$¥£₹₽ ©®™℠ 🚀🎉🎯⚡🔥💎";
        for (let char of Array.from(text)) {
            if (char.length === 2) {
                tab.char(char.charCodeAt(0));
                tab.char(char.charCodeAt(1));
            } else {
                tab.char(char.charCodeAt(0));
            }
        }

        press(tab, KeyCode.ENTER);
        await expect.poll(() => tab.title(), pollTimeout).toBe(text);

        tab.selectAll();
        tab.cut();
        press(tab, KeyCode.ENTER);
        await expect.poll(() => tab.title(), pollTimeout).toBe("Keyboard");

        tab.paste();
        press(tab, KeyCode.ENTER);
        await expect.poll(() => tab.title(), pollTimeout).toBe(text);

        press(tab, KeyCode.BACKSPACE);
        press(tab, KeyCode.ENTER);
        await expect.poll(() => tab.title(), pollTimeout).toBe(text.slice(0, -2));
    });

    test.skip('screenshot', async () => {
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

    test.skip('subframes', async () => {
        const tab = await browser.openTab({ url: getPageUrl("frames.html"), wait_until_loaded: true });
        expect(await tab.title()).toBe("Frames");
    });
});

