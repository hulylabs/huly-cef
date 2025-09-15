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

    // test('restore session', async () => {
    //     const { cef, finished } = await launchCef(port, "cache/server", 5000);
    //     const browser = await connect("ws://localhost:" + port + "/browser");
    //     const tab = await browser.openTab({ url: getPageUrl("title.html"), wait_until_loaded: true });
    //     expect(tab).toBeDefined();
    //     expect(await tab.title()).toBe("Title");

    //     await browser.close();
    //     await finished;

    //     const { cef: cef2, finished: finished2 } = await launchCef(port, "cache/server", 5000);

    //     const browser2 = await connect("ws://localhost:" + port + "/browser");
    //     let tabs = await browser2.restoreTabs();

    //     expect(tabs.length).toBe(1);
    //     expect(tabs[0].id).toBe(tab.id);
    //     expect(await tabs[0].title()).toBe("Title");

    //     await browser2.close();
    //     await exited;
    // });
});

