import { afterAll, beforeAll, describe, expect, inject, test } from 'vitest';

import { Browser, connect } from '../src/index';

import { Cursor, LoadState, LoadStatus } from '../src/event_stream';
import { GenericContainer, StartedTestContainer, Wait } from 'testcontainers';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const testdir = dirname(fileURLToPath(import.meta.url));

describe('Tab Events', () => {
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

    afterAll(async () => {
        (await cefContainer.logs())
            .on("data", line => console.log(line))
            .on("err", line => console.error(line))
            .on("end", () => console.log("Stream closed"));
        await cefContainer.stop();
    });

    test('basic', async () => {
        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/events.html", wait_until_loaded: false });
        expect(tab.id).toBeDefined();

        let title = "";
        let url = "";
        let loadState: LoadState | null;
        let favicon = "";
        let cursor = "";

        let stream = tab.events();

        stream.on("Title", (data) => title = data);
        stream.on("Url", (data) => url = data);
        stream.on("LoadState", (data) => loadState = data);
        stream.on("Favicon", (data) => favicon = data);
        stream.on("Cursor", (data) => cursor = data);

        let expectedLoadedState: LoadState = {
            status: LoadStatus.Loaded,
            canGoBack: false,
            canGoForward: false,
            errorCode: 0,
            errorMessage: "",
        };

        await expect.poll(() => title).toBe("Events");
        await expect.poll(() => url).toBe("file://" + testdir + "/testpages/events.html");
        await expect.poll(() => loadState).toStrictEqual(expectedLoadedState);
        await expect.poll(() => favicon).toBe("file://" + testdir + "/testpages/favicon.svg");
        await expect.poll(() => cursor).toBe(Cursor.Pointer);

        tab.close();
    });

    test('video', async () => {

        const tab = await browser.openTab({ url: "file://" + testdir + "/testpages/events.html" });
        expect(tab.id).toBeDefined();

        let width = 640;
        let height = 360;
        browser.resize(width, height);

        let stream = tab.events();
        let frames: number[] = [];
        stream.on("Frame", (data) => {
            frames.push(data.length);
        });

        await expect.poll(() => frames.length).toBeGreaterThan(10);

        tab.stopVideo();
        await new Promise(resolve => setTimeout(resolve, 100));
        let framecount = frames.length;
        await new Promise(resolve => setTimeout(resolve, 100));
        expect(frames.length).toEqual(framecount);

        tab.startVideo();
        await expect.poll(() => frames.length).toBeGreaterThan(framecount * 2);

        for (let i = 0; i < frames.length; i++) {
            console.log(`Frame ${i + 1}: ${frames[i]} bytes`);
        }

        console.log(`width * height * 4 = ${width * height * 4}`);
        expect(frames.every(frame => frame === width * height * 4)).toBe(true);

        tab.close();
    });

    test('new tab', async () => {
    });

    test('multiple subscribers', async () => {
    });
});

