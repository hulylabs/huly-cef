import { afterAll, beforeAll, describe, expect, it, test, vi } from 'vitest';
import sharp from 'sharp';

import { Browser, connect, KeyCode, MouseButton } from '../src/index';

import { GenericContainer, StartedTestContainer, Wait } from "testcontainers";
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const testdir = dirname(fileURLToPath(import.meta.url));

describe('Huly CEF Websockets', () => {
    let cefContainer: StartedTestContainer;
    let browser: Browser;
    let port: number;

    beforeAll(async () => {
        cefContainer = await new GenericContainer("huly-cef")
            .withCopyDirectoriesToContainer([{
                source: join(testdir, "testpages"),
                target: join(testdir, "testpages"),
            }])
            .withExposedPorts(8080)
            .withWaitStrategy(Wait.forListeningPorts())
            .start();

        port = cefContainer.getMappedPort(8080);
        // port = 8080;
        browser = await connect("ws://localhost:" + port + "/browser");
    });

    test('open a new tab', async () => {
        const url = "file://" + testdir + "/testpages/title.html";
        const tab = await browser.openTab({ url });
        expect(tab.id).toBeDefined();
        await expect.poll(() => tab.title()).toBe("Title");
        await expect.poll(() => tab.url()).toBe(url);

        await tab.close();
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test('load state', async () => {
    });

    test('resize', async () => {
        let [width, height] = [800, 600];
        browser.resize(width, height);

        const url = "file://" + testdir + "/testpages/title.html";
        const tab = await browser.openTab({ url });
        let screenshot = await tab.screenshot();
        expect(screenshot).toBeDefined();

        let img = Buffer.from(screenshot, 'base64');
        let metadata = await sharp(img).metadata();
        expect(metadata.width).toBe(width);
        expect(metadata.height).toBe(height);
        expect(metadata.format).toBe('png');

        [width, height] = [1024, 768];
        browser.resize(width, height);

        screenshot = await tab.screenshot();
        expect(screenshot).toBeDefined();

        img = Buffer.from(screenshot, 'base64');
        metadata = await sharp(img).metadata();
        expect(metadata.width).toBe(width);
        expect(metadata.height).toBe(height);
        expect(metadata.format).toBe('png');

        tab.close();
    });

    test('go to a url', async () => {
        const tab = await browser.openTab();
        expect(tab.id).toBeDefined();

        tab.navigate("https://www.google.com/");

        await expect.poll(() => tab.title()).toBe("Google");
        tab.close();
    });

    test.skip('multiple tabs', async () => {
        let client = await connect("ws://localhost:" + port + "/browser");
        await browser.openTab({ url: "file://" + testdir + "/testpages/title.html" });
        await client.openTab({ url: "file://" + testdir + "/testpages/keyboard.html" });
        await client.openTab({ url: "file://" + testdir + "/testpages/links.html" });

        await new Promise(resolve => setTimeout(resolve, 1000));

        const tabs = await browser.tabs();
        const titles = (await Promise.all(tabs.map(tab => tab.title()))).sort();
        expect(titles).toEqual(["Title", "Keyboard", "Links"].sort());

        tabs.forEach(tab => tab.close());
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test.skip('tab navigation', async () => {
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/links.html" });
        await expect.poll(() => tab.title(), { interval: 200, timeout: 5000 }).toBe("Links");

        let elements = await tab.clickableElements();
        expect(elements[0].id).toBe(0);
        expect(elements[0].tag).toBe("a");
        expect(elements[0].text).toBe("External Link (Title)");

        tab.clickElement(elements[0].id);
        await expect.poll(() => tab.title()).toBe("Title");

        tab.back();
        await expect.poll(() => tab.title()).toBe("Links");

        tab.forward();
        await expect.poll(() => tab.title()).toBe("Title");

        tab.navigate("file://" + testdir + "/testpages/reload.html");
        await expect.poll(() => tab.title()).toBe("Reloads: 1");

        tab.reload();
        await expect.poll(() => tab.title()).toBe("Reloads: 2");

        tab.close();
        await expect.poll(() => browser.tabs()).toEqual([]);
    });

    test.skip('mouse', async () => {
        browser.resize(800, 600);
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/mouse.html" });
        expect(tab.id).toBeDefined();
        await expect.poll(() => tab.title()).toBe("Mouse");

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

    test.skip('keyboard', async () => {
        let tab = await browser.openTab({ url: "file://" + testdir + "/testpages/keyboard.html" });
        expect(tab.id).toBeDefined();
        await expect.poll(() => tab.title()).toBe("Keyboard");

        const unicodeTexts = [
            "Hello, World! 🌍",
            "Café, naïve, résumé",
            "Здравствуй мир",
            "こんにちは世界",
            "مرحبا بالعالم",
            "Γεια σας κόσμε",
            "α²+β²=γ² ∑∞∫∆",
            "€$¥£₹₽¢",
            "©®™℠",
            "→←↑↓⇄⇅⇆⇇",
            "♠♣♥♦♪♫♬",
            "🚀🎉🎯⚡🔥💎",
            "中文测试文字",
            "한글 테스트",
            "עברית בדיקה",
            "Ñoño niño",
            "Ümlauts: äöü",
            "Français: çàéèêë",
        ];

        for (const text of unicodeTexts) {
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
        }

        await tab.close();
    });

    test.skip('screenshot', async () => {
        browser.resize(1920, 1080);

        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/title.html" });
        expect(tab.id).toBeDefined();
        await expect.poll(() => tab.title()).toBe("Title");

        let width = 800;
        let height = 600;
        const screenshot = await tab.screenshot({ size: [width, height] });
        expect(screenshot).toBeDefined();

        const img = Buffer.from(screenshot, 'base64');
        const metadata = await sharp(img).metadata();
        expect(metadata.width).toBe(width);
        expect(metadata.height).toBe(height);
        expect(metadata.format).toBe('png');

        await tab.close();
    });

    afterAll(async () => {
        // if (browser) {
        //     browser.close();
        // }
        (await cefContainer.logs())
            .on("data", line => console.log(line))
            .on("err", line => console.error(line))
            .on("end", () => console.log("Stream closed"));
        await cefContainer.stop();
    });
});

