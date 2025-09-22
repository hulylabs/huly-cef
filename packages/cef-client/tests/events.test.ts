import { afterAll, beforeAll, describe, expect, test } from 'vitest';

import { Browser, connect } from '../src/index';
import { Cursor, LoadState, LoadStatus } from '../src/types';
import { CefProcess, getPageUrl, launchCef, pollTimeout } from './common';

describe('tab events', () => {
    let cef_process: CefProcess;
    let browser: Browser;
    let port: number;

    beforeAll(async () => {
        port = 8082;
        cef_process = await launchCef(port, "cache/events", 5000);
        browser = await connect("ws://localhost:" + port + "/browser");
    });

    afterAll(() => {
        cef_process.cef.kill();
    });

    test('basic', async () => {
        const tab = await browser.openTab({ url: getPageUrl("events.html"), wait_until_loaded: false });
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

        await expect.poll(() => title, pollTimeout).toBe("Events");
        await expect.poll(() => url, pollTimeout).toBe(getPageUrl("events.html"));
        await expect.poll(() => loadState, pollTimeout).toStrictEqual(expectedLoadedState);
        await expect.poll(() => favicon, pollTimeout).toBe(getPageUrl("favicon.svg"));
        await expect.poll(() => cursor, pollTimeout).toBe(Cursor.Pointer);

        tab.close();
    });

    test('video', async () => {
        const tab = await browser.openTab({ url: getPageUrl("events.html") });
        expect(tab.id).toBeDefined();

        let width = 640;
        let height = 360;
        browser.resize(width, height);

        let stream = tab.events();
        let frames: number[] = [];
        stream.on("Frame", (data) => frames.push(data.data.length));

        await expect.poll(() => frames.length, pollTimeout).toBeGreaterThan(10);

        tab.stopVideo();
        await new Promise(resolve => setTimeout(resolve, 100));
        let framecount = frames.length;
        await new Promise(resolve => setTimeout(resolve, 100));
        expect(frames.length).toEqual(framecount);

        tab.startVideo();
        await expect.poll(() => frames.length, pollTimeout).toBeGreaterThan(framecount * 2);

        expect(frames.every(frame => frame === width * height * 4)).toBe(true);

        tab.close();
    });

    test.skip('download', async () => {
        const tab = await browser.openTab({ url: getPageUrl("download.html") });
        let events = tab.events();

        let name = "test.txt";
        let size = 17;
        let content = "Test file content";

        let filePath = "";
        let received = 0;
        let total = 0;

        events.on("DownloadProgress", (progress) => {
            filePath = progress.path;
            received = progress.received;
            total = progress.total;
        });

        await expect.poll(() => total, pollTimeout).toBe(size);
        await expect.poll(() => received, pollTimeout).toBe(size);
        await expect.poll(() => filePath.endsWith(name), pollTimeout).toBe(true);

        const fs = await import('fs');
        expect(fs.readFileSync(filePath, 'utf8')).toBe(content);
    });

    test('new tab', async () => {
    });

    test('multiple subscribers', async () => {
    });
});

