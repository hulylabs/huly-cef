export const REQUEST_TIMEOUT = 30000;
export const DEFAULT_URL = 'about:blank';
export const DEFAULT_WIDTH = 800;
export const DEFAULT_HEIGHT = 600;

export interface OpenTabOptions {
    url: string;
    wait_until_loaded: boolean;
    width: number;
    height: number;
}

export interface ScreenshotOptions {
    size?: {
        width: number;
        height: number;
    }
}

export interface ClickableElement {
    id: number;
    tag: string;
    text: string;
}

export enum Platform {
    Windows,
    MacOS,
    Linux,
}

export function detectPlatform(): Platform {
    const platform = navigator.userAgent;
    if (platform.includes("Windows")) {
        return Platform.Windows;
    }
    if (platform.includes("Mac")) {
        return Platform.MacOS;
    }
    return Platform.Linux;
}


export enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}