import { beforeAll, describe, expect, test } from 'vitest';
import sharp from 'sharp';

import { Browser, connect, KeyCode, MouseButton } from '../src/index';

import { dirname } from 'path';
import { fileURLToPath, pathToFileURL } from 'url';

const testdir = dirname(fileURLToPath(import.meta.url));

describe('Basic API', () => {
    let browser: Browser;
    let port: number;

    beforeAll(async () => {
        port = 8080;
        browser = await connect("ws://localhost:" + port + "/browser");
    });

    test.only('open a new tab', async () => {
        const url = pathToFileURL(testdir + "/testpages/title.html").href;
        const tab = await browser.openTab({ url: url, wait_until_loaded: true });
        expect(tab.id).toBeDefined();
        expect(await tab.title()).toBe("Title");
        expect(await tab.url()).toBe(url);

        await tab.close();
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test('resize', async () => {
        let [width, height] = [800, 600];

        const url = "file://" + testdir + "/testpages/resize.html";
        const tab = await browser.openTab({ url, width, height });
        await expect.poll(() => tab.title()).toBe("800x600");

        [width, height] = [1024, 768];
        browser.resize(width, height);
        await expect.poll(() => tab.title()).toBe("1024x768");

        tab.close();
    });

    test('go to a url', async () => {
        const tab = await browser.openTab({ url: "https://www.google.com", wait_until_loaded: true });
        expect(tab.id).toBeDefined();

        await tab.navigate("https://www.google.com/", true);
        expect(await tab.title()).toBe("Google");
        tab.close();
    });

    test('multiple tabs', async () => {
        let client = await connect("ws://localhost:" + port + "/browser");
        await browser.openTab({ url: "file://" + testdir + "/testpages/title.html", wait_until_loaded: true });
        await client.openTab({ url: "file://" + testdir + "/testpages/keyboard.html", wait_until_loaded: true });
        await client.openTab({ url: "file://" + testdir + "/testpages/links.html", wait_until_loaded: true });

        const tabs = await browser.tabs();
        const titles = (await Promise.all(tabs.map(tab => tab.title()))).sort();
        expect(titles).toEqual(["Title", "Keyboard", "Links"].sort());

        tabs.forEach(tab => tab.close());
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test('tab navigation', async () => {
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/links.html", wait_until_loaded: true });
        expect(await tab.title()).toBe("Links");

        let elements = await tab.clickableElements();
        expect(elements[0].id).toBe(0);
        expect(elements[0].tag).toBe("a");
        expect(elements[0].text).toBe("External Link (Title)");

        tab.clickElement(elements[0].id);
        await expect.poll(() => tab.title()).toBe("Title");

        tab.back(true);
        expect(await tab.title()).toBe("Links");

        tab.forward(true);
        expect(await tab.title()).toBe("Title");

        tab.close();
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test('tab reloading', async () => {
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/reload.html", wait_until_loaded: true });
        expect(tab.id).toBeDefined();

        tab.reload(true);
        expect(await tab.title()).toBe("Reloads: 2");

        tab.close();
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test('mouse', async () => {
        browser.resize(800, 600);
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/mouse.html", wait_until_loaded: true });
        expect(tab.id).toBeDefined();
        expect(await tab.title()).toBe("Mouse");

        // Mouse Move
        await tab.mouseMove(300, 400);
        await expect.poll(() => tab.title()).toBe("Move: (300, 400)");

        // Left Button
        await tab.click(100, 200, MouseButton.Left, true);
        await expect.poll(() => tab.title()).toBe("Mouse Down: (100, 200) Button: 0");

        await tab.click(100, 200, MouseButton.Left, false);
        await expect.poll(() => tab.title()).toBe("Mouse Up: (100, 200) Button: 0");

        // Middle Button
        await tab.click(150, 250, MouseButton.Middle, true);
        await expect.poll(() => tab.title()).toBe("Mouse Down: (150, 250) Button: 1");

        await tab.click(150, 250, MouseButton.Middle, false);
        await expect.poll(() => tab.title()).toBe("Mouse Up: (150, 250) Button: 1");

        // Right Button
        await tab.click(200, 300, MouseButton.Right, true);
        await expect.poll(() => tab.title()).toBe("Mouse Down: (200, 300) Button: 2");

        await tab.click(200, 300, MouseButton.Right, false);
        await expect.poll(() => tab.title()).toBe("Mouse Up: (200, 300) Button: 2");

        // Scroll
        await tab.scroll(250, 350, 0, 100);
        await expect.poll(() => tab.title()).toMatch("Scroll: (250, 350) Delta: (0, 100)");

        await tab.scroll(250, 350, 0, -100);
        await expect.poll(() => tab.title()).toMatch("Scroll: (250, 350) Delta: (0, -100)");

        await tab.close();
    });

    test('keyboard', async () => {
        let tab = await browser.openTab({ url: "file://" + testdir + "/testpages/keyboard.html", wait_until_loaded: true });
        expect(tab.id).toBeDefined();
        expect(await tab.title()).toBe("Keyboard");

        const text = "Hello, World! 🌍 Café, naïve, résumé Здравствуй мир こんにちは世界 مرحبا بالعالم Γεια σας κόσμε α²+β²=γ² ∑∞∫∆ €$¥£₹₽ ©®™℠ 🚀🎉🎯⚡🔥💎";
        // Enter text
        for (let char of Array.from(text)) {
            if (char.length === 2) {
                tab.char(char.charCodeAt(0));
                tab.char(char.charCodeAt(1));
            } else {
                tab.char(char.charCodeAt(0));
            }
        }

        // Press Enter to submit
        tab.key(KeyCode.ENTER, 0, true, false, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.ENTER, 0, false, false, false);

        // Wait and verify the text was entered correctly
        await expect.poll(() => tab.title()).toBe(text);

        // Clear for next test (Ctrl+A then Delete)
        tab.key(KeyCode.KEY_A, 0, true, true, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.KEY_A, 0, false, true, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.DELETE, 0, true, false, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.DELETE, 0, false, false, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.ENTER, 0, true, false, false);
        await new Promise(resolve => setTimeout(resolve, 20));
        tab.key(KeyCode.ENTER, 0, false, false, false);

        await expect.poll(() => tab.title()).toBe("Keyboard");

        tab.close();
    });

    test('screenshot', async () => {
        browser.resize(1920, 1080);

        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/title.html", wait_until_loaded: true });
        expect(tab.id).toBeDefined();
        expect(await tab.title()).toBe("Title");

        let width = 800;
        let height = 600;
        const screenshot = await tab.screenshot({ size: { width, height } });
        expect(screenshot).toBeDefined();

        const img = Buffer.from(screenshot, 'base64');
        const metadata = await sharp(img).metadata();
        expect(metadata.width).toBe(width);
        expect(metadata.height).toBe(height);
        expect(metadata.format).toBe('png');

        await tab.close();
    });
});

