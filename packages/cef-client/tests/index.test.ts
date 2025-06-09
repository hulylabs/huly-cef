import { afterAll, beforeAll, describe, expect, it, test, vi } from 'vitest';

import { BrowserClient, KeyCode } from '../src/index';

import { GenericContainer, StartedTestContainer, Wait } from "testcontainers";

describe('BrowserClient', () => {
    let cefContainer: StartedTestContainer;
    let client: BrowserClient;
    let port: number;

    beforeAll(async () => {
        cefContainer = await new GenericContainer("huly-cef-server")
            .withCopyDirectoriesToContainer([{
                source: "/home/nikita/repos/huly-cef/packages/cef-client/tests/testpages",
                target: "/testpages",
            }])
            .withExposedPorts(8080)
            .withWaitStrategy(Wait.forListeningPorts())
            .start();

        port = cefContainer.getMappedPort(8080);
        client = new BrowserClient("ws://localhost:" + port + "/browser");
    });

    // test('open a new tab', async () => {
    //     const url = "https://google.com";
    //     const id = await client.openTab(url);
    //     expect(id).toBeDefined();

    //     const tabs = await client.getTabs();
    //     expect(tabs).toBeDefined();
    //     expect(tabs.length).toBe(1);
    //     expect(tabs[0] === url).toBe(true);

    //     client.closeTab(id);
    //     await expect.poll(() => client.getTabs()).toEqual([]);
    // });

    // test('go to a url', async () => {
    //     let url = "https://www.google.com/";
    //     const id = await client.openTab("about:blank");
    //     client.goTo(id, url);

    //     await expect.poll(() => client.getTabs()).toEqual([url]);

    //     client.closeTab(id);
    //     await expect.poll(() => client.getTabs()).toEqual([]);
    // });

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

    test('browser navigation (back and forward)', async () => {
        const id = await client.openTab("file:///testpages/input.html");
        // client.keyPress(id, KeyCode.TAB, 0, false, false, false);
        client.setFocus(id, true);
        client.keyPress(id, KeyCode.KEY_A, 'a'.charCodeAt(0), false, false, false);
        await expect.poll(() => client.getTabs(), { timeout: 100000, interval: 3000 }).toEqual(["https://www.google.com/"]);
        await expect.poll(() => client.getTabs(), { timeout: 100000, interval: 3000 }).toEqual(["https://www.google.com/"]);

        client.goBack(id);
        await expect.poll(() => client.getTabs()).toEqual(["https://youtube.com"]);
        client.goForward(id);
        await expect.poll(() => client.getTabs()).toEqual(["https://www.google.com"]);
        client.closeTab(id);
        await expect.poll(() => client.getTabs()).toEqual([]);
    }, 200000);

    afterAll(async () => {
        (await cefContainer.logs())
            .on("data", line => console.log(line))
            .on("err", line => console.error(line))
            .on("end", () => console.log("Stream closed"));
        await cefContainer.stop();
    });
});

