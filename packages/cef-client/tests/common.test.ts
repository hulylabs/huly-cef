import { dirname } from "path";
import { fileURLToPath } from "url";

export const testdir = dirname(fileURLToPath(import.meta.url));
export const pollTimeout = { timeout: 5000, interval: 200 };

export const cef_exe = (() => {
    if (process.platform === "darwin") {
        return "../../target/release/huly-cef-websockets.app/Contents/MacOS/huly-cef-websockets";
    } else if (process.platform === "win32") {
        return "../../target/release/huly-cef-websockets.exe";
    } else {
        return "../../target/release/huly-cef-websockets";
    }
})();