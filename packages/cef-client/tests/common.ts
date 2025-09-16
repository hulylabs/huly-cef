import { ChildProcess, spawn } from "child_process";
import { dirname, resolve } from "path";
import { fileURLToPath, pathToFileURL } from "url";

export const pollTimeout = { timeout: 5000, interval: 200 };

const cefExe = (() => {
    if (process.platform === "darwin") {
        return "../../target/release/huly-cef-websockets.app/Contents/MacOS/huly-cef-websockets";
    } else if (process.platform === "win32") {
        return "../../target/release/huly-cef-websockets.exe";
    } else {
        return "../../target/release/huly-cef-websockets";
    }
})();

const testdir = dirname(fileURLToPath(import.meta.url));
export const getPageUrl = (page: string) => {
    const fullPath = resolve(testdir, "testpages", page);
    return pathToFileURL(fullPath).href;
};

export interface CefProcess {
    cef: ChildProcess;
    finished: Promise<number | null>;
}

export async function launchCef(port: number, cache: string, timeout: number): Promise<CefProcess> {
    const cef = spawn(cefExe, ["--port", port.toString(), "--cache-path", cache]);
    cef.on('error', (err) => {
        console.error("Failed to start CEF process:", err);
    });

    cef.on('exit', (code, signal) => {
        console.log(`CEF process exited with code ${code} and signal ${signal}`);
    });

    cef.on('close', (code) => {
        console.log(`CEF process closed with code ${code}`);
    });

    cef.on('disconnect', () => {
        console.log("CEF process disconnected");
    });

    cef.on('message', (message) => {
        console.log("CEF process message:", message);
    });
    const finished = new Promise<number | null>((resolve) => cef.on('exit', (code) => resolve(code)));
    await new Promise(resolve => setTimeout(resolve, timeout));
    return { cef, finished };
};