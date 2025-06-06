import { afterAll, beforeAll, describe, expect, it, test } from 'vitest';

import { BrowserClient } from '../src/index';

import { GenericContainer, StartedTestContainer, waitForContainer, Wait } from "testcontainers";

describe('BrowserClient', () => {
    let cefContainer: StartedTestContainer;

    beforeAll(async () => {
        cefContainer = await new GenericContainer("huly-cef-server")
            .withExposedPorts(8080)
            .withWaitStrategy(Wait.forListeningPorts())
            .start();
    });

    it('open tab', async () => {
        let port = cefContainer.getMappedPort(8080);
        const client = new BrowserClient("ws://localhost:" + port + "/browser");
        const id = await client.openTab("https://google.com");

        expect(id).toBeDefined();
        expect(id).toBeGreaterThan(0);
    });

    afterAll(async () => {
        (await cefContainer.logs())
            .on("data", line => console.log(line))
            .on("err", line => console.error(line))
            .on("end", () => console.log("Stream closed"));
        await cefContainer.stop();
    });
});

