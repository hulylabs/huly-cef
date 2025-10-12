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
});

