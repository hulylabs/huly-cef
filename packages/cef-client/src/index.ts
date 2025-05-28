import {
  KeyCode,
  keyCodeToMacOSVirtualKey,
  keyCodeToWindowsVirtualKey,
} from "./keyboard";

export { KeyCode };

enum Platform {
  Windows,
  MacOS,
  Linux,
}

function detectPlatform(): Platform {
  const platform = navigator.userAgent;
  if (platform.includes("Windows")) {
    return Platform.Windows;
  }
  if (platform.includes("Mac")) {
    return Platform.MacOS;
  }
  return Platform.Linux;
}

export enum LoadStateKind {
  Loading,
  Loaded,
  Error,
}

export type LoadState = {
  LoadStateKind: LoadStateKind;
  canGoBack: boolean;
  canGoForward: boolean;
  error_code?: number;
  error_message?: string;
};

// export enum CursorType {
//     Default = "default",
//     Pointer = "pointer",
//     Text = "text",
//     Wait = "wait",
//     Crosshair = "crosshair",
//     Move = "move",
//     NotAllowed = "not-allowed",
// }

const HEARTBEAT_INTERVAL = 5000;

export class CEFClient {
  websocket: WebSocket;
  pendingMessages: string[] = [];

  heartbeatInterval: NodeJS.Timeout | number | undefined;

  size: { width: number; height: number } = { width: 0, height: 0 };
  url: string = "";
  platform: Platform = detectPlatform();

  public onConnectionBroken: (() => void) | undefined;
  public onLoadStateChanged: ((state: LoadState) => void) | undefined;
  public onTitleChanged: ((title: string) => void) | undefined;
  public onUrlChanged: ((url: string) => void) | undefined;
  public onFaviconUrlChanged: ((url: string) => void) | undefined;
  public onCursorChanged: ((cursor: string) => void) | undefined;
  public onNewTabRequested: ((url: string) => void) | undefined;
  public onRender: ((data: Uint8Array) => void) | undefined;
  public onPopupRender:
    | ((x: number, y: number, w: number, h: number, data: Uint8Array) => void)
    | undefined;

  constructor(url: string) {
    this.websocket = this.createWebSocket(url);
    this.startHeartbeat();
  }

  goTo(url: string) {
    this.send(JSON.stringify({ GoTo: { url: url } }));
  }

  onMouseMove(x: number, y: number) {
    this.send(JSON.stringify({ MouseMove: { x: x, y: y } }));
  }

  onMouseDown(x: number, y: number, button: number) {
    this.send(
      JSON.stringify({
        MouseClick: { x: x, y: y, down: true, button: button },
      }),
    );
  }

  onMouseUp(x: number, y: number, button: number) {
    this.send(
      JSON.stringify({
        MouseClick: { x: x, y: y, down: false, button: button },
      }),
    );
  }

  onMouseWheel(x: number, y: number, dx: number, dy: number) {
    this.send(JSON.stringify({ MouseWheel: { x: x, y: y, dx: dx, dy: dy } }));
  }

  onKeyPress(
    keycode: KeyCode,
    character: number,
    down: boolean,
    ctrl: boolean,
    shift: boolean,
  ) {
    let platformKeyCode = 0;
    switch (this.platform) {
      case Platform.Windows:
      case Platform.Linux:
        platformKeyCode = keyCodeToWindowsVirtualKey(keycode);
        break;
      case Platform.MacOS:
        platformKeyCode = keyCodeToMacOSVirtualKey(keycode);
        break;
    }
    this.send(
      JSON.stringify({
        KeyPress: {
          character: character,
          windowscode: keyCodeToWindowsVirtualKey(keycode),
          code: platformKeyCode,
          down: down,
          ctrl: ctrl,
          shift: shift,
        },
      }),
    );
  }

  resize(width: number, height: number) {
    this.size.width = Math.floor(width);
    this.size.height = Math.floor(height);

    this.send(JSON.stringify({ Resize: this.size }));
  }

  startVideo() {
    this.send(JSON.stringify("StartVideo"));
  }

  stopVideo() {
    this.send(JSON.stringify("StopVideo"));
  }

  close() {
    clearInterval(this.heartbeatInterval);
    this.send(JSON.stringify("Close"));
  }

  goBack() {
    this.send(JSON.stringify("GoBack"));
  }

  goForward() {
    this.send(JSON.stringify("GoForward"));
  }

  reload() {
    this.send(JSON.stringify("Reload"));
  }

  private createWebSocket(url: string) {
    let websocket = new WebSocket(url);
    websocket.binaryType = "arraybuffer";
    websocket.onopen = () => this.onopen();
    websocket.onmessage = (event) => this.onmessage(event);

    return websocket;
  }

  private onopen() {
    for (let i = 0; i < this.pendingMessages.length; i++) {
      this.websocket.send(this.pendingMessages[i]);
    }
    this.pendingMessages = [];
  }

  private onmessage(event: MessageEvent) {
    if (event.data instanceof ArrayBuffer) {
      let data = new Uint8Array(event.data);

      if (data[0] == 0) {
        this.onRender?.(data.subarray(1));
      } else {
        let x = data[1] | (data[2] << 8) | (data[3] << 16) | (data[4] << 24);
        let y = data[5] | (data[6] << 8) | (data[7] << 16) | (data[8] << 24);
        let w = data[9] | (data[10] << 8) | (data[11] << 16) | (data[12] << 24);
        let h =
          data[13] | (data[14] << 8) | (data[15] << 16) | (data[16] << 24);

        this.onPopupRender?.(x, y, w, h, data.subarray(17));
      }
      return;
    }

    if (typeof event.data === "string") {
      let parsed = JSON.parse(event.data);

      if (typeof parsed === "object") {
        if (parsed.TitleChanged) {
          this.onTitleChanged?.(parsed.TitleChanged);
        }

        if (parsed.CursorChanged) {
          this.onCursorChanged?.(parsed.CursorChanged);
        }

        if (parsed.UrlChanged) {
          this.url = parsed.UrlChanged;
          this.onUrlChanged?.(parsed.UrlChanged);
        }

        if (parsed.NewTabRequested) {
          this.onNewTabRequested?.(parsed.NewTabRequested);
        }

        if (parsed.FaviconUrlChanged) {
          this.onFaviconUrlChanged?.(parsed.FaviconUrlChanged);
        }

        if (parsed.LoadStateChanged) {
          let state = parsed.LoadStateChanged;
          let loadState: LoadState = {
            LoadStateKind: LoadStateKind.Loading,
            canGoBack: state.can_go_back,
            canGoForward: state.can_go_forward,
          };

          switch (state.state) {
            case "Loading":
              loadState.LoadStateKind = LoadStateKind.Loading;
              break;
            case "Loaded":
              loadState.LoadStateKind = LoadStateKind.Loaded;
              break;
            case "LoadError":
              loadState.LoadStateKind = LoadStateKind.Error;
              break;
          }

          if (state.error_code != 0) {
            loadState.error_code = state.error_code;
          }

          if (state.error_message != "") {
            loadState.error_message = state.error_message;
          }

          this.onLoadStateChanged?.(loadState);
        }
      }
    }
  }

  private send(message: string) {
    switch (this.websocket.readyState) {
      case WebSocket.CONNECTING:
        this.pendingMessages.push(message);
        break;
      case WebSocket.OPEN:
        this.websocket.send(message);
        break;
    }
  }

  private startHeartbeat() {
    this.heartbeatInterval = setInterval(() => {
      if (this.websocket.readyState === WebSocket.CLOSED) {
        this.websocket = this.createWebSocket(this.websocket.url);
        this.send(JSON.stringify({ Resize: this.size }));
        this.send(JSON.stringify({ GoTo: { url: this.url } }));
      }
    }, HEARTBEAT_INTERVAL);
  }
}
