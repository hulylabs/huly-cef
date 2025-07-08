import { afterAll, beforeAll, describe, expect, it, test, vi } from 'vitest';
import sharp from 'sharp';

import { Browser, connect, KeyCode } from '../src/index';

import { GenericContainer, StartedTestContainer, Wait } from "testcontainers";
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const testdir = dirname(fileURLToPath(import.meta.url));

describe('BrowserClient', () => {
    let cefContainer: StartedTestContainer;
    let browser: Browser;
    let port: number;

    beforeAll(async () => {
        cefContainer = await new GenericContainer("huly-cef")
            .withCopyDirectoriesToContainer([{
                source: join(testdir, "testpages"),
                target: "/testpages"
            }])
            .withExposedPorts(8080)
            .withWaitStrategy(Wait.forListeningPorts())
            .start();

        port = cefContainer.getMappedPort(8080);
        browser = await connect("ws://localhost:" + 8080 + "/browser");
    });

    test('open a new tab', async () => {
        const url = "https://www.google.com/";
        const tab = await browser.openTab({ url });
        expect(tab.id).toBeDefined();
        expect(await tab.title()).toBe("Google");
        expect(await tab.url()).toBe(url);

        await tab.close();
        await expect.poll(() => browser.getTabs(), { interval: 2000 }).toEqual([]);
    });

    test('resize the browser', async () => {
        let [width, height] = [800, 600];
        browser.resize(width, height);

        const url = "https://www.google.com/";
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

    // Connect to the server with the second client and create more tabs. Then check if the first client can see them.

    // test('multiple tabs', async () => {
    //     const id1 = await client.openTab("https://example.com");
    //     const id2 = await client.openTab("https://www.google.com");
    //     const id3 = await client.openTab("https://youtube.com");

    //     await expect.poll(async () => (await client.getTabs()).sort()).toEqual([
    //         "https://example.com",
    //         "https://www.google.com",
    //         "https://youtube.com"
    //     ].sort());

    //     client.closeTab(id1);
    //     client.closeTab(id2);
    //     client.closeTab(id3);
    //     await expect.poll(() => client.getTabs()).toEqual([]);
    // });

    // test('browser navigation (back and forward)', async () => {
    //     const id = await client.openTab("file:///testpages/input.html");
    //     // client.keyPress(id, KeyCode.TAB, 0, false, false, false);
    //     client.setFocus(id, true);
    //     client.keyPress(id, KeyCode.KEY_A, 'a'.charCodeAt(0), false, false, false);
    //     await expect.poll(() => client.getTabs(), { timeout: 100000, interval: 3000 }).toEqual(["https://www.google.com/"]);
    //     await expect.poll(() => client.getTabs(), { timeout: 100000, interval: 3000 }).toEqual(["https://www.google.com/"]);

    //     client.goBack(id);
    //     await expect.poll(() => client.getTabs()).toEqual(["https://youtube.com"]);
    //     client.goForward(id);
    //     await expect.poll(() => client.getTabs()).toEqual(["https://www.google.com"]);
    //     client.closeTab(id);
    //     await expect.poll(() => client.getTabs()).toEqual([]);
    // }, 200000);

    afterAll(async () => {
        // if (browser) {
        //     browser.close();
        // }
        // (await cefContainer.logs())
        //     .on("data", line => console.log(line))
        //     .on("err", line => console.error(line))
        //     .on("end", () => console.log("Stream closed"));
        await cefContainer.stop();
    });
});

