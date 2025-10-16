export interface OpenTabOptions {
    url: string;
    wait_until_loaded: boolean;
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

export enum LoadStatus {
    Loading = 0,
    Loaded = 1,
    Error = 2,
}

export type LoadState = {
    status: LoadStatus;
    canGoBack: boolean;
    canGoForward: boolean;
    errorCode?: number;
    errorMessage?: string;
};

export enum Cursor {
    Pointer = "Pointer",
    Hand = "Hand",
    IBeam = "IBeam",
    Crosshair = "Crosshair",
}

export interface DownloadProgress {
    id: number;
    path: string;
    received: number;
    total: number;
    is_complete: boolean;
    is_aborted: boolean;
}

export interface FileDialog {
    mode: number;
    title: string;
    default_file_path: string;
    accept_types: string[];
    accept_extensions: string[];
    accept_descriptions: string[];
}

export interface Frame {
    width: number;
    height: number;
    data: Uint8Array;
}