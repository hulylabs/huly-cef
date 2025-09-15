import { ChildProcess, spawn } from "child_process";
import { dirname } from "path";
import { fileURLToPath } from "url";

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
    return `file://${testdir}/testpages/${page}`;
};

export interface CefProcess {
    cef: ChildProcess;
    finished: Promise<number | null>;
}

export async function launchCef(port: number, cache: string, timeout: number): Promise<CefProcess> {
    const cef = spawn(cefExe, ["--port", port.toString(), "--cache-path", cache]);
    const finished = new Promise<number | null>((resolve) => cef.on('exit', (code) => resolve(code)));
    await new Promise(resolve => setTimeout(resolve, timeout));
    return { cef, finished };
};