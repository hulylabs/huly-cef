export const REQUEST_TIMEOUT = 30000;

export interface OpenTabOptions {
    url?: string;
}

export interface ScreenshotOptions {
    size?: {
        width: number;
        height: number;
    };
}

export interface ClickableElement {
    tag: string;
    text: string;
    x: number;
    y: number;
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
